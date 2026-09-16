import assert from 'node:assert/strict';
import { TauriBackendService } from '../solid/src/services/backend/tauri-backend.service';
import { createLibraryStore } from '../solid/src/stores/library.store';
import type { Game } from '../solid/src/types/game.types';
import { createRoot } from 'solid-js';
import { useJackettSearch } from '../solid/src/hooks/useJackettSearch';

const game: Game = {
  id: 'catalog-one', title: 'Catalog one', platform: 'ps2', platformName: 'PS2',
  releaseYear: 2000, genre: '', developer: '', publisher: '', rating: 0,
  playTimeMinutes: 0, favorite: false, coverImage: '', description: '', installed: false,
};
const second = { ...game, id: 'catalog-two', title: 'Catalog two' };
const installed = { ...game, installed: true, romPath: '/fixture/game.iso' };
const backend = new TauriBackendService();
const store = createLibraryStore(backend);
await store.loadGames([installed, second]);
assert.equal(store.games().length, 2);
assert.equal(store.games().find(entry => entry.id === game.id)?.installed, true);
const cachedGames = store.games();
const cachedGroups = store.catalogGames();
await store.loadGames([{ ...installed }, { ...second }]);
assert.equal(store.games(), cachedGames);
assert.equal(store.catalogGames(), cachedGroups);
await store.loadGames([{ ...installed, favorite: true }, { ...second }]);
assert.equal(store.games()[1], cachedGames[1]);
assert.equal(store.games()[0].favorite, true);
assert.notEqual(store.catalogGames(), cachedGroups);
await store.loadGames([installed, installed]);
assert.equal(store.games().length, 1);
await store.loadGames([]);
assert.equal(store.games().length, 0);

const errorStore = createLibraryStore(backend);
await Promise.all([errorStore.downloadGame(game.id), errorStore.downloadGame(game.id)]);
assert.match(errorStore.downloadError()?.message || '', /runtime nativo Tauri/);
assert.equal(errorStore.downloadingIds().size, 0);
await assert.rejects(() => store.loadGames(), /runtime nativo Tauri/);
assert.equal(store.isLoading(), false);
await store.loadGames([installed]);
const firstRefresh = store.loadGames();
assert.equal(store.loadGames(), firstRefresh);
assert.equal(store.isLoading(), true);
await assert.rejects(() => firstRefresh, /runtime nativo Tauri/);
assert.deepEqual(store.games(), [installed]);
assert.ok(store.loadError());
await store.loadGames([second]);
assert.equal(store.loadError(), '');
assert.equal(store.isLoading(), false);
const superseded = store.loadGames();
const supersededFailure = assert.rejects(() => superseded, /runtime nativo Tauri/);
await store.loadGames([second]);
await supersededFailure;
assert.deepEqual(store.games(), [second]);
assert.equal(store.loadError(), '');
const sourceStore = createLibraryStore(backend);
await sourceStore.openSources(game);
assert.equal(sourceStore.sourceOptions().length, 0);
assert.equal(sourceStore.sourcesLoading(), false);
assert.match(sourceStore.sourcesError(), /runtime nativo Tauri/);
sourceStore.closeSources();
assert.equal(sourceStore.sourceGame(), null);

const canonicalBackend = new TauriBackendService();
let canonicalRequests = 0;
let requestedDownload: [string, string | undefined] | undefined;
canonicalBackend.getDownloadJobs = async () => [];
canonicalBackend.getCanonicalGameOptions = async canonicalId => {
  canonicalRequests++;
  assert.equal(canonicalId, 'canonical-one');
  return {
    game: { ...game, canonicalId, canonicalTitle: 'Canonical one' },
    releases: [
      { id: 'release-one', catalogGameId: 'variant-one', title: 'USA', installed: false, sourceCount: 1, downloadableSourceCount: 1 },
      { id: 'release-two', catalogGameId: 'variant-two', title: 'Europe', installed: false, sourceCount: 1, downloadableSourceCount: 1 },
    ],
    sources: [
      { id: 'source-one', gameId: 'variant-one', name: 'Original one', sourceType: 'http', uri: 'https://example.invalid/one', available: true, access: 'http', downloadable: true },
      { id: 'source-two', gameId: 'variant-two', name: 'Original two', sourceType: 'http', uri: 'https://example.invalid/two', available: true, access: 'http', downloadable: true },
    ],
  };
};
canonicalBackend.downloadGame = async (gameId, sourceId) => {
  requestedDownload = [gameId, sourceId];
  throw new Error('download fixture');
};
const canonicalStore = createLibraryStore(canonicalBackend);
const canonicalGame = { ...game, id: 'variant-one', canonicalId: 'canonical-one', canonicalTitle: 'Canonical one' };
await canonicalStore.loadGames([canonicalGame, { ...canonicalGame, id: 'variant-two' }]);
await canonicalStore.openSources(canonicalGame);
assert.equal(canonicalRequests, 1);
assert.deepEqual(canonicalStore.sourceOptions().map(source => source.id), ['source-one']);
assert.equal(canonicalStore.sourceOptions()[0].name, 'Original one');
await canonicalStore.openSources({ ...canonicalGame, id: 'variant-two' });
assert.equal(canonicalStore.releaseIndex(), 1);
assert.deepEqual(canonicalStore.sourceOptions().map(source => source.id), ['source-two']);
canonicalStore.selectRelease(0);
canonicalStore.selectRelease(1);
assert.deepEqual(canonicalStore.sourceOptions().map(source => source.id), ['source-two']);
await canonicalStore.confirmSource();
assert.deepEqual(requestedDownload, ['variant-two', 'source-two']);

await canonicalStore.openSources(canonicalGame);
let searchedVersion = '';
let selectedResult: string[] = [];
canonicalBackend.searchJackett = async id => {
  searchedVersion = id;
  return [{ id: 'result-one', title: 'Fixture', tracker: 'Local', sizeBytes: 4, seeders: 1 }];
};
canonicalBackend.selectJackettResult = async (gameId, resultId) => {
  selectedResult = [gameId, resultId];
  return { id: 'source-one', gameId, name: 'Fixture', sourceType: 'magnet', uri: 'magnet:?xt=fixture', available: true };
};
const jackett = createRoot(dispose => ({ model: useJackettSearch(canonicalStore), dispose }));
try {
  await jackett.model.search();
  assert.equal(searchedVersion, 'variant-one');
  assert.equal(jackett.model.results().length, 1);
  assert.equal(jackett.model.open(), true);
  await jackett.model.select();
  assert.deepEqual(selectedResult, ['variant-one', 'result-one']);
  assert.equal(jackett.model.open(), false);
  assert.deepEqual(requestedDownload, ['variant-two', 'source-two'], 'Adding a search result must not start downloading');
  let complete!: (results: import('../solid/src/types/download.types').JackettResult[]) => void;
  canonicalBackend.searchJackett = () => new Promise(resolve => { complete = resolve; });
  const pending = jackett.model.search();
  canonicalStore.selectRelease(1);
  complete([{ id: 'stale', title: 'Stale', tracker: 'Local' }]);
  await pending;
  assert.deepEqual(jackett.model.results(), []);
} finally { jackett.dispose(); }
console.log('Library: real grouping and native errors, no fabricated downloads or sources.');
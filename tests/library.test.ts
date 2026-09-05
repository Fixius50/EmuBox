import assert from 'node:assert/strict';
import { MockBackendService } from '../solid/src/services/backend/mock-backend.service';
import { createLibraryStore } from '../solid/src/stores/library.store';
import type { Game } from '../solid/src/types/game.types';
import { shelfColumns } from '../solid/src/services/library/grid-layout';

assert.equal(shelfColumns(390), 1);
assert.equal(shelfColumns(900), 3);
assert.equal(shelfColumns(1920), 5);

const game: Game = {
  id: 'catalog-one', title: 'Catalog one', platform: 'ps2', platformName: 'PS2',
  releaseYear: 2000, genre: '', developer: '', publisher: '', rating: 0,
  playTimeMinutes: 0, favorite: false, coverImage: '', description: '', installed: false,
};
const second = { ...game, id: 'catalog-two', title: 'Catalog two' };
const installed = { ...game, installed: true, romPath: '/fixture/game.iso' };
const backend = new MockBackendService([installed, second]);
const store = createLibraryStore(backend);
await store.loadGames();
assert.equal(store.games().length, 2);
assert.equal(store.games().find(entry => entry.id === game.id)?.installed, true);
await store.loadGames([installed, installed]);
assert.equal(store.games().length, 1);
await store.loadGames([]);
assert.equal(store.games().length, 0);

class UnavailableSourceBackend extends MockBackendService {
  calls = 0;
  override async downloadGame(): Promise<never> {
    this.calls++;
    throw { code: 'NotFound', details: 'No hay fuente de descarga disponible' };
  }
}
const unavailable = new UnavailableSourceBackend([game]);
const errorStore = createLibraryStore(unavailable);
await Promise.all([errorStore.downloadGame(game.id), errorStore.downloadGame(game.id)]);
assert.equal(unavailable.calls, 1);
assert.equal(errorStore.downloadError()?.message, 'No hay fuente de descarga disponible');
assert.equal(errorStore.downloadingIds().size, 0);

const downloadBackend = new MockBackendService([game, second]);
const downloadStore = createLibraryStore(downloadBackend);
await downloadStore.loadGames();
await downloadStore.downloadGame(game.id);
assert.equal(downloadStore.downloadingIds().size, 0);
assert.equal(downloadStore.downloadError(), null);
assert.equal(downloadStore.games().length, 2);
assert.equal(downloadStore.games().find(entry => entry.id === game.id)?.installed, true);
console.log('Library totals, catalog reload, duplicate clicks and download errors: OK');

const sourceBackend = new MockBackendService([game]);
await sourceBackend.createDownloadSource({ id: 'source-choice', gameId: game.id, name: 'Fixture source',
  sourceType: 'http', uri: 'https://example.test/file.zip', available: true });
const sourceStore = createLibraryStore(sourceBackend);
await sourceStore.openSources(game);
assert.equal(sourceStore.sourceOptions().length, 1);
assert.equal(sourceStore.sourcesLoading(), false);
assert.equal((await sourceBackend.getDownloadJobs()).length, 0);
const chosenSource = sourceStore.sourceOptions()[0];
sourceStore.closeSources();
await sourceStore.downloadGame(game.id, chosenSource.id);
assert.equal(sourceStore.downloadError(), null);

let resolveSources: (sources: Awaited<ReturnType<MockBackendService['getDownloadSources']>>) => void = () => {};
class DelayedSources extends MockBackendService {
  override getDownloadSources() {
    return new Promise<Awaited<ReturnType<MockBackendService['getDownloadSources']>>>(resolve => { resolveSources = resolve; });
  }
}
const delayedStore = createLibraryStore(new DelayedSources());
const opening = delayedStore.openSources(game);
delayedStore.closeSources();
resolveSources([chosenSource]);
await opening;
assert.equal(delayedStore.sourceGame(), null);
assert.equal(delayedStore.sourceOptions().length, 0);
console.log('Source selection: no automatic download and stale responses ignored: OK');
import assert from 'node:assert/strict';
import { TauriBackendService } from '../solid/src/services/backend/tauri-backend.service';
import { createLibraryStore } from '../solid/src/stores/library.store';
import type { Game } from '../solid/src/types/game.types';

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
const sourceStore = createLibraryStore(backend);
await sourceStore.openSources(game);
assert.equal(sourceStore.sourceOptions().length, 0);
assert.equal(sourceStore.sourcesLoading(), false);
assert.match(sourceStore.sourcesError(), /runtime nativo Tauri/);
sourceStore.closeSources();
assert.equal(sourceStore.sourceGame(), null);
console.log('Library: real grouping and native errors, no fabricated downloads or sources.');
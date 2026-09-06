import assert from 'node:assert/strict';
import { catalogTitle, groupCatalog } from '../solid/src/services/library/catalog-groups';
import { createLibraryStore } from '../solid/src/stores/library.store';
import { MockBackendService } from '../solid/src/services/backend/mock-backend.service';
import type { Game } from '../solid/src/types/game.types';

const game: Game = { id: 'one', title: 'Fixture (2024) [Multi] (1.0) Repack FitGirl', platform: 'ps2', platformName: 'PS2',
  releaseYear: 2024, genre: '', developer: '', publisher: '', rating: 0, playTimeMinutes: 0,
  favorite: false, coverImage: '', description: '', installed: false };
const variants = [game, { ...game, id: 'two', title: 'Fixture (2024) [Multi] (1.0) Scene Skidrow' },
  { ...game, id: 'three', title: 'Fixture Free Download', platform: 'ps3' as const },
  { ...game, id: 'four', title: 'Fixture Deluxe Edition' }, { ...game, id: 'five', title: 'Fixture 2' }];
assert.equal(catalogTitle(game.title), 'Fixture');
assert.equal(catalogTitle('Fixture (Build 1234)'), 'Fixture');
assert.equal(catalogTitle('Fixture - Build 1234 + Expansion DLC'), 'Fixture - Build 1234 + Expansion DLC');
assert.equal(catalogTitle('Fixture (Part 2)'), 'Fixture (Part 2)');
assert.equal(catalogTitle('Fixture (USA)'), 'Fixture (USA)');
assert.equal(catalogTitle('Fixture (Build 1234 + DLC)'), 'Fixture (Build 1234 + DLC)');
assert.equal(groupCatalog([{ ...game, title: 'Fixture (1993)' }, { ...game, id: 'remake', title: 'Fixture (2016)' }]).length, 2);
assert.equal(groupCatalog(variants).length, 4);
assert.equal(groupCatalog([...variants].reverse()).find(entry => entry.title === 'Fixture' && entry.platform === 'ps2')?.id, 'one');
assert.equal(groupCatalog([game, { ...variants[1], installed: true, romPath: '/fixture/two.iso' }])[0].id, 'two');

class Backend extends MockBackendService {
  queried: string[] = [];
  downloaded: string[] = [];
  override async downloadGame(gameId: string, sourceId?: string) {
    this.downloaded = [gameId, sourceId || ''];
    this.setGames(variants.map(entry => entry.id === gameId ? { ...entry, installed: true } : entry));
    return { id: 'download-fixture', gameId, sourceId: sourceId || '', platform: 'ps2', destinationPath: '/fixture/two.iso',
      status: 'completed' as const, progress: 1, downloadedBytes: 1, speedBytesPerSecond: 0 };
  }
  override async getDownloadSources(gameId: string) {
    this.queried.push(gameId);
    return [{ id: `source-${gameId}`, gameId, name: 'server name', uri: `https://example.test/${gameId}.zip`,
      sourceType: 'http' as const, available: true, downloadable: true, access: 'http' as const }];
  }
}
const backend = new Backend(variants);
const store = createLibraryStore(backend);
await store.loadGames();
assert.equal(store.games().length, 5);
assert.equal(store.catalogGames().length, 4);
await store.openSources(store.catalogGames()[0]);
assert.deepEqual(backend.queried, ['one', 'two']);
assert.deepEqual(store.sourceOptions().map(source => source.gameId), ['one', 'two']);
assert.equal(store.sourceOptions()[1].name, variants[1].title);
assert.ok((await backend.getDownloadJobs()).length === 0);
store.setSourceIndex(1);
await store.confirmSource();
assert.deepEqual(backend.downloaded, ['two', 'source-two']);
assert.equal(store.catalogGames()[0].id, 'two');
assert.equal(store.catalogGames()[0].installed, true);
assert.equal(store.catalogGames().length, 4);
console.log('Catalog groups: packages combined, platforms/editions preserved, original download IDs retained');
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
assert.equal(catalogTitle('Fixture - Build 1234 + Expansion DLC'), 'Fixture');
assert.equal(catalogTitle('Fixture (Part 2)'), 'Fixture (Part 2)');
assert.equal(catalogTitle('Fixture (USA)'), 'Fixture (USA)');
assert.equal(catalogTitle('Fixture (Build 1234 + DLC)'), 'Fixture (Build 1234 + DLC)');
assert.equal(groupCatalog([{ ...game, title: 'Fixture (1993)' }, { ...game, id: 'remake', title: 'Fixture (2016)' }]).length, 2);
const screenshotTitles = [
  '#DRIVE Rally',
  '#DRIVE Rally (2024) [Ru/Multi] (1.0.0.0) Repack FitGirl',
  '#DRIVE Rally (2024) [Ru/Multi] (1.0.0.0) Repack seleZen',
  '#DRIVE Rally (2024) [Ru/Multi] (1.0.0.3) License GOG',
  "#DRIVE Rally (2024) [Ru/Multi] (1.1.1.0) Repack Let'sРlay",
  '#DRIVE Rally (2024) [Ru/Multi] (1.3.21.0) License GOG',
  '#DRIVE Rally (2024) [Ru/Multi] (1.3.24.0) License GOG',
  '#DRIVE Rally [GOG]',
];
const screenshotPackages = screenshotTitles.map((title, index) => ({ ...game, id: `drive-${index}`, platform: 'pc' as const, title, releaseYear: 0 }));
const driveGroups = groupCatalog(screenshotPackages);
assert.equal(driveGroups.length, 1);
assert.equal(driveGroups[0].title, '#DRIVE Rally');
assert.equal(driveGroups[0].variants.length, 8);
assert.deepEqual(new Set(driveGroups[0].variants.map(entry => entry.title)), new Set(screenshotTitles));
const bludTitles = ['#BLUD (2024) [Multi] (1.0) Repack FitGirl', '#BLUD (2024) [Multi] (1.0) Scene Skidrow',
  '#BLUD (v21.10.2024 | Build 16031108)', '#BLUD + DLC (Build 16031108) [Pre-Instalado]',
  '#BLUD Free Download', '#BLUD [P2P]', '#BLUD – Build 16031108 + Claws for Alarm DLC', '#Blud'];
assert.equal(groupCatalog(bludTitles.map((title, index) => ({ ...game, id: `blud-${index}`, title, releaseYear: index === 4 ? 2024 : 0 }))).length, 1);
assert.equal(groupCatalog([...screenshotPackages, { ...screenshotPackages[0], id: 'other-console', platform: 'ps3' }]).length, 2);
assert.equal(groupCatalog([{ ...game, title: 'Fixture (1993)' }, { ...game, id: 'remake', title: 'Fixture (2016)' },
  { ...game, id: 'unknown-year', title: 'Fixture', releaseYear: 0 }]).length, 3);
assert.equal(catalogTitle('Fixture Claws for Alarm'), 'Fixture Claws for Alarm');
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

const screenshotBackend = new Backend(screenshotPackages);
const screenshotStore = createLibraryStore(screenshotBackend);
await screenshotStore.loadGames();
assert.equal(screenshotStore.catalogGames().length, 1);
await screenshotStore.openSources(screenshotStore.catalogGames()[0]);
assert.equal(screenshotStore.sourceOptions().length, 8);
assert.deepEqual(new Set(screenshotBackend.queried), new Set(screenshotPackages.map(entry => entry.id)));
assert.deepEqual(new Set(screenshotStore.sourceOptions().map(source => source.name)), new Set(screenshotTitles));
screenshotStore.setSourceIndex(7);
await screenshotStore.confirmSource();
assert.deepEqual(screenshotBackend.downloaded, ['drive-7', 'source-drive-7']);
console.log('Catalog groups: packages combined, platforms/editions preserved, original download IDs retained');
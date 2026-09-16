import assert from 'node:assert/strict';
import { catalogTitle, groupCatalog } from '../solid/src/services/library/catalog-groups';
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
const canonical = groupCatalog([
  { ...game, id: 'mario-package-a', title: '#Mario Bros. (USA)', canonicalId: 'libretro-mario', canonicalTitle: 'Mario Bros.' },
  { ...game, id: 'mario-package-b', title: '^^Mario Bros. [Rev A]', canonicalId: 'libretro-mario', canonicalTitle: 'Mario Bros.', installed: true, romPath: '/fixture/mario.nes' },
]);
assert.equal(canonical.length, 1);
assert.equal(canonical[0].id, 'mario-package-b');
assert.equal(canonical[0].title, 'Mario Bros.');
assert.equal(canonical[0].variants.length, 2);
assert.equal(groupCatalog([
  { ...game, canonicalId: 'same-canonical', title: 'Fixture (1993)', releaseYear: 1993 },
  { ...game, id: 'second-release', canonicalId: 'same-canonical', title: 'Fixture (1994)', releaseYear: 1994 },
]).length, 1, 'A canonical identity must not split by package year');
assert.equal(groupCatalog([
  { ...game, id: 'same-a', title: 'Same title', canonicalId: 'canonical-a' },
  { ...game, id: 'same-b', title: 'Same title', canonicalId: 'canonical-b' },
]).length, 2);

console.log('Catalog groups: packages combined, platforms/editions preserved, original download IDs retained');
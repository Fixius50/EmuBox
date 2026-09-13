import assert from 'node:assert/strict';
import { moveXmb, xmbFolders, type XmbPosition } from '../solid/src/services/library/xmb-navigation';
import { groupCatalog } from '../solid/src/services/library/catalog-groups';
import type { Game } from '../solid/src/types/game.types';

const bounds = { categories: 5, rows: 120000, games: 3 };
let position: XmbPosition = { category: 1, row: 0, expanded: false, game: 0 };
position = moveXmb(position, 'enter', bounds);
assert.equal(position.row, 1);
position = moveXmb(position, 'enter', bounds);
assert.equal(position.expanded, true);
position = moveXmb(position, 'right', bounds);
assert.equal(position.game, 1);
assert.equal(position.category, 1);
position = moveXmb(position, 'back', bounds);
assert.equal(position.expanded, false);
assert.equal(position.row, 1);
position = moveXmb(position, 'down', bounds);
assert.equal(position.row, 2);
position = moveXmb(position, 'right', bounds);
assert.equal(position.category, 2);
assert.equal(position.row, 0);
assert.equal(moveXmb({ ...position, category: 0 }, 'left', bounds).category, 0);
assert.equal(moveXmb({ ...position, row: 120000 }, 'down', bounds).row, 120000);
assert.equal(moveXmb(position, 'enter', { ...bounds, rows: 0, games: 0 }).row, 0);
assert.equal(moveXmb({ ...position, row: 1, expanded: true }, 'left', bounds).expanded, false);
const fixtures = [
	{ id: 'ps1-one', title: 'Example', platform: 'ps1' },
	{ id: 'ps2-one', title: 'EXAMPLE', platform: 'ps2' },
	{ id: 'ps2-two', title: 'Example 2', platform: 'ps2' },
].map(entry => ({ ...entry, installed: false, favorite: false, releaseYear: 2000 } as Game));
const folders = xmbFolders(groupCatalog(fixtures));
assert.equal(folders.length, 2);
assert.equal(folders[0].games.length, 2);
assert.deepEqual(folders[0].games.flatMap(game => game.variants.map(variant => variant.id)), ['ps1-one', 'ps2-one']);
assert.equal(folders[1].title, 'Example 2');
assert.deepEqual(xmbFolders([]), []);
console.log('XMB: categories, folders, horizontal titles, back and empty/large catalog navigation: OK');
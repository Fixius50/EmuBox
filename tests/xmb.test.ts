import assert from "node:assert/strict";
import test from "node:test";
import {
  moveXmb,
  xmbFolders,
  type XmbPosition,
} from "../solid/src/services/library/xmb-navigation";
import { groupCatalog } from "../solid/src/services/library/catalog-groups";
import type { Game } from "../solid/src/types/game.types";
import { createRoot, createSignal } from "solid-js";
import { useXmbLibrary } from "../solid/src/hooks/useXmbLibrary";
import type { InputAction } from "../solid/src/types/input.types";
import { SETTINGS_TABS } from "@contracts/settings.types";

test("XMB: movement respects category, row and game bounds", () => {
const bounds = { categories: 5, rows: 120000, games: 3 };
let position: XmbPosition = { category: 1, row: 0, expanded: false, game: 0 };
position = moveXmb(position, "enter", bounds);
assert.equal(position.row, 1);
position = moveXmb(position, "enter", bounds);
assert.equal(position.expanded, true);
position = moveXmb(position, "right", bounds);
assert.equal(position.game, 1);
assert.equal(position.category, 1);
position = moveXmb(position, "back", bounds);
assert.equal(position.expanded, false);
assert.equal(position.row, 1);
position = moveXmb(position, "down", bounds);
assert.equal(position.row, 2);
position = moveXmb(position, "right", bounds);
assert.equal(position.category, 2);
assert.equal(position.row, 0);
assert.equal(moveXmb({ ...position, category: 0 }, "left", bounds).category, 0);
assert.equal(moveXmb({ ...position, row: 120000 }, "down", bounds).row, 120000);
assert.equal(
  moveXmb(position, "enter", { ...bounds, rows: 0, games: 0 }).row,
  0,
);
assert.equal(
  moveXmb({ ...position, row: 1, expanded: true }, "left", bounds).expanded,
  false,
);
});

const fixtures = [
  { id: "ps1-one", title: "Example", platform: "ps1" },
  { id: "ps2-one", title: "EXAMPLE", platform: "ps2" },
  { id: "ps2-two", title: "Example 2", platform: "ps2" },
].map(
  (entry) =>
    ({
      ...entry,
      installed: false,
      favorite: false,
      releaseYear: 2000,
    }) as Game,
);
test("XMB: folders preserve platforms and original variants", () => {
const folders = xmbFolders(groupCatalog(fixtures));
assert.equal(folders.length, 2);
assert.equal(folders[0].games.length, 2);
assert.deepEqual(
  folders[0].games.flatMap((game) =>
    game.variants.map((variant) => variant.id),
  ),
  ["ps1-one", "ps2-one"],
);
assert.equal(folders[1].title, "Example 2");
assert.deepEqual(xmbFolders([]), []);
});

test("XMB: canonical versions are built lazily and cached", () => {
const canonicalFolders = xmbFolders(groupCatalog([
  { ...fixtures[0], id: 'edition-us', canonicalId: 'canonical-example', canonicalTitle: 'Example', releaseTitle: 'Example (USA)' },
  { ...fixtures[0], id: 'edition-eu', canonicalId: 'canonical-example', canonicalTitle: 'Example', releaseTitle: 'Example (Europe)', installed: true },
]));
assert.equal(canonicalFolders.length, 1);
assert.equal(canonicalFolders[0].title, 'Example');
assert.equal(canonicalFolders[0].games.length, 2);
assert.deepEqual(canonicalFolders[0].games.map(game => game.title), ['Example (Europe)', 'Example (USA)']);
assert.deepEqual(canonicalFolders[0].games.map(game => game.id), ['edition-eu', 'edition-us']);
assert.equal(canonicalFolders[0].games[0].installed, true);
let versionReads = 0;
const lazyFolder = xmbFolders([{
  ...groupCatalog(fixtures)[0], canonicalId: 'lazy-canonical',
  get variants() { versionReads++; return [fixtures[0]]; },
}]);
assert.equal(versionReads, 0, 'Creating folders must not build every version card');
const lazyVersions = lazyFolder[0].games;
const afterFirstAccess = versionReads;
assert.equal(lazyFolder[0].games, lazyVersions);
assert.equal(versionReads, afterFirstAccess, 'Opening the same folder reuses version cards');
});

test("XMB: controller handles search, settings, empty catalogs and cleanup", () => {
let controller: ((action: InputAction) => void) | null = null;
const opened: string[] = [];
const runtime = createRoot((dispose) => {
  const [games, setGames] = createSignal(groupCatalog(fixtures));
  const model = useXmbLibrary({
    get games() {
      return games();
    },
    platforms: [],
    loading: false,
    downloadingIds: new Set(),
    inputStatus: {
      isConnected: true,
      deviceName: "Fixture",
      source: "keyboard",
    },
    onOpenGame: (game) => opened.push(game.id),
    onOpenSettings: (tab) => opened.push(tab),
    onMaintenance: () => opened.push("maintenance"),
    onFavorite: (id) => opened.push(id),
    onMove: () => {},
    onControllerReady: (handler) => {
      controller = handler;
    },
  });
  return { model, setGames, dispose };
});
const dispatch = (action: InputAction) => {
  assert.ok(controller, "Controller must be registered while mounted");
  controller(action);
};
try {
  const { model } = runtime;
  assert.equal(model.rows(), 2);
  const grouped = model.folders();
  dispatch("NAV_DOWN");
  dispatch("BUTTON_A");
  assert.equal(model.position().expanded, true);
  assert.equal(model.folders(), grouped, "Moving must not regroup the catalog");
  dispatch("NAV_RIGHT");
  assert.equal(model.selectedGame()?.id, "ps2-one");
  dispatch("BUTTON_A");
  assert.deepEqual(opened, ["ps2-one"]);
  assert.ok(model.rowWindow().length <= 7);
  model.search("missing");
  assert.equal(model.rows(), 0);
  assert.equal(model.position().expanded, false);
  dispatch("BUTTON_A");
  assert.equal(model.position().row, 0);
  model.search("example 2");
  assert.equal(model.rows(), 1);
  model.chooseRow(1);
  runtime.setGames([]);
  assert.equal(model.position().row, 0);
  model.chooseCategory(0);
  assert.equal(model.rows(), SETTINGS_TABS.length + 1);
  for (const [index, tab] of SETTINGS_TABS.entries()) {
    model.chooseRow(index + 1);
    dispatch("BUTTON_A");
    assert.equal(opened.at(-1), tab.id);
  }
  model.chooseRow(SETTINGS_TABS.length + 1);
  dispatch("BUTTON_A");
  assert.equal(opened.at(-1), "maintenance");
} finally {
  runtime.dispose();
}
assert.equal(controller, null, "Controller must be released on unmount");
});

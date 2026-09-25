import assert from "node:assert/strict";
import test from "node:test";
import {
  moveXmb,
  xmbDownloadStatus,
  xmbFolders,
  type XmbPosition,
} from "../solid/src/services/library/xmb-navigation";
import { groupCatalog } from "../solid/src/services/library/catalog-groups";
import type { Game } from "../solid/src/types/game.types";
import type { DownloadJob } from "../solid/src/types/download.types";
import { createRoot, createSignal } from "solid-js";
import { useXmbLibrary } from "../solid/src/hooks/useXmbLibrary";
import type { InputAction } from "../solid/src/types/input.types";

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
  assert.equal(
    moveXmb({ ...position, category: 0 }, "left", bounds).category,
    0,
  );
  assert.equal(
    moveXmb({ ...position, row: 120000 }, "down", bounds).row,
    120000,
  );
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
test("XMB: download label follows progress and preparation phases without scanning games", () => {
  const job: DownloadJob = {
    id: 'job-one', gameId: fixtures[0].id, sourceId: 'source-one', platform: 'ps1',
    destinationPath: '/fixture', status: 'queued', progress: 0,
    downloadedBytes: 0, speedBytesPerSecond: 0,
  };
  assert.equal(xmbDownloadStatus(fixtures[0], [job], new Set()), 'En cola 0 %');
  job.status = 'downloading';
  job.progress = 0.42;
  assert.equal(xmbDownloadStatus(fixtures[0], [job], new Set()), 'Descargando 42 %');
  job.phase = 'verifying';
  job.progress = 1;
  assert.equal(xmbDownloadStatus(fixtures[0], [job], new Set()), 'Verificando 100 %');
  job.phase = 'preparing';
  assert.equal(xmbDownloadStatus(fixtures[0], [job], new Set()), 'Preparando 100 %');
  assert.equal(xmbDownloadStatus(fixtures[1], [job], new Set([fixtures[1].id])), 'Iniciando descarga');
  assert.equal(xmbDownloadStatus({ ...fixtures[1], installed: true }, [], new Set()), 'Instalada');
});
test("XMB: folders preserve platforms and original variants", () => {
  const folders = xmbFolders(groupCatalog(fixtures));
  assert.equal(folders.length, 1);
  assert.equal(folders[0].games.length, 3);
  assert.deepEqual(
    folders[0].games.flatMap((game) =>
      game.variants.map((variant) => variant.id),
    ),
    ["ps1-one", "ps2-one", "ps2-two"],
  );
  assert.equal(folders[0].title, "Example");
  assert.equal(folders[0].games[2].title, "Example 2");
  assert.equal(xmbFolders(groupCatalog([fixtures[2]])).at(0)?.title, "Example 2");
  assert.deepEqual(xmbFolders([]), []);
});

test("XMB: unrelated canonical identities remain separate", () => {
  const folders = xmbFolders(groupCatalog([
    { ...fixtures[0], canonicalId: "first" },
    { ...fixtures[0], id: "another", canonicalId: "second" },
  ]));
  assert.equal(folders.length, 2);
});

test("XMB: multiple sequels share a folder without a base game", () => {
  const folders = xmbFolders(groupCatalog([
    { ...fixtures[0], title: "The Sims 2", canonicalId: "sims-2" },
    { ...fixtures[0], id: "four", title: "The Sims 4 Free Download", canonicalId: "sims-4", canonicalTitle: "The Sims 4 Free Download" },
    { ...fixtures[0], id: "edition", title: "The Sims 2: Complete Edition", canonicalId: "sims-2-edition" },
    { ...fixtures[0], id: "alone", title: "Portal 2", canonicalId: "portal-2" },
  ]));
  assert.equal(folders.length, 2);
  assert.equal(folders[0].title, "The Sims");
  assert.deepEqual(folders[0].games.map(game => game.title), ["The Sims 2", "The Sims 4", "The Sims 2: Complete Edition"]);
  assert.equal(folders[1].title, "Portal 2");
});

test("XMB: subtitled games share their franchise without merging releases", () => {
  const folders = xmbFolders(groupCatalog([
    { ...fixtures[0], title: "The Legend of Zelda: Breath of the Wild", canonicalId: "zelda-botw" },
    { ...fixtures[0], id: "tears", title: "The Legend of Zelda: Tears of the Kingdom", canonicalId: "zelda-totk" },
  ]));
  assert.equal(folders.length, 1);
  assert.equal(folders[0].title, "The Legend of Zelda");
  assert.deepEqual(folders[0].games.map(game => game.canonicalId), ["zelda-botw", "zelda-totk"]);
});

test("XMB: 007 releases and package variants share one horizontal folder", () => {
  const folders = xmbFolders(groupCatalog([
    { ...fixtures[0], id: "bond-base", title: "007 First Light", platform: "pc", matchMethod: "local-title", canonicalId: "bond-base" },
    { ...fixtures[0], id: "bond-update", title: "007 First Light (v1.1.0) [Pre-Instalado]", platform: "pc", matchMethod: "local-title", canonicalId: "bond-update" },
    { ...fixtures[0], id: "bond-ps1", title: "007 - Tomorrow Never Dies [SLUS-00975] [Vector] [RUS]", canonicalId: "bond-ps1" },
    { ...fixtures[0], id: "bond-dotted", title: "007.Legends-PLAZA", platform: "pc", canonicalId: "bond-dotted" },
    { ...fixtures[0], id: "other", title: "008 First Light", canonicalId: "other" },
  ]));
  assert.equal(folders.length, 2);
  assert.equal(folders[0].title, "007");
  assert.deepEqual(folders[0].games.map(game => game.id), ["bond-base", "bond-update", "bond-ps1", "bond-dotted"]);
  assert.equal(folders[1].title, "008 First Light");
});

test("XMB: local package suffixes share one folder without changing catalog IDs", () => {
  const titles = ["1 Trait Escape", "1 Trait Escape Free Download (Build 17471495)",
    "1 Trait Escape Free Download (v1.15)", "1 Trait Escape [P2P]",
    "1 Trait Escape- Free Download (TENOKE)", "1 Trait Escape-TENOKE"];
  const folders = xmbFolders(groupCatalog(titles.map((title, index) => ({
    ...fixtures[0], id: `trait-${index}`, platform: "pc" as const, title,
    canonicalId: `local-${index}`, canonicalTitle: title, matchMethod: "local-title",
  }))));
  assert.equal(folders.length, 1);
  assert.equal(folders[0].title, "1 Trait Escape");
  assert.deepEqual(folders[0].games.map(game => game.id), titles.map((_, index) => `trait-${index}`));
});

test("XMB: wheel navigates vertical folders and horizontal categories/versions", () => {
  const model = createRoot((dispose) => ({
    dispose,
    library: useXmbLibrary({
      games: groupCatalog([
        { ...fixtures[0], id: "edition-a", canonicalId: "fixture" },
        { ...fixtures[0], id: "edition-b", canonicalId: "fixture" },
        { ...fixtures[0], id: "other", title: "Other" },
      ]),
      platforms: [], loading: false, downloadingIds: new Set(), downloadJobs: [],
      inputStatus: { isConnected: false, deviceName: "", source: "keyboard" },
      onOpenGame: () => {}, onOpenSettings: () => {}, onFavorite: () => {}, onMove: () => {},
      onControllerReady: () => {},
    }),
  }));
  try {
    const { library } = model;
    assert.deepEqual(library.categories().slice(0, 4).map(category => category.id),
      ["settings", "favorites", "installed", "all"]);
    assert.equal(library.category().id, "all");
    assert.equal(library.scrollWheel("folders", 20, 0), true);
    assert.equal(library.position().row, 0);
    library.scrollWheel("folders", 40, 0);
    assert.equal(library.position().row, 1);
    library.scrollWheel("folders", 100, 0);
    assert.equal(library.position().row, 2);
    library.scrollWheel("folders", -4, 1);
    assert.equal(library.position().row, 1);
    library.scrollWheel("categories", 100, 0);
    assert.equal(library.position().category, 4);
    library.scrollWheel("categories", -100, 0);
    assert.equal(library.position().category, 3);
    library.navigate("enter");
    library.navigate("enter");
    assert.equal(library.position().expanded, true);
    library.scrollWheel("versions", 100, 0);
    assert.equal(library.position().game, 1);
    library.scrollWheel("versions", -100, 0);
    assert.equal(library.position().game, 0);
    assert.equal(library.scrollWheel("versions", 0, 0), false);
  } finally {
    model.dispose();
  }
});

test("XMB: canonical versions are built lazily and cached", () => {
  const canonicalFolders = xmbFolders(
    groupCatalog([
      {
        ...fixtures[0],
        id: "edition-us",
        canonicalId: "canonical-example",
        canonicalTitle: "Example",
        releaseTitle: "Example (USA)",
        coverImage: "https://example.invalid/example.png",
        description: "Descripcion disponible",
        genre: "Aventura",
      },
      {
        ...fixtures[0],
        id: "edition-eu",
        canonicalId: "canonical-example",
        canonicalTitle: "Example",
        releaseTitle: "Example (Europe)",
        installed: true,
      },
    ]),
  );
  assert.equal(canonicalFolders.length, 1);
  assert.equal(canonicalFolders[0].title, "Example");
  assert.equal(canonicalFolders[0].games.length, 2);
  assert.deepEqual(
    canonicalFolders[0].games.map((game) => game.title),
    ["Example (Europe)", "Example (USA)"],
  );
  assert.deepEqual(
    canonicalFolders[0].games.map((game) => game.id),
    ["edition-eu", "edition-us"],
  );
  assert.equal(canonicalFolders[0].games[0].installed, true);
  assert.equal(canonicalFolders[0].games[0].coverImage, "https://example.invalid/example.png");
  assert.equal(canonicalFolders[0].games[0].description, "Descripcion disponible");
  assert.equal(canonicalFolders[0].games[0].genre, "Aventura");
  let versionReads = 0;
  const lazyFolder = xmbFolders([
    {
      ...groupCatalog(fixtures)[0],
      canonicalId: "lazy-canonical",
      get variants() {
        versionReads++;
        return [fixtures[0]];
      },
    },
  ]);
  assert.equal(
    versionReads,
    0,
    "Creating folders must not build every version card",
  );
  const lazyVersions = lazyFolder[0].games;
  const afterFirstAccess = versionReads;
  assert.equal(lazyFolder[0].games, lazyVersions);
  assert.equal(
    versionReads,
    afterFirstAccess,
    "Opening the same folder reuses version cards",
  );
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
      downloadJobs: [],
      inputStatus: {
        isConnected: true,
        deviceName: "Fixture",
        source: "keyboard",
      },
      onOpenGame: (game) => opened.push(game.id),
      onOpenSettings: (tab) => opened.push(tab),
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
    assert.equal(model.rows(), 1);
    const grouped = model.folders();
    dispatch("NAV_DOWN");
    dispatch("BUTTON_A");
    assert.equal(model.position().expanded, true);
    assert.equal(
      model.folders(),
      grouped,
      "Moving must not regroup the catalog",
    );
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
    assert.equal(model.rows(), 1);
    assert.equal(opened.at(-1), "system");
    dispatch("NAV_DOWN");
    assert.equal(opened.at(-1), "system");
    assert.equal(model.position().row, 0);
    dispatch("BUTTON_A");
    assert.equal(opened.at(-1), "system");
    assert.ok(!opened.includes("maintenance"));
    model.search("");
    runtime.setGames(groupCatalog([
      ...fixtures,
      { ...fixtures[0], id: "gba-one", platform: "gba" },
      { ...fixtures[0], id: "snes-one", platform: "snes" },
    ]));
    model.chooseCategory(1);
    assert.deepEqual(
      model.categories().filter((entry) => entry.kind === "platform").map((entry) => entry.title),
      ["gba", "ps1", "ps2", "snes"],
    );
  } finally {
    runtime.dispose();
  }
  assert.equal(controller, null, "Controller must be released on unmount");
});

test("XMB: virtual keyboard opens only without a keyboard input source", () => {
  let controller: ((action: InputAction) => void) | null = null;
  const runtime = createRoot((dispose) => {
    const model = useXmbLibrary({
      games: groupCatalog(fixtures), platforms: [], loading: false, downloadingIds: new Set(), downloadJobs: [],
      inputStatus: { isConnected: true, deviceName: "Fixture pad", source: "gamepad" },
      onOpenGame: () => {}, onOpenSettings: () => {}, onFavorite: () => {}, onMove: () => {},
      onControllerReady: handler => { controller = handler; },
    });
    return { model, dispose };
  });
  try {
    controller?.("BUTTON_Y");
    assert.equal(runtime.model.virtualKeyboardOpen(), true);
    controller?.("BUTTON_A");
    assert.equal(runtime.model.query(), "A");
    controller?.("BUTTON_Y");
    assert.equal(runtime.model.query(), "A ");
    controller?.("BUTTON_X");
    assert.equal(runtime.model.query(), "A");
    controller?.("BUTTON_B");
    assert.equal(runtime.model.virtualKeyboardOpen(), false);
  } finally { runtime.dispose(); }
  assert.equal(controller, null);
});

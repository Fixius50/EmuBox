import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import { createRoot } from "solid-js";
import { useAudioDevices } from "@hooks/useAudioDevices";
import { DevelopmentBackendService } from "@services/backend/development-backend.service";

test("Preview: sample data is isolated, local and has no download sources", async () => {
  const backend = new DevelopmentBackendService();
  const games = await backend.getGames();
  assert.ok(games.length > 0);
  assert.ok(
    games.every((game) => game.id.startsWith("preview-") && !game.romPath),
  );
  games[0].title = "Changed copy";
  assert.notEqual((await backend.getGames())[0].title, games[0].title);
  assert.deepEqual(await backend.getDownloadSources(), []);
  assert.deepEqual(await backend.getDownloadJobs(), []);
  assert.deepEqual(await backend.getStoreAccounts(), []);
});

test("Preview: edits and favorites exist only in the current instance", async () => {
  const backend = new DevelopmentBackendService();
  const fresh = new DevelopmentBackendService();
  const settings = await backend.getSettings();
  settings.audio.masterVolume = 15;
  await backend.saveSettings(settings);
  settings.audio.masterVolume = 2;
  assert.equal((await backend.getSettings()).audio.masterVolume, 15);
  assert.equal((await fresh.getSettings()).audio.masterVolume, 80);
  const [emulator] = await backend.getEmulators();
  await backend.saveEmulator({ ...emulator, name: "Edited preview" });
  assert.equal((await backend.getEmulators())[0].name, "Edited preview");
  assert.equal((await fresh.getEmulators())[0].name, emulator.name);
  await backend.deleteEmulator(emulator.id);
  assert.deepEqual(await backend.getEmulators(), []);
  const [game] = await backend.getGames();
  assert.equal(await backend.toggleFavorite(game.id), true);
  assert.equal((await fresh.getGames())[0].favorite, false);
});

test("Preview: native operations remain unavailable", async () => {
  const backend = new DevelopmentBackendService();
  assert.equal(backend.isTauriEnvironment, false);
  for (const operation of [
    () => backend.launchGame("preview-orbita-snes"),
    () => backend.downloadGame("preview-orbita-snes"),
    () => backend.getHardwareInfo(),
    () => backend.exitToLinuxShell(),
    () => backend.startSteamAuthorization(),
    () => backend.startEpicAuthorization(),
    () => backend.startGogAuthorization(),
    () => backend.searchJackett("preview-orbita-snes"),
    () => backend.scanGames(),
    () => backend.recordFrontendEvents([]),
  ])
    await assert.rejects(operation, /runtime nativo Tauri/);
});

test("Preview: entry remains development-only and native runtime takes precedence", () => {
  const app = readFileSync(
    new URL("../solid/src/App.tsx", import.meta.url),
    "utf8",
  );
  assert.match(
    app,
    /const BrowserFallback: Component = import\.meta\.env\.DEV\s*\? lazy\(\(\) => import\(['"]@components\/development\/DevelopmentPreview['"]\)\)/,
  );
  assert.match(app, /when=\{new TauriBackendService\(\)\.isTauriEnvironment\}/);
  assert.match(app, /<NativeApp\s*\/>/);
  assert.match(app, /Runtime nativo Tauri no disponible/);
});

test("Preview: audio uses the backend contract without capture and resets disconnected selections", async (context) => {
  const originalNavigator = Object.getOwnPropertyDescriptor(
    globalThis,
    "navigator",
  );
  const media = new EventTarget();
  let available = [
    { deviceId: "default", label: "System default", kind: "audiooutput" },
    {
      deviceId: "mic-fixture",
      label: "Microphone fixture",
      kind: "audioinput",
    },
    {
      deviceId: "speaker-fixture",
      label: "Speaker fixture",
      kind: "audiooutput",
    },
    { deviceId: "camera-fixture", label: "Camera fixture", kind: "videoinput" },
  ];
  Object.assign(media, { enumerateDevices: async () => available });
  Object.defineProperty(globalThis, "navigator", {
    configurable: true,
    value: { mediaDevices: media },
  });
  context.after(() => {
    if (originalNavigator)
      Object.defineProperty(globalThis, "navigator", originalNavigator);
    else Reflect.deleteProperty(globalThis, "navigator");
  });
  const backend = new DevelopmentBackendService();
  const info = await backend.getAudioInfo();
  assert.deepEqual(
    info.devices.map((device) => [device.id, device.type]),
    [
      ["mic-fixture", "source"],
      ["speaker-fixture", "sink"],
    ],
  );
  assert.equal(info.deviceNamesLimited, false);
  assert.equal(info.masterVolume, null);
  const settings = await backend.getSettings();
  settings.audio.inputDevice = "mic-fixture";
  settings.audio.outputDevice = "speaker-fixture";
  const runtime = createRoot((dispose) => ({
    model: useAudioDevices({
      backend,
      settings,
      isRowFocused: () => false,
      onUpdateSettings: (update) => update(settings),
    }),
    dispose,
  }));
  context.after(runtime.dispose);
  await runtime.model.refresh();
  assert.equal(runtime.model.devices().length, 2);
  assert.equal(settings.audio.inputDevice, "mic-fixture");
  available = [
    {
      deviceId: "speaker-fixture",
      label: "Speaker fixture",
      kind: "audiooutput",
    },
  ];
  await runtime.model.refresh();
  assert.equal(settings.audio.inputDevice, "default");
  assert.equal(settings.audio.outputDevice, "speaker-fixture");
  available = [{ deviceId: "", label: "", kind: "audioinput" }];
  await runtime.model.refresh();
  assert.equal(runtime.model.limited(), true);
  assert.equal(settings.audio.outputDevice, "default");
  Object.assign(media, {
    enumerateDevices: async () => {
      throw new Error("Detection failed");
    },
  });
  await runtime.model.refresh();
  assert.equal(runtime.model.error(), "Detection failed");
  assert.equal(runtime.model.loading(), false);
});

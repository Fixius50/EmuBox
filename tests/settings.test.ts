import assert from "node:assert/strict";
import test from "node:test";
import { createRoot, createSignal } from "solid-js";
import type { Emulator } from "@contracts/game.types";
import type { InputAction } from "@contracts/input.types";
import { useSettingsNavigation } from "@hooks/useSettingsNavigation";
import { useSettingsController } from "@hooks/useSettingsController";
import { useEmulatorCrud } from "@hooks/useEmulatorCrud";
import { useMaintenanceController } from "@hooks/useMaintenanceController";
import { createSystemStore } from "@stores/system.store";
import { TauriBackendService } from "@services/backend/tauri-backend.service";
import { SoundFxService } from "@services/audio/sound-fx.service";

function createEmulatorFixture(): Emulator {
  return {
    id: "fixture-emulator",
    name: "Fixture",
    supportedPlatforms: ["snes"],
    coreType: "libretro",
    executable: "/fixture/retroarch",
    arguments: [],
    version: "1.0",
    status: "active",
  };
}

test("Settings: tabs, volume and back navigation", () => {
  let settingsArea: "sidebar" | "content" = "sidebar";
  let settingsTab = "system";
  let settingsRow = 0;
  let volumeChange = 0;
  let wentBack = false;
  const settingsAction = useSettingsNavigation({
    soundFx: new SoundFxService(),
    emulatorCount: () => 0,
    activeSettingsTab: () => settingsTab,
    settingsFocusArea: () => settingsArea,
    settingsRowIndex: () => settingsRow,
    onSettingsTabChange: (tab) => {
      settingsTab = tab;
    },
    onSettingsFocusAreaChange: (area) => {
      settingsArea = area;
    },
    onSettingsRowIndexChange: (row) => {
      settingsRow = row;
    },
    onToggleCurrentSetting: () => {},
    onAdjustCurrentSlider: (delta) => {
      volumeChange += delta;
    },
    onBack: () => {
      wentBack = true;
    },
  });
  settingsAction("BUTTON_LB");
  assert.equal(settingsTab, "stores");
  settingsAction("BUTTON_RB");
  assert.equal(settingsTab, "system");
  settingsArea = "content";
  settingsRow = 0;
  settingsAction("NAV_DOWN");
  assert.equal(settingsRow, 0, "System has no editable performance preference");
  settingsArea = "sidebar";
  settingsAction("NAV_UP");
  assert.equal(wentBack, true, "Up from the first sidebar tab exits settings");
  wentBack = false;
  settingsTab = "audio";
  settingsAction("BUTTON_A");
  settingsAction("NAV_DOWN");
  settingsAction("NAV_DOWN");
  settingsAction("NAV_DOWN");
  settingsAction("NAV_RIGHT");
  assert.equal(volumeChange, 5);
  settingsAction("BUTTON_B");
  assert.equal(settingsArea, "sidebar");
  settingsAction("BUTTON_B");
  assert.equal(wentBack, true);
  wentBack = false;
  settingsArea = "content";
  settingsRow = 0;
  settingsAction("NAV_UP");
  assert.equal(
    wentBack,
    true,
    "Up from the first control returns to categories",
  );
});

test("Settings: emulator persistence, reload and rejected writes", async () => {
  const emulatorFixture = createEmulatorFixture();
  const emulatorBackend = new TauriBackendService();
  const emulatorStore = createSystemStore(emulatorBackend);
  let persistedEmulators: Emulator[] = [];
  let saveCalls = 0;
  let deleteCalls = 0;
  let saveSounds = 0;
  let deleteSounds = 0;
  const emulatorSound = new SoundFxService();
  emulatorSound.playFavorite = () => {
    saveSounds++;
  };
  emulatorSound.playBack = () => {
    deleteSounds++;
  };
  emulatorBackend.saveEmulator = async (emulator) => {
    saveCalls++;
    persistedEmulators = [
      ...persistedEmulators.filter((entry) => entry.id !== emulator.id),
      emulator,
    ];
  };
  emulatorBackend.deleteEmulator = async (id) => {
    deleteCalls++;
    persistedEmulators = persistedEmulators.filter((entry) => entry.id !== id);
  };
  emulatorBackend.getEmulators = async () => [...persistedEmulators];
  const emulatorSettings = useSettingsController({
    systemStore: emulatorStore,
    soundFx: emulatorSound,
    activeSettingsTab: () => "emulators",
    settingsRowIndex: () => 0,
  });
  const savingEmulator = emulatorSettings.handleSaveEmulator(emulatorFixture);
  assert.deepEqual(
    emulatorStore.emulators(),
    [],
    "State waits for persistence",
  );
  assert.equal(saveSounds, 0);
  await savingEmulator;
  assert.deepEqual(emulatorStore.emulators(), [emulatorFixture]);
  assert.equal(saveCalls, 1);
  assert.equal(saveSounds, 1);
  const editedEmulator = { ...emulatorFixture, name: "Edited fixture" };
  await emulatorSettings.handleSaveEmulator(editedEmulator);
  assert.equal(saveCalls, 2);
  assert.deepEqual(emulatorStore.emulators(), [editedEmulator]);
  emulatorStore.setEmulators([]);
  emulatorStore.setEmulators(await emulatorBackend.getEmulators());
  assert.deepEqual(
    emulatorStore.emulators(),
    [editedEmulator],
    "Saved edits survive a reload",
  );
  const confirmedEmulators = emulatorStore.emulators();
  emulatorBackend.saveEmulator = async () => {
    throw new Error("Save rejected");
  };
  await assert.rejects(
    () => emulatorSettings.handleSaveEmulator(emulatorFixture),
    /Save rejected/,
  );
  assert.equal(emulatorStore.emulators(), confirmedEmulators);
  assert.equal(saveSounds, 2);
  const removeEmulator = emulatorBackend.deleteEmulator;
  emulatorBackend.deleteEmulator = async () => {
    throw new Error("Delete rejected");
  };
  await assert.rejects(
    () => emulatorSettings.handleDeleteEmulator(emulatorFixture.id),
    /Delete rejected/,
  );
  assert.equal(emulatorStore.emulators(), confirmedEmulators);
  assert.equal(deleteSounds, 0);
  emulatorBackend.deleteEmulator = removeEmulator;
  const deletingEmulator = emulatorSettings.handleDeleteEmulator(
    emulatorFixture.id,
  );
  assert.equal(emulatorStore.emulators(), confirmedEmulators);
  await deletingEmulator;
  assert.equal(deleteCalls, 1);
  assert.equal(deleteSounds, 1);
  assert.deepEqual(emulatorStore.emulators(), []);
  assert.deepEqual(await emulatorBackend.getEmulators(), []);
});

test("Settings: modal waits, rejects duplicate actions and allows retry", async (context) => {
  const emulatorFixture = createEmulatorFixture();
  let crudController: ((action: InputAction) => void) | null = null;
  let closeCalls = 0;
  let crudSaveCalls = 0;
  let crudDeleteCalls = 0;
  let finishSave!: () => void;
  let rejectSave!: (cause: unknown) => void;
  let rejectDelete!: (cause: unknown) => void;
  const crud = createRoot((dispose) => ({
    model: useEmulatorCrud({
      isOpen: () => true,
      initialData: () => emulatorFixture,
      onClose: () => {
        closeCalls++;
      },
      onSave: (emulator) => {
        assert.deepEqual(emulator, emulatorFixture);
        crudSaveCalls++;
        return new Promise<void>((resolve, reject) => {
          finishSave = resolve;
          rejectSave = reject;
        });
      },
      onDelete: (id) => {
        assert.equal(id, emulatorFixture.id);
        crudDeleteCalls++;
        return new Promise<void>((_resolve, reject) => {
          rejectDelete = reject;
        });
      },
      onControllerReady: (handler) => {
        crudController = handler;
      },
    }),
    dispose,
  }));
  context.after(() => {
    crud.dispose();
    assert.equal(crudController, null);
  });
  const pendingSave = crud.model.handleSave();
  assert.equal(crud.model.isPending(), true);
  crud.model.handleClose();
  crudController?.("BUTTON_B");
  await crud.model.handleSave();
  await crud.model.handleDelete();
  assert.equal(closeCalls, 0);
  assert.equal(crudSaveCalls, 1);
  assert.equal(crudDeleteCalls, 0);
  rejectSave({ details: "Persistence unavailable" });
  await pendingSave;
  assert.equal(crud.model.isPending(), false);
  assert.equal(crud.model.error(), "Persistence unavailable");
  assert.deepEqual(crud.model.formData(), emulatorFixture);
  assert.equal(closeCalls, 0);
  const pendingDelete = crud.model.handleDelete();
  assert.equal(crud.model.error(), "");
  assert.equal(crud.model.isPending(), true);
  rejectDelete(new Error("Delete unavailable"));
  await pendingDelete;
  assert.equal(crud.model.error(), "Delete unavailable");
  assert.equal(crud.model.isPending(), false);
  assert.equal(closeCalls, 0);
  const retrySave = crud.model.handleSave();
  assert.equal(crud.model.error(), "");
  finishSave();
  await retrySave;
  assert.equal(crudSaveCalls, 2);
  assert.equal(crud.model.isPending(), false);
  assert.equal(closeCalls, 1);
});

test("Settings: maintenance controller bounds and cleanup", (context) => {
  let controller: ((action: InputAction) => void) | null = null;
  const maintenance = createRoot((dispose) => {
    const [index, setIndex] = createSignal(0);
    const model = useMaintenanceController({
      backend: new TauriBackendService(),
      isOpen: () => true,
      focusedIndex: index,
      onSelectIndex: setIndex,
      onClose: () => {},
      onControllerReady: (handler) => {
        controller = handler;
      },
    });
    return { model, dispose };
  });
  context.after(() => {
    maintenance.dispose();
    assert.equal(controller, null);
  });
  for (let step = 0; step < 10; step++) controller?.("NAV_DOWN");
  assert.equal(
    maintenance.model.activeIndex(),
    maintenance.model.actions.length - 1,
  );
});

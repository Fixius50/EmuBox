import type { Emulator, PerformanceMode } from "@contracts/game.types";
import type {
  UseSettingsControllerOptions,
  UseSettingsControllerReturn,
} from "@contracts/settings.types";

const PERFORMANCE_MODES_LIST: readonly PerformanceMode[] = [
  "high-performance",
  "balanced",
  "power-saver",
  "ultra-boost",
] as const;

export function useSettingsController(
  options: UseSettingsControllerOptions,
): UseSettingsControllerReturn {
  const { systemStore, soundFx, activeSettingsTab, settingsRowIndex } = options;

  // Emulator CRUD Handlers
  const handleSaveEmulator = async (emulator: Emulator) => {
    await systemStore.saveEmulator(emulator);
    soundFx.playFavorite();
  };

  const handleDeleteEmulator = async (emulatorId: string) => {
    await systemStore.deleteEmulator(emulatorId);
    soundFx.playBack();
  };

  // Physical Gamepad Vibration Test
  const triggerVibrationTest = (padIndex: number) => {
    try {
      const pads =
        typeof navigator !== "undefined" && navigator.getGamepads
          ? navigator.getGamepads()
          : [];
      const pad = pads[padIndex];
      if (pad && (pad as any).vibrationActuator) {
        (pad as any).vibrationActuator.playEffect("dual-rumble", {
          startDelay: 0,
          duration: 300,
          weakMagnitude: 0.8,
          strongMagnitude: 0.8,
        });
      }
    } catch {
      // Ignored if browser does not support vibration
    }
  };

  // Main Action Toggle Handler with switch
  const handleToggleCurrentSetting = () => {
    const settings = systemStore.settings();
    if (!settings) return;

    const tab = activeSettingsTab();
    const row = settingsRowIndex();
    const clone = JSON.parse(JSON.stringify(settings));

    switch (tab) {
      case "system":
        switch (row) {
          case 0: {
            const current = clone.system?.performanceMode || "high-performance";
            const curIdx = PERFORMANCE_MODES_LIST.indexOf(
              current as PerformanceMode,
            );
            const nextMode =
              PERFORMANCE_MODES_LIST[
                (curIdx + 1) % PERFORMANCE_MODES_LIST.length
              ];
            if (!clone.system) clone.system = {};
            clone.system.performanceMode = nextMode;
            systemStore.updateSettings(clone);
            soundFx.playMove();
            break;
          }
          case 3:
            clone.display.vsync = !clone.display.vsync;
            systemStore.updateSettings(clone);
            soundFx.playSelect();
            break;
          default:
            break;
        }
        break;

      case "audio":
        if (row === 0) {
          clone.audio.uiSoundEffects = !clone.audio.uiSoundEffects;
          soundFx.setEnabled(clone.audio.uiSoundEffects);
          systemStore.updateSettings(clone);
          soundFx.playSelect();
        }
        break;

      case "gamepad":
        switch (row) {
          case 0:
            clone.gamepad.vibration = !clone.gamepad.vibration;
            systemStore.updateSettings(clone);
            soundFx.playSelect();
            break;
          default:
            if (row >= 2) {
              triggerVibrationTest(row - 2);
              soundFx.playSelect();
            }
            break;
        }
        break;

      default:
        break;
    }
  };

  // Slider adjustments with switch
  const handleAdjustCurrentSlider = (delta: number) => {
    const settings = systemStore.settings();
    if (!settings) return;

    const tab = activeSettingsTab();
    const row = settingsRowIndex();
    const clone = JSON.parse(JSON.stringify(settings));

    switch (tab) {
      case "audio":
        if (row === 1) {
          clone.audio.masterVolume = Math.min(
            100,
            Math.max(0, clone.audio.masterVolume + delta),
          );
          systemStore.updateSettings(clone);
          soundFx.playMove();
        }
        break;
      default:
        break;
    }
  };

  return {
    handleSaveEmulator,
    handleDeleteEmulator,
    handleToggleCurrentSetting,
    handleAdjustCurrentSlider,
    triggerVibrationTest,
  };
}

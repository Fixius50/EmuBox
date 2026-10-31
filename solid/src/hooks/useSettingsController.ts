import type { Emulator, SystemSettings } from "@contracts/game.types";
import type {
  UseSettingsControllerOptions,
  UseSettingsControllerReturn,
} from "@contracts/settings.types";
import type { SoundFxService } from "@services/audio/sound-fx.service";

type SettingToggleHandler = (
  settings: SystemSettings,
  context: { soundFx: SoundFxService; triggerVibrationTest: (padIndex: number) => void; rowIndex: number }
) => SystemSettings | void;

const SETTING_TOGGLES: Record<string, Record<number, SettingToggleHandler>> = {
  system: {
    0: (s, { soundFx }) => {
      soundFx.playSelect();
      return { ...s, display: { ...s.display, vsync: !s.display.vsync } };
    },
  },
  services: {
    0: (s, { soundFx }) => {
      soundFx.playSelect();
      return {
        ...s,
        system: { ...s.system, seedCompletedTorrents: !s.system?.seedCompletedTorrents },
      };
    },
    1: (s, { soundFx }) => {
      soundFx.playSelect();
      return {
        ...s,
        system: { ...s.system, autoInstallDownloads: !(s.system?.autoInstallDownloads ?? true) },
      };
    },
  },
  audio: {
    2: (s, { soundFx }) => {
      const next = !s.audio.uiSoundEffects;
      soundFx.setEnabled(next);
      soundFx.playSelect();
      return { ...s, audio: { ...s.audio, uiSoundEffects: next } };
    },
  },
  gamepad: {
    0: (s, { soundFx }) => {
      soundFx.playSelect();
      return { ...s, gamepad: { ...s.gamepad, vibration: !s.gamepad.vibration } };
    },
  },
};

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

  // Main Action Toggle Handler (Schema / Dictionary based)
  const handleToggleCurrentSetting = () => {
    const settings = systemStore.settings();
    if (!settings) return;

    const tab = activeSettingsTab();
    const row = settingsRowIndex();

    const handler = SETTING_TOGGLES[tab]?.[row];
    if (handler) {
      const updated = handler(settings, { soundFx, triggerVibrationTest, rowIndex: row });
      if (updated) systemStore.updateSettings(updated);
      return;
    }

    if (tab === "gamepad" && row >= 2) {
      triggerVibrationTest(row - 2);
      soundFx.playSelect();
    }
  };

  // Slider adjustments
  const handleAdjustCurrentSlider = (delta: number) => {
    const settings = systemStore.settings();
    if (!settings) return;

    const tab = activeSettingsTab();
    const row = settingsRowIndex();

    if (tab === "audio" && row === 3) {
      const current = settings.audio.masterVolume ?? 80;
      const nextVolume = Math.min(100, Math.max(0, current + delta));
      systemStore.updateSettings({
        ...settings,
        audio: { ...settings.audio, masterVolume: nextVolume },
      });
      soundFx.playMove();
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

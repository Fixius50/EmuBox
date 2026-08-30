import { createSignal } from 'solid-js';
import type { Emulator, PerformanceMode } from '@contracts/game.types';
import type { UpdateInfo, UpdateCheckResult, UpdateProgress } from '@contracts/update.types';
import type { UseSettingsControllerOptions, UseSettingsControllerReturn } from '@contracts/settings.types';

const PERFORMANCE_MODES_LIST: readonly PerformanceMode[] = [
  'high-performance',
  'balanced',
  'power-saver',
  'ultra-boost'
] as const;

export function useSettingsController(options: UseSettingsControllerOptions): UseSettingsControllerReturn {
  const { systemStore, soundFx, backend, activeSettingsTab, settingsRowIndex } = options;
  const [updateInfo, setUpdateInfo] = createSignal<UpdateInfo | undefined>(undefined);

  // OTA Handlers
  const handleCheckUpdates = async (): Promise<UpdateCheckResult | undefined> => {
    soundFx.playMove();
    try {
      await backend.executeCommand("git fetch origin main");
    } catch {
      // ignore in browser
    }
    const res = await backend.checkForUpdates();
    const info = await backend.getUpdateInfo();
    setUpdateInfo(info);
    soundFx.playSelect();
    return res;
  };

  const handleApplyUpdate = async (ver?: string): Promise<UpdateProgress | undefined> => {
    soundFx.playSelect();
    try {
      // 1. Ejecutar actualización real de Git y compilación en Arch Linux
      const output = await backend.executeCommand("bash /opt/emubox/scripts/update-emubox.sh");
      console.log('[EmuBox Update]', output);
    } catch (err) {
      console.error('[EmuBox Update Error]', err);
    }

    const progress = await backend.applyUpdate(ver);
    const info = await backend.getUpdateInfo();
    setUpdateInfo(info);
    soundFx.playFavorite();

    // 2. Reiniciar el sistema operativo completo tras actualizar
    setTimeout(async () => {
      try {
        await backend.executeCommand("sudo systemctl reboot || sudo reboot");
      } catch {
        await backend.restart();
      }
    }, 2000);

    return progress;
  };

  // Emulator CRUD Handlers
  const handleSaveEmulator = (emulator: Emulator) => {
    const emus = [...systemStore.emulators()];
    const existingIdx = emus.findIndex((e) => e.id === emulator.id);
    const updatedEmus = existingIdx >= 0
      ? emus.map((e, idx) => (idx === existingIdx ? emulator : e))
      : [...emus, emulator];

    systemStore.setEmulators(updatedEmus);
    soundFx.playFavorite();
  };

  const handleDeleteEmulator = (emulatorId: string) => {
    const emus = systemStore.emulators().filter((e) => e.id !== emulatorId);
    systemStore.setEmulators(emus);
    soundFx.playBack();
  };

  // Physical Gamepad Vibration Test
  const triggerVibrationTest = (padIndex: number) => {
    try {
      const pads = typeof navigator !== 'undefined' && navigator.getGamepads ? navigator.getGamepads() : [];
      const pad = pads[padIndex];
      if (pad && (pad as any).vibrationActuator) {
        (pad as any).vibrationActuator.playEffect('dual-rumble', {
          startDelay: 0,
          duration: 300,
          weakMagnitude: 0.8,
          strongMagnitude: 0.8
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
      case 'system':
        switch (row) {
          case 0: {
            const current = clone.system?.performanceMode || 'high-performance';
            const curIdx = PERFORMANCE_MODES_LIST.indexOf(current as PerformanceMode);
            const nextMode = PERFORMANCE_MODES_LIST[(curIdx + 1) % PERFORMANCE_MODES_LIST.length];
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

      case 'audio':
        if (row === 0) {
          clone.audio.uiSoundEffects = !clone.audio.uiSoundEffects;
          soundFx.setEnabled(clone.audio.uiSoundEffects);
          systemStore.updateSettings(clone);
          soundFx.playSelect();
        }
        break;

      case 'gamepad':
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

      case 'update':
        switch (row) {
          case 0:
            if (!clone.updates) clone.updates = { autoUpdate: true, channel: 'stable', checkOnStartup: true };
            clone.updates.autoUpdate = !clone.updates.autoUpdate;
            systemStore.updateSettings(clone);
            soundFx.playSelect();
            break;
          case 1:
            soundFx.playSelect();
            (window as any).__EMUBOX_TRIGGER_UPDATE_ACTION__?.();
            break;
          default:
            break;
        }
        break;

      case 'emulators':
        soundFx.playSelect();
        (window as any).__EMUBOX_OPEN_EMULATOR_CONFIG__?.(row);
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
      case 'audio':
        if (row === 1) {
          clone.audio.masterVolume = Math.min(100, Math.max(0, clone.audio.masterVolume + delta));
          systemStore.updateSettings(clone);
          soundFx.playMove();
        }
        break;
      default:
        break;
    }
  };

  return {
    updateInfo,
    setUpdateInfo,
    handleCheckUpdates,
    handleApplyUpdate,
    handleSaveEmulator,
    handleDeleteEmulator,
    handleToggleCurrentSetting,
    handleAdjustCurrentSlider,
    triggerVibrationTest
  };
}

import { createSignal } from 'solid-js';
import type { Platform, Emulator, SystemSettings } from '@contracts/game.types';
import type { IEmuBoxBackend } from '@contracts/backend.types';

export function createSystemStore(backend: IEmuBoxBackend) {
  const [platforms, setPlatforms] = createSignal<Platform[]>([]);
  const [emulators, setEmulators] = createSignal<Emulator[]>([]);
  const [settings, setSettings] = createSignal<SystemSettings | null>(null);
  const [isLoading, setIsLoading] = createSignal<boolean>(false);

  const loadSystemData = async () => {
    setIsLoading(true);
    try {
      await Promise.all([
        backend.getPlatforms().then(setPlatforms),
        backend.getEmulators().then(setEmulators),
        backend.getSettings().then(setSettings)
      ]);
    } finally {
      setIsLoading(false);
    }
  };

  const updateSettings = async (newSettings: SystemSettings) => {
    setSettings(newSettings);
    await backend.saveSettings(newSettings);
  };

  const saveEmulator = async (emulator: Emulator) => {
    await backend.saveEmulator(emulator);
    setEmulators(current => current.some(entry => entry.id === emulator.id)
      ? current.map(entry => entry.id === emulator.id ? emulator : entry)
      : [...current, emulator]);
  };

  const deleteEmulator = async (emulatorId: string) => {
    await backend.deleteEmulator(emulatorId);
    setEmulators(current => current.filter(entry => entry.id !== emulatorId));
  };

  return {
    platforms,
    setPlatforms,
    emulators,
    setEmulators,
    settings,
    setSettings,
    isLoading,
    loadSystemData,
    updateSettings,
    saveEmulator,
    deleteEmulator
  };
}

export type SystemStore = ReturnType<typeof createSystemStore>;

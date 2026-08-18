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
      const [plats, emus, sett] = await Promise.all([
        backend.getPlatforms(),
        backend.getEmulators(),
        backend.getSettings()
      ]);
      setPlatforms(plats);
      setEmulators(emus);
      setSettings(sett);
    } finally {
      setIsLoading(false);
    }
  };

  const updateSettings = async (newSettings: SystemSettings) => {
    setSettings(newSettings);
    await backend.saveSettings(newSettings);
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
    updateSettings
  };
}

export type SystemStore = ReturnType<typeof createSystemStore>;

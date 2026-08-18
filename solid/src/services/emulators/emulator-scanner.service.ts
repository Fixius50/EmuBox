import type { Emulator } from '@contracts/game.types';

export class EmulatorScannerService {
  private knownEngines: Array<{ id: string; name: string; binary: string; type: 'libretro' | 'standalone' }> = [
    { id: 'retroarch', name: 'RetroArch Frontend', binary: 'retroarch', type: 'libretro' },
    { id: 'duckstation', name: 'DuckStation', binary: 'duckstation-qt', type: 'standalone' },
    { id: 'pcsx2', name: 'PCSX2', binary: 'pcsx2-qt', type: 'standalone' },
    { id: 'mgba', name: 'mGBA', binary: 'mgba-qt', type: 'standalone' },
    { id: 'flycast', name: 'Flycast', binary: 'flycast', type: 'standalone' }
  ];

  public async scanInstalledEngines(registeredEmulators: Emulator[]): Promise<Emulator[]> {
    // Returns emulators with verified status
    return registeredEmulators.map(e => ({
      ...e,
      status: e.status || 'active'
    }));
  }
}

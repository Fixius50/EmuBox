import type { Game, Platform, Emulator, SystemSettings } from './game.types';
import type { HardwareInfo } from './system.types';

export interface StartupReport {
  phase: 'preparing' | 'prepared' | 'ready' | 'degraded' | 'error';
  resources: {
    cpuAvailable: number;
    memoryAvailableMb: number | null;
    maxConcurrentTasks: number;
    ioSlots: number;
    graphicsSlots: number;
    graphicsState: string | null;
    graphicsBackend: string | null;
    virtualMachine: boolean | null;
    storageAvailable: boolean | null;
  };
  tasks: { id: 'library' | 'hardware' | 'services' | 'emulators'; state: string; elapsedMs: number }[];
  warnings: string[];
  error: string | null;
  elapsedMs: number;
}

export interface StartupData {
  settings: SystemSettings;
  platforms: Platform[];
  games: Game[];
  hardware: HardwareInfo;
  emulators: Emulator[];
  report: StartupReport;
}
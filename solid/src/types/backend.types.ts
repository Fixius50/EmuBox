import type {
  Game,
  Platform,
  PlatformId,
  Emulator,
  SystemSettings,
  EmuBoxConfig
} from './game.types';

export interface SystemInfo {
  osName: string;
  kernelVersion: string;
  architecture: string;
  gpuRenderer: string;
  cpuModel: string;
  cpuCores: number;
  totalMemoryMb: number;
  usedMemoryMb: number;
  gamescopeAvailable: boolean;
  activeCompositor: string;
  batteryLevelPercent?: number;
  isPluggedIn?: boolean;
}

export interface GamepadDevice {
  index: number;
  id: string;
  name: string;
  connected: boolean;
  vendorId?: string;
  productId?: string;
  buttonsCount: number;
  axesCount: number;
  hasVibration: boolean;
  batteryPercent?: number;
  isPrimary: boolean;
}

export interface GamepadStatus {
  connectedCount: number;
  primaryDeviceIndex: number;
  devices: GamepadDevice[];
}

export interface LaunchGameRequest {
  gameId: string;
  emulatorId: string;
  romPath?: string;
  saveStateSlot?: number;
  customArgs?: string[];
  fullscreen?: boolean;
  useGamescope?: boolean;
}

export interface LaunchResult {
  success: boolean;
  message: string;
  pid?: number;
  executable?: string;
  startTime?: number;
}

export interface ScanGamesRequest {
  platforms?: PlatformId[];
  romsDirectory?: string;
  deepScan?: boolean;
}

export interface ScanGamesResult {
  scannedCount: number;
  addedCount: number;
  updatedCount: number;
  errors: string[];
}

export interface FirstRunDetectionResult {
  gpuVendor: 'amd' | 'nvidia' | 'intel' | 'generic';
  gpuRenderer: string;
  vulkanSupported: boolean;
  gamepadsDetected: string[];
  installedEmulators: string[];
  romsDirectoryFound: boolean;
  configGenerated: boolean;
}

export interface GameFilter {
  platform?: string;
  search?: string;
  favorite?: boolean;
  limit?: number;
  offset?: number;
}

/**
 * Master EmuBox Backend Interface.
 * Defines the strict, unified contract between the frontend UI (SolidJS)
 * and the execution backend (MockBackend during development, Tauri IPC in production).
 */
export interface IEmuBoxBackend {
  // System & Environment
  getSystemInfo(): Promise<SystemInfo>;
  runFirstRunDetection(): Promise<FirstRunDetectionResult>;

  // Central Versioned Configuration (XDG Base Directory)
  getConfig(): Promise<EmuBoxConfig>;
  saveConfig(config: EmuBoxConfig): Promise<void>;

  // Legacy/Runtime Quick Settings
  getSettings(): Promise<SystemSettings>;
  saveSettings(settings: SystemSettings): Promise<boolean>;

  // Games & Library
  scanGames(request?: ScanGamesRequest): Promise<ScanGamesResult>;
  getGames(filter?: GameFilter): Promise<Game[]>;
  getGameById(id: string): Promise<Game | null>;
  toggleFavorite(gameId: string): Promise<boolean>;

  // Platforms & Consoles
  getPlatforms(): Promise<Platform[]>;

  // Emulators & Libretro Cores (CRUD)
  getEmulators(): Promise<Emulator[]>;
  saveEmulator(emulator: Emulator): Promise<void>;
  deleteEmulator(id: string): Promise<void>;

  // Game Execution & Lifecycle
  launchGame(gameIdOrRequest: string | LaunchGameRequest, emulatorId?: string): Promise<LaunchResult>;
  stopGame(): Promise<void>;

  // Gamepad & Input
  getGamepadStatus(): Promise<GamepadStatus>;
}

import type {
  Game,
  Platform,
  PlatformId,
  Emulator,
  SystemSettings,
  EmuBoxConfig,
  CompatibilityAssociation
} from './game.types';

import type {
  SystemInfo,
  HardwareInfo,
  DisplayInfo,
  AudioInfo
} from './system.types';

import type {
  StorageInfo,
  StorageLocation
} from './storage.types';

import type {
  ProcessStatus,
  RunningGameInfo
} from './process.types';

import type {
  GamepadDevice,
  GamepadStatus
} from './input.types';

import type {
  DiagnosticReport,
  LogEntry
} from './diagnostics.types';

import type {
  BiosStatus
} from './bios.types';

import type {
  UpdateInfo,
  UpdateCheckResult,
  UpdateProgress,
  RollbackResult,
  UpdateChannel
} from './update.types';
import type { CreateDownloadRequest, DownloadJob, DownloadSource } from './download.types';

export * from './system.types';
export * from './storage.types';
export * from './process.types';
export * from './input.types';
export * from './diagnostics.types';
export * from './bios.types';
export * from './update.types';
export * from './events.types';
export * from './errors.types';
export * from './download.types';

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
  removedCount: number;
  totalCount: number;
  errors: string[];
}

export interface FirstRunDetectionResult {
  gpuVendor: HardwareInfo['gpuVendor'];
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
 * Universal contract implemented identically by MockBackend (dev) and TauriBackend (production).
 */
export interface IEmuBoxBackend {
  // 1. Sistema & Hardware Telemetry
  getSystemInfo(): Promise<SystemInfo>;
  getHardwareInfo(): Promise<HardwareInfo>;
  getDisplayInfo(): Promise<DisplayInfo>;
  getAudioInfo(): Promise<AudioInfo>;
  runFirstRunDetection(): Promise<FirstRunDetectionResult>;

  // 2. Configuración (Versioned & Legacy)
  getConfig(): Promise<EmuBoxConfig>;
  saveConfig(config: EmuBoxConfig): Promise<void>;
  getSettings(): Promise<SystemSettings>;
  saveSettings(settings: SystemSettings): Promise<boolean>;

  // 3. Biblioteca & Juegos
  getGames(filter?: GameFilter): Promise<Game[]>;
  getGame(id: string): Promise<Game | null>;
  getGameById(id: string): Promise<Game | null>;
  scanGames(request?: ScanGamesRequest): Promise<ScanGamesResult>;
  getPlatforms(): Promise<Platform[]>;
  toggleFavorite(gameId: string): Promise<boolean>;

  // 4. Emuladores & Cores Libretro
  getEmulators(): Promise<Emulator[]>;
  getEmulator(id: string): Promise<Emulator | null>;
  scanEmulators(): Promise<Emulator[]>;
  getEmulatorStatus(id: string): Promise<'active' | 'inactive' | 'missing_bios'>;
  saveEmulator(emulator: Emulator): Promise<void>;
  deleteEmulator(id: string): Promise<void>;

  // 4.1 Asociaciones Juego <-> Emulador (Compatibilidad SQLite)
  getGameAssociations(gameId: string): Promise<CompatibilityAssociation[]>;
  setGameAssociation(association: CompatibilityAssociation): Promise<void>;
  removeGameAssociation(gameId: string, emulatorId: string): Promise<void>;

  // 5. Ejecución & Procesos de Juego
  launchGame(gameIdOrRequest: string | LaunchGameRequest, emulatorId?: string): Promise<LaunchResult>;
  stopGame(): Promise<void>;
  isGameRunning(): Promise<boolean>;
  getRunningGame(): Promise<RunningGameInfo | null>;
  getProcessStatus(): Promise<ProcessStatus>;
  killProcess(pid: number): Promise<boolean>;

  // 6. Dispositivos de Entrada (Gamepads)
  getGamepads(): Promise<GamepadDevice[]>;
  getGamepadStatus(): Promise<GamepadStatus>;

  // 7. Sistema Operativo & Energía
  shutdown(): Promise<void>;
  restart(): Promise<void>;
  sleep(): Promise<void>;
  logout(): Promise<void>;
  restartAppSession(): Promise<void>;
  exitToLinuxShell(): Promise<void>;

  // 8. Almacenamiento & XDG
  getStorageInfo(): Promise<StorageInfo>;
  getStorageLocations(): Promise<Record<string, StorageLocation>>;

  // 9. Diagnóstico, Terminal & Logs
  getSystemLogs(limit?: number): Promise<LogEntry[]>;
  getEmuBoxLogs(limit?: number): Promise<LogEntry[]>;
  getDiagnostics(): Promise<DiagnosticReport>;
  executeCommand(cmd: string): Promise<string>;

  // 10. BIOS Scanner
  getBiosRequirements(): Promise<BiosStatus>;
  scanBios(): Promise<BiosStatus>;

  // 11. Actualización OTA, Desacoplamiento & Mantenimiento
  getUpdateInfo(): Promise<UpdateInfo>;
  checkForUpdates(channel?: UpdateChannel): Promise<UpdateCheckResult>;
  applyUpdate(targetVersion?: string): Promise<UpdateProgress>;
  rollbackToVersion(version: string): Promise<RollbackResult>;

  // 12. Descargas de fuentes autorizadas
  createDownloadSource(source: DownloadSource): Promise<DownloadSource>;
  createDownloadJob(request: CreateDownloadRequest): Promise<DownloadJob>;
  getDownloadJobs(): Promise<DownloadJob[]>;
  startDownload(id: string): Promise<DownloadJob>;
  pauseDownload(id: string): Promise<DownloadJob>;
  resumeDownload(id: string): Promise<DownloadJob>;
  cancelDownload(id: string): Promise<DownloadJob>;
  downloadGame(gameId: string): Promise<DownloadJob>;
  importDownloadLinks(): Promise<DownloadSource[]>;
  importDownloadsFromJson(jsonContent: string): Promise<DownloadSource[]>;
  importDownloadsFromUrl(url: string): Promise<DownloadSource[]>;
}

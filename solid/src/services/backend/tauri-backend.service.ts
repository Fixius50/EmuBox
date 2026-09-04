import type { Game, Platform, Emulator, SystemSettings, EmuBoxConfig, CompatibilityAssociation } from '@contracts/game.types';
import type {
  IEmuBoxBackend,
  LaunchResult,
  LaunchGameRequest,
  SystemInfo,
  HardwareInfo,
  DisplayInfo,
  AudioInfo,
  StorageInfo,
  StorageLocation,
  ProcessStatus,
  RunningGameInfo,
  GamepadDevice,
  GamepadStatus,
  ScanGamesRequest,
  ScanGamesResult,
  FirstRunDetectionResult,
  DiagnosticReport,
  LogEntry,
  BiosStatus,
  GameFilter,
  UpdateInfo,
  UpdateCheckResult,
  UpdateProgress,
  RollbackResult,
  UpdateChannel
} from '@contracts/backend.types';
import type { CreateDownloadRequest, DownloadJob, DownloadSource } from '@contracts/download.types';
import { MockBackendService } from './mock-backend.service';

/**
 * Tauri IPC Backend Implementation.
 * Bridges SolidJS frontend with Rust commands via Tauri IPC invoke calls.
 * Automatically and seamlessly falls back to MockBackendService in non-Tauri browser environments.
 */
export class TauriBackendService implements IEmuBoxBackend {
  private fallback: MockBackendService;
  private isTauri: boolean;

  constructor(fallbackMock?: MockBackendService) {
    this.fallback = fallbackMock || new MockBackendService();
    this.isTauri = typeof window !== 'undefined' && (!!(window as any).__TAURI_INTERNALS__ || !!(window as any).__TAURI__);
  }

  public get isTauriEnvironment(): boolean {
    return this.isTauri || (typeof window !== 'undefined' && (!!(window as any).__TAURI_INTERNALS__ || !!(window as any).__TAURI__));
  }

  private async invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
    if (typeof window !== 'undefined') {
      const internals = (window as any).__TAURI_INTERNALS__;
      if (internals && typeof internals.invoke === 'function') {
        return internals.invoke(cmd, args);
      }
      const tauri = (window as any).__TAURI__;
      if (tauri?.core && typeof tauri.core.invoke === 'function') {
        return tauri.core.invoke(cmd, args);
      }
    }
    throw new Error('Tauri runtime not detected');
  }

  // Only browser-dev (non-Tauri) sessions use the mock; real IPC failures must surface, not be masked.
  private async resolve<T>(cmd: string, args: Record<string, unknown> | undefined, fallback: () => Promise<T> | T): Promise<T> {
    if (!this.isTauriEnvironment) {
      return fallback();
    }
    return this.invoke<T>(cmd, args);
  }

  // 1. Sistema & Hardware Telemetry
  public async getSystemInfo(): Promise<SystemInfo> {
    return this.resolve('get_system_info', undefined, () => this.fallback.getSystemInfo());
  }

  public async getHardwareInfo(): Promise<HardwareInfo> {
    return this.resolve('get_hardware_info', undefined, () => this.fallback.getHardwareInfo());
  }

  public async getDisplayInfo(): Promise<DisplayInfo> {
    return this.resolve('get_display_info', undefined, () => this.fallback.getDisplayInfo());
  }

  public async getAudioInfo(): Promise<AudioInfo> {
    return this.resolve('get_audio_info', undefined, () => this.fallback.getAudioInfo());
  }

  public async runFirstRunDetection(): Promise<FirstRunDetectionResult> {
    return this.resolve('first_run_detection', undefined, () => this.fallback.runFirstRunDetection());
  }

  // 2. Configuración
  public async getConfig(): Promise<EmuBoxConfig> {
    return this.resolve('get_config', undefined, () => this.fallback.getConfig());
  }

  public async saveConfig(config: EmuBoxConfig): Promise<void> {
    return this.resolve('save_config', { config }, () => this.fallback.saveConfig(config));
  }

  public async getSettings(): Promise<SystemSettings> {
    return this.resolve('get_settings', undefined, () => this.fallback.getSettings());
  }

  public async saveSettings(settings: SystemSettings): Promise<boolean> {
    return this.resolve('save_settings', { settings }, () => this.fallback.saveSettings(settings));
  }

  // 3. Biblioteca & Juegos
  public async getGames(filter?: GameFilter): Promise<Game[]> {
    return this.resolve('get_games', { filter }, () => this.fallback.getGames(filter));
  }

  public async getGame(id: string): Promise<Game | null> {
    return this.resolve('get_game_by_id', { id }, () => this.fallback.getGame(id));
  }

  public async getGameById(id: string): Promise<Game | null> {
    return this.getGame(id);
  }

  public async scanGames(request?: ScanGamesRequest): Promise<ScanGamesResult> {
    return this.resolve('scan_games', { request }, () => this.fallback.scanGames(request));
  }

  public async getPlatforms(): Promise<Platform[]> {
    return this.resolve('get_platforms', undefined, () => this.fallback.getPlatforms());
  }

  public async toggleFavorite(gameId: string): Promise<boolean> {
    return this.resolve('toggle_favorite', { gameId }, () => this.fallback.toggleFavorite(gameId));
  }

  // 4. Emuladores (CRUD)
  public async getEmulators(): Promise<Emulator[]> {
    return this.resolve('get_emulators', undefined, () => this.fallback.getEmulators());
  }

  public async getEmulator(id: string): Promise<Emulator | null> {
    return this.resolve('get_emulator_by_id', { id }, () => this.fallback.getEmulator(id));
  }

  public async scanEmulators(): Promise<Emulator[]> {
    return this.resolve('scan_emulators', undefined, () => this.fallback.scanEmulators());
  }

  public async getEmulatorStatus(id: string): Promise<'active' | 'inactive' | 'missing_bios'> {
    return this.resolve('get_emulator_status', { id }, () => this.fallback.getEmulatorStatus(id));
  }

  public async saveEmulator(emulator: Emulator): Promise<void> {
    return this.resolve('save_emulator', { emulator }, () => this.fallback.saveEmulator(emulator));
  }

  public async deleteEmulator(id: string): Promise<void> {
    return this.resolve('delete_emulator', { id }, () => this.fallback.deleteEmulator(id));
  }

  // 4.1 Asociaciones Juego <-> Emulador (SQLite)
  public async getGameAssociations(gameId: string): Promise<CompatibilityAssociation[]> {
    return this.resolve('get_game_associations', { gameId }, () => this.fallback.getGameAssociations(gameId));
  }

  public async setGameAssociation(association: CompatibilityAssociation): Promise<void> {
    return this.resolve('set_game_association', { association }, () => this.fallback.setGameAssociation(association));
  }

  public async removeGameAssociation(gameId: string, emulatorId: string): Promise<void> {
    return this.resolve('remove_game_association', { gameId, emulatorId }, () => this.fallback.removeGameAssociation(gameId, emulatorId));
  }

  // 5. Ejecución & Procesos
  public async launchGame(gameIdOrRequest: string | LaunchGameRequest, emulatorId?: string): Promise<LaunchResult> {
    const request: LaunchGameRequest = typeof gameIdOrRequest === 'string'
      ? { gameId: gameIdOrRequest, emulatorId: emulatorId || '' }
      : gameIdOrRequest;

    return this.resolve('launch_game', { request }, () => this.fallback.launchGame(gameIdOrRequest, emulatorId));
  }

  public async createDownloadSource(source: DownloadSource): Promise<DownloadSource> {
    return this.invoke<DownloadSource>('create_download_source', { source });
  }

  public async createDownloadJob(request: CreateDownloadRequest): Promise<DownloadJob> {
    return this.invoke<DownloadJob>('create_download_job', { request });
  }

  public async getDownloadJobs(): Promise<DownloadJob[]> {
    return this.invoke<DownloadJob[]>('get_download_jobs');
  }

  public async startDownload(id: string): Promise<DownloadJob> {
    return this.invoke<DownloadJob>('start_download', { id });
  }

  public async pauseDownload(id: string): Promise<DownloadJob> {
    return this.invoke<DownloadJob>('pause_download', { id });
  }

  public async resumeDownload(id: string): Promise<DownloadJob> {
    return this.invoke<DownloadJob>('resume_download', { id });
  }

  public async cancelDownload(id: string): Promise<DownloadJob> {
    return this.invoke<DownloadJob>('cancel_download', { id });
  }

  public async downloadGame(gameId: string): Promise<DownloadJob> {
    return this.resolve('download_game', { gameId }, () => this.fallback.downloadGame(gameId));
  }

  public async importDownloadLinks(): Promise<DownloadJob[]> {
    return this.resolve('import_download_links', undefined, () => this.fallback.importDownloadLinks());
  }

  public async importDownloadsFromJson(jsonContent: string): Promise<DownloadJob[]> {
    return this.resolve('import_downloads_from_json', { jsonContent }, () => this.fallback.importDownloadsFromJson(jsonContent));
  }

  public async importDownloadsFromUrl(url: string): Promise<DownloadJob[]> {
    return this.resolve('import_downloads_from_url', { url }, () => this.fallback.importDownloadsFromUrl(url));
  }


  public async stopGame(): Promise<void> {
    return this.resolve('stop_game', undefined, () => this.fallback.stopGame());
  }

  public async isGameRunning(): Promise<boolean> {
    return this.resolve('is_game_running', undefined, () => this.fallback.isGameRunning());
  }

  public async getRunningGame(): Promise<RunningGameInfo | null> {
    return this.resolve('get_running_game', undefined, () => this.fallback.getRunningGame());
  }

  public async getProcessStatus(): Promise<ProcessStatus> {
    return this.resolve('get_process_status', undefined, () => this.fallback.getProcessStatus());
  }

  public async killProcess(pid: number): Promise<boolean> {
    return this.resolve('kill_process', { pid }, () => this.fallback.killProcess(pid));
  }

  // 6. Input / Gamepads
  public async getGamepads(): Promise<GamepadDevice[]> {
    return this.resolve('get_gamepads', undefined, () => this.fallback.getGamepads());
  }

  public async getGamepadStatus(): Promise<GamepadStatus> {
    return this.resolve('get_gamepad_status', undefined, () => this.fallback.getGamepadStatus());
  }

  // 7. Sistema Operativo & Energía
  public async shutdown(): Promise<void> {
    return this.resolve('system_shutdown', undefined, () => this.fallback.shutdown());
  }

  public async restart(): Promise<void> {
    return this.resolve('system_restart', undefined, () => this.fallback.restart());
  }

  public async sleep(): Promise<void> {
    return this.resolve('system_sleep', undefined, () => this.fallback.sleep());
  }

  public async logout(): Promise<void> {
    return this.resolve('system_logout', undefined, () => this.fallback.logout());
  }

  public async restartAppSession(): Promise<void> {
    return this.resolve('restart_app_session', undefined, () => this.fallback.restartAppSession());
  }

  public async exitToLinuxShell(): Promise<void> {
    return this.resolve('exit_to_linux_shell', undefined, () => this.fallback.exitToLinuxShell());
  }

  // 8. Almacenamiento & XDG
  public async getStorageInfo(): Promise<StorageInfo> {
    return this.resolve('get_storage_info', undefined, () => this.fallback.getStorageInfo());
  }

  public async getStorageLocations(): Promise<Record<string, StorageLocation>> {
    return this.resolve('get_storage_locations', undefined, () => this.fallback.getStorageLocations());
  }

  // 9. Diagnóstico & Logs
  public async getSystemLogs(limit?: number): Promise<LogEntry[]> {
    return this.resolve('get_system_logs', { limit }, () => this.fallback.getSystemLogs(limit));
  }

  public async getEmuBoxLogs(limit?: number): Promise<LogEntry[]> {
    return this.resolve('get_emubox_logs', { limit }, () => this.fallback.getEmuBoxLogs(limit));
  }

  public async getDiagnostics(): Promise<DiagnosticReport> {
    return this.resolve('get_diagnostics', undefined, () => this.fallback.getDiagnostics());
  }

  // 10. BIOS Scanner
  public async getBiosRequirements(): Promise<BiosStatus> {
    return this.resolve('get_bios_requirements', undefined, () => this.fallback.getBiosRequirements());
  }

  public async scanBios(): Promise<BiosStatus> {
    return this.resolve('scan_bios', undefined, () => this.fallback.scanBios());
  }

  // 11. Actualización OTA & Mantenimiento Desacoplado
  public async getUpdateInfo(): Promise<UpdateInfo> {
    return this.resolve('get_update_info', undefined, () => this.fallback.getUpdateInfo());
  }

  public async checkForUpdates(channel?: UpdateChannel): Promise<UpdateCheckResult> {
    return this.resolve('check_for_updates', { channel }, () => this.fallback.checkForUpdates(channel));
  }

  public async applyUpdate(targetVersion?: string): Promise<UpdateProgress> {
    return this.resolve('apply_update', { targetVersion }, () => this.fallback.applyUpdate(targetVersion));
  }

  public async rollbackToVersion(version: string): Promise<RollbackResult> {
    return this.resolve('rollback_to_version', { version }, () => this.fallback.rollbackToVersion(version));
  }

  public async executeCommand(cmd: string): Promise<string> {
    return this.resolve('execute_command', { command: cmd }, () => this.fallback.executeCommand(cmd));
  }
}

export { TauriBackendService as TauriBackend };

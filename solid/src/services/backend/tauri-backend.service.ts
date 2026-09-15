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
import { invoke, isTauri } from '@tauri-apps/api/core';
import type { StartupData, StartupReport } from '@contracts/startup.types';

/**
 * Tauri IPC Backend Implementation.
 * Bridges SolidJS frontend with Rust commands via Tauri IPC invoke calls.
 */
export class TauriBackendService implements IEmuBoxBackend {
  public get isTauriEnvironment(): boolean {
    return typeof window !== 'undefined' && isTauri();
  }

  private async invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
    if (!this.isTauriEnvironment) {
      throw new Error('EmuBox requiere el runtime nativo Tauri. Inicia la aplicacion nativa.');
    }
    return invoke<T>(cmd, args);
  }

  // 1. Sistema & Hardware Telemetry
  public getStartupStatus(): Promise<StartupReport> {
    return this.invoke('get_startup_status');
  }

  public getStartupData(): Promise<StartupData> {
    return this.invoke('get_startup_data');
  }

  public startupFrontendReady(): Promise<StartupReport> {
    return this.invoke('startup_frontend_ready');
  }

  public async getSystemInfo(): Promise<SystemInfo> {
    return this.invoke('get_system_info', undefined);
  }

  public async getHardwareInfo(): Promise<HardwareInfo> {
    return this.invoke('get_hardware_info', undefined);
  }

  public async getDisplayInfo(): Promise<DisplayInfo> {
    return this.invoke('get_display_info', undefined);
  }

  public async getAudioInfo(): Promise<AudioInfo> {
    return this.invoke('get_audio_info', undefined);
  }

  public async runFirstRunDetection(): Promise<FirstRunDetectionResult> {
    return this.invoke('first_run_detection', undefined);
  }

  // 2. Configuración
  public async getConfig(): Promise<EmuBoxConfig> {
    return this.invoke('get_config', undefined);
  }

  public async saveConfig(config: EmuBoxConfig): Promise<void> {
    return this.invoke('save_config', { config });
  }

  public async getSettings(): Promise<SystemSettings> {
    return this.invoke('get_settings', undefined);
  }

  public async saveSettings(settings: SystemSettings): Promise<boolean> {
    return this.invoke('save_settings', { settings });
  }

  // 3. Biblioteca & Juegos
  public async getGames(filter?: GameFilter): Promise<Game[]> {
    return this.invoke('get_games', { filter });
  }

  public async getGame(id: string): Promise<Game | null> {
    return this.invoke('get_game_by_id', { id });
  }

  public async getGameById(id: string): Promise<Game | null> {
    return this.getGame(id);
  }

  public async scanGames(request?: ScanGamesRequest): Promise<ScanGamesResult> {
    return this.invoke('scan_games', { request });
  }

  public async getPlatforms(): Promise<Platform[]> {
    return this.invoke('get_platforms', undefined);
  }

  public async toggleFavorite(gameId: string): Promise<boolean> {
    return this.invoke('toggle_favorite', { gameId });
  }

  // 4. Emuladores (CRUD)
  public async getEmulators(): Promise<Emulator[]> {
    return this.invoke('get_emulators', undefined);
  }

  public async getEmulator(id: string): Promise<Emulator | null> {
    return this.invoke('get_emulator_by_id', { id });
  }

  public async scanEmulators(): Promise<Emulator[]> {
    return this.invoke('scan_emulators', undefined);
  }

  public async getEmulatorStatus(id: string): Promise<'active' | 'inactive' | 'missing_bios'> {
    return this.invoke('get_emulator_status', { id });
  }

  public async saveEmulator(emulator: Emulator): Promise<void> {
    return this.invoke('save_emulator', { emulator });
  }

  public async deleteEmulator(id: string): Promise<void> {
    return this.invoke('delete_emulator', { id });
  }

  // 4.1 Asociaciones Juego <-> Emulador (SQLite)
  public async getGameAssociations(gameId: string): Promise<CompatibilityAssociation[]> {
    return this.invoke('get_game_associations', { gameId });
  }

  public async setGameAssociation(association: CompatibilityAssociation): Promise<void> {
    return this.invoke('set_game_association', { association });
  }

  public async removeGameAssociation(gameId: string, emulatorId: string): Promise<void> {
    return this.invoke('remove_game_association', { gameId, emulatorId });
  }

  // 5. Ejecución & Procesos
  public async launchGame(gameIdOrRequest: string | LaunchGameRequest, emulatorId?: string): Promise<LaunchResult> {
    const request: LaunchGameRequest = typeof gameIdOrRequest === 'string'
      ? { gameId: gameIdOrRequest, emulatorId: emulatorId || '' }
      : gameIdOrRequest;

    return this.invoke('launch_game', { request });
  }

  public async createDownloadSource(source: DownloadSource): Promise<DownloadSource> {
    return this.invoke<DownloadSource>('create_download_source', { source });
  }

  public async createDownloadJob(request: CreateDownloadRequest): Promise<DownloadJob> {
    return this.invoke<DownloadJob>('create_download_job', { request });
  }

  public async getDownloadJobs(): Promise<DownloadJob[]> {
    return this.invoke('get_download_jobs', undefined);
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

  public async getDownloadCandidates(id: string): Promise<string[]> {
    return this.invoke('get_download_candidates', { id });
  }

  public async selectDownloadCandidate(id: string, path: string): Promise<DownloadJob> {
    return this.invoke('select_download_candidate', { id, path });
  }

  public async downloadGame(gameId: string, sourceId?: string): Promise<DownloadJob> {
    return this.invoke('download_game', { gameId, sourceId });
  }

  public async getDownloadSources(gameId: string): Promise<import('@contracts/download.types').DownloadSourceOption[]> {
    return this.invoke('get_download_sources', { gameId });
  }

  public async importDownloadLinks(): Promise<DownloadSource[]> {
    return this.invoke('import_download_links', undefined);
  }

  public async importDownloadsFromJson(jsonContent: string): Promise<DownloadSource[]> {
    return this.invoke('import_downloads_from_json', { jsonContent });
  }

  public async importDownloadsFromUrl(url: string): Promise<DownloadSource[]> {
    return this.invoke('import_downloads_from_url', { url });
  }


  public async stopGame(): Promise<void> {
    return this.invoke('stop_game', undefined);
  }

  public async isGameRunning(): Promise<boolean> {
    return this.invoke('is_game_running', undefined);
  }

  public async getRunningGame(): Promise<RunningGameInfo | null> {
    return this.invoke('get_running_game', undefined);
  }

  public async getProcessStatus(): Promise<ProcessStatus> {
    return this.invoke('get_process_status', undefined);
  }

  public async killProcess(pid: number): Promise<boolean> {
    return this.invoke('kill_process', { pid });
  }

  // 6. Input / Gamepads
  public async getGamepads(): Promise<GamepadDevice[]> {
    return this.invoke('get_gamepads', undefined);
  }

  public async getGamepadStatus(): Promise<GamepadStatus> {
    return this.invoke('get_gamepad_status', undefined);
  }

  // 7. Sistema Operativo & Energía
  public async shutdown(): Promise<void> {
    return this.invoke('system_shutdown', undefined);
  }

  public async restart(): Promise<void> {
    return this.invoke('system_restart', undefined);
  }

  public async sleep(): Promise<void> {
    return this.invoke('system_sleep', undefined);
  }

  public async logout(): Promise<void> {
    return this.invoke('system_logout', undefined);
  }

  public async restartAppSession(): Promise<void> {
    return this.invoke('restart_app_session', undefined);
  }

  public async exitToLinuxShell(): Promise<void> {
    return this.invoke('exit_to_linux_shell', undefined);
  }

  // 8. Almacenamiento & XDG
  public async getStorageInfo(): Promise<StorageInfo> {
    return this.invoke('get_storage_info', undefined);
  }

  public async getStorageLocations(): Promise<Record<string, StorageLocation>> {
    return this.invoke('get_storage_locations', undefined);
  }

  // 9. Diagnóstico & Logs
  public async getSystemLogs(limit?: number): Promise<LogEntry[]> {
    return this.invoke('get_system_logs', { limit });
  }

  public async getEmuBoxLogs(limit?: number): Promise<LogEntry[]> {
    return this.invoke('get_emubox_logs', { limit });
  }

  public async getDiagnostics(): Promise<DiagnosticReport> {
    return this.invoke('get_diagnostics', undefined);
  }

  // 10. BIOS Scanner
  public async getBiosRequirements(): Promise<BiosStatus> {
    return this.invoke('get_bios_requirements', undefined);
  }

  public async scanBios(): Promise<BiosStatus> {
    return this.invoke('scan_bios', undefined);
  }

  // 11. Actualización OTA & Mantenimiento Desacoplado
  public async getUpdateInfo(): Promise<UpdateInfo> {
    throw new Error('Actualizacion OTA no disponible en este runtime.');
  }

  public async checkForUpdates(_channel?: UpdateChannel): Promise<UpdateCheckResult> {
    throw new Error('Comprobacion OTA no disponible en este runtime.');
  }

  public async applyUpdate(_targetVersion?: string): Promise<UpdateProgress> {
    throw new Error('Actualizacion OTA no disponible en este runtime.');
  }

  public async rollbackToVersion(_version: string): Promise<RollbackResult> {
    throw new Error('Restauracion OTA no disponible en este runtime.');
  }

  public async executeCommand(cmd: string): Promise<string> {
    return this.invoke('execute_command', { command: cmd });
  }
}

export { TauriBackendService as TauriBackend };

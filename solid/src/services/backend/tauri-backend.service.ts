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

  // 1. Sistema & Hardware Telemetry
  public async getSystemInfo(): Promise<SystemInfo> {
    try {
      return await this.invoke<SystemInfo>('get_system_info');
    } catch {
      return this.fallback.getSystemInfo();
    }
  }

  public async getHardwareInfo(): Promise<HardwareInfo> {
    try {
      return await this.invoke<HardwareInfo>('get_hardware_info');
    } catch {
      return this.fallback.getHardwareInfo();
    }
  }

  public async getDisplayInfo(): Promise<DisplayInfo> {
    try {
      return await this.invoke<DisplayInfo>('get_display_info');
    } catch {
      return this.fallback.getDisplayInfo();
    }
  }

  public async getAudioInfo(): Promise<AudioInfo> {
    try {
      return await this.invoke<AudioInfo>('get_audio_info');
    } catch {
      return this.fallback.getAudioInfo();
    }
  }

  public async runFirstRunDetection(): Promise<FirstRunDetectionResult> {
    try {
      return await this.invoke<FirstRunDetectionResult>('first_run_detection');
    } catch {
      return this.fallback.runFirstRunDetection();
    }
  }

  // 2. Configuración
  public async getConfig(): Promise<EmuBoxConfig> {
    try {
      return await this.invoke<EmuBoxConfig>('get_config');
    } catch {
      return this.fallback.getConfig();
    }
  }

  public async saveConfig(config: EmuBoxConfig): Promise<void> {
    try {
      await this.invoke('save_config', { config });
    } catch {
      await this.fallback.saveConfig(config);
    }
  }

  public async getSettings(): Promise<SystemSettings> {
    try {
      return await this.invoke<SystemSettings>('get_settings');
    } catch {
      return this.fallback.getSettings();
    }
  }

  public async saveSettings(settings: SystemSettings): Promise<boolean> {
    try {
      return await this.invoke<boolean>('save_settings', { settings });
    } catch {
      return this.fallback.saveSettings(settings);
    }
  }

  // 3. Biblioteca & Juegos
  public async getGames(filter?: GameFilter): Promise<Game[]> {
    try {
      return await this.invoke<Game[]>('get_games', { filter });
    } catch {
      return this.fallback.getGames(filter);
    }
  }

  public async getGame(id: string): Promise<Game | null> {
    try {
      return await this.invoke<Game | null>('get_game_by_id', { id });
    } catch {
      return this.fallback.getGame(id);
    }
  }

  public async getGameById(id: string): Promise<Game | null> {
    return this.getGame(id);
  }

  public async scanGames(request?: ScanGamesRequest): Promise<ScanGamesResult> {
    try {
      return await this.invoke<ScanGamesResult>('scan_games', { request });
    } catch {
      return this.fallback.scanGames(request);
    }
  }

  public async getPlatforms(): Promise<Platform[]> {
    try {
      return await this.invoke<Platform[]>('get_platforms');
    } catch {
      return this.fallback.getPlatforms();
    }
  }

  public async toggleFavorite(gameId: string): Promise<boolean> {
    try {
      return await this.invoke<boolean>('toggle_favorite', { gameId });
    } catch {
      return this.fallback.toggleFavorite(gameId);
    }
  }

  // 4. Emuladores (CRUD)
  public async getEmulators(): Promise<Emulator[]> {
    try {
      return await this.invoke<Emulator[]>('get_emulators');
    } catch {
      return this.fallback.getEmulators();
    }
  }

  public async getEmulator(id: string): Promise<Emulator | null> {
    try {
      return await this.invoke<Emulator | null>('get_emulator_by_id', { id });
    } catch {
      return this.fallback.getEmulator(id);
    }
  }

  public async scanEmulators(): Promise<Emulator[]> {
    try {
      return await this.invoke<Emulator[]>('scan_emulators');
    } catch {
      return this.fallback.scanEmulators();
    }
  }

  public async getEmulatorStatus(id: string): Promise<'active' | 'inactive' | 'missing_bios'> {
    try {
      return await this.invoke<'active' | 'inactive' | 'missing_bios'>('get_emulator_status', { id });
    } catch {
      return this.fallback.getEmulatorStatus(id);
    }
  }

  public async saveEmulator(emulator: Emulator): Promise<void> {
    try {
      await this.invoke('save_emulator', { emulator });
    } catch {
      await this.fallback.saveEmulator(emulator);
    }
  }

  public async deleteEmulator(id: string): Promise<void> {
    try {
      await this.invoke('delete_emulator', { id });
    } catch {
      await this.fallback.deleteEmulator(id);
    }
  }

  // 4.1 Asociaciones Juego <-> Emulador (SQLite)
  public async getGameAssociations(gameId: string): Promise<CompatibilityAssociation[]> {
    try {
      return await this.invoke<CompatibilityAssociation[]>('get_game_associations', { gameId });
    } catch {
      return this.fallback.getGameAssociations(gameId);
    }
  }

  public async setGameAssociation(association: CompatibilityAssociation): Promise<void> {
    try {
      await this.invoke('set_game_association', { association });
    } catch {
      await this.fallback.setGameAssociation(association);
    }
  }

  public async removeGameAssociation(gameId: string, emulatorId: string): Promise<void> {
    try {
      await this.invoke('remove_game_association', { gameId, emulatorId });
    } catch {
      await this.fallback.removeGameAssociation(gameId, emulatorId);
    }
  }

  // 5. Ejecución & Procesos
  public async launchGame(gameIdOrRequest: string | LaunchGameRequest, emulatorId?: string): Promise<LaunchResult> {
    try {
      const request: LaunchGameRequest = typeof gameIdOrRequest === 'string'
        ? { gameId: gameIdOrRequest, emulatorId: emulatorId || '' }
        : gameIdOrRequest;

      return await this.invoke<LaunchResult>('launch_game', { request });
    } catch {
      return this.fallback.launchGame(gameIdOrRequest, emulatorId);
    }
  }

  public async stopGame(): Promise<void> {
    try {
      await this.invoke('stop_game');
    } catch {
      await this.fallback.stopGame();
    }
  }

  public async isGameRunning(): Promise<boolean> {
    try {
      return await this.invoke<boolean>('is_game_running');
    } catch {
      return this.fallback.isGameRunning();
    }
  }

  public async getRunningGame(): Promise<RunningGameInfo | null> {
    try {
      return await this.invoke<RunningGameInfo | null>('get_running_game');
    } catch {
      return this.fallback.getRunningGame();
    }
  }

  public async getProcessStatus(): Promise<ProcessStatus> {
    try {
      return await this.invoke<ProcessStatus>('get_process_status');
    } catch {
      return this.fallback.getProcessStatus();
    }
  }

  public async killProcess(pid: number): Promise<boolean> {
    try {
      return await this.invoke<boolean>('kill_process', { pid });
    } catch {
      return this.fallback.killProcess(pid);
    }
  }

  // 6. Input / Gamepads
  public async getGamepads(): Promise<GamepadDevice[]> {
    try {
      return await this.invoke<GamepadDevice[]>('get_gamepads');
    } catch {
      return this.fallback.getGamepads();
    }
  }

  public async getGamepadStatus(): Promise<GamepadStatus> {
    try {
      return await this.invoke<GamepadStatus>('get_gamepad_status');
    } catch {
      return this.fallback.getGamepadStatus();
    }
  }

  // 7. Sistema Operativo & Energía
  public async shutdown(): Promise<void> {
    try {
      await this.invoke('system_shutdown');
    } catch {
      await this.fallback.shutdown();
    }
  }

  public async restart(): Promise<void> {
    try {
      await this.invoke('system_restart');
    } catch {
      await this.fallback.restart();
    }
  }

  public async sleep(): Promise<void> {
    try {
      await this.invoke('system_sleep');
    } catch {
      await this.fallback.sleep();
    }
  }

  public async logout(): Promise<void> {
    try {
      await this.invoke('system_logout');
    } catch {
      await this.fallback.logout();
    }
  }

  public async restartAppSession(): Promise<void> {
    try {
      await this.invoke('restart_app_session');
    } catch {
      await this.fallback.restartAppSession();
    }
  }

  public async exitToLinuxShell(): Promise<void> {
    try {
      await this.invoke('exit_to_linux_shell');
    } catch {
      await this.fallback.exitToLinuxShell();
    }
  }

  // 8. Almacenamiento & XDG
  public async getStorageInfo(): Promise<StorageInfo> {
    try {
      return await this.invoke<StorageInfo>('get_storage_info');
    } catch {
      return this.fallback.getStorageInfo();
    }
  }

  public async getStorageLocations(): Promise<Record<string, StorageLocation>> {
    try {
      return await this.invoke<Record<string, StorageLocation>>('get_storage_locations');
    } catch {
      return this.fallback.getStorageLocations();
    }
  }

  // 9. Diagnóstico & Logs
  public async getSystemLogs(limit?: number): Promise<LogEntry[]> {
    try {
      return await this.invoke<LogEntry[]>('get_system_logs', { limit });
    } catch {
      return this.fallback.getSystemLogs(limit);
    }
  }

  public async getEmuBoxLogs(limit?: number): Promise<LogEntry[]> {
    try {
      return await this.invoke<LogEntry[]>('get_emubox_logs', { limit });
    } catch {
      return this.fallback.getEmuBoxLogs(limit);
    }
  }

  public async getDiagnostics(): Promise<DiagnosticReport> {
    try {
      return await this.invoke<DiagnosticReport>('get_diagnostics');
    } catch {
      return this.fallback.getDiagnostics();
    }
  }

  // 10. BIOS Scanner
  public async getBiosRequirements(): Promise<BiosStatus> {
    try {
      return await this.invoke<BiosStatus>('get_bios_requirements');
    } catch {
      return this.fallback.getBiosRequirements();
    }
  }

  public async scanBios(): Promise<BiosStatus> {
    try {
      return await this.invoke<BiosStatus>('scan_bios');
    } catch {
      return this.fallback.scanBios();
    }
  }

  // 11. Actualización OTA & Mantenimiento Desacoplado
  public async getUpdateInfo(): Promise<UpdateInfo> {
    try {
      return await this.invoke<UpdateInfo>('get_update_info');
    } catch {
      return this.fallback.getUpdateInfo();
    }
  }

  public async checkForUpdates(channel?: UpdateChannel): Promise<UpdateCheckResult> {
    try {
      return await this.invoke<UpdateCheckResult>('check_for_updates', { channel });
    } catch {
      return this.fallback.checkForUpdates(channel);
    }
  }

  public async applyUpdate(targetVersion?: string): Promise<UpdateProgress> {
    try {
      return await this.invoke<UpdateProgress>('apply_update', { targetVersion });
    } catch {
      return this.fallback.applyUpdate(targetVersion);
    }
  }

  public async rollbackToVersion(version: string): Promise<RollbackResult> {
    try {
      return await this.invoke<RollbackResult>('rollback_to_version', { version });
    } catch {
      return this.fallback.rollbackToVersion(version);
    }
  }

  public async executeCommand(cmd: string): Promise<string> {
    try {
      return await this.invoke<string>('execute_command', { command: cmd });
    } catch {
      return this.fallback.executeCommand(cmd);
    }
  }
}

export { TauriBackendService as TauriBackend };

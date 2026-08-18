import type { Game, Platform, Emulator, SystemSettings, EmuBoxConfig } from '@contracts/game.types';
import type {
  IEmuBoxBackend,
  LaunchResult,
  LaunchGameRequest,
  SystemInfo,
  GamepadStatus,
  ScanGamesRequest,
  ScanGamesResult,
  FirstRunDetectionResult,
  GameFilter
} from '@contracts/backend.types';
import { MockBackendService } from './mock-backend.service';

/**
 * Tauri IPC Backend Implementation.
 * Bridges SolidJS frontend with Rust backend via Tauri IPC invoke calls.
 * Automatically and seamlessly falls back to MockBackendService in non-Tauri browser environments.
 */
export class TauriBackendService implements IEmuBoxBackend {
  private fallback: MockBackendService;
  private isTauri: boolean;

  constructor(fallbackMock?: MockBackendService) {
    this.fallback = fallbackMock || new MockBackendService();
    this.isTauri = typeof window !== 'undefined' && !!(window as any).__TAURI__;
  }

  private async invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
    if (this.isTauri && (window as any).__TAURI__?.core?.invoke) {
      return (window as any).__TAURI__.core.invoke(cmd, args);
    }
    throw new Error('Tauri runtime not detected');
  }

  // System & Environment
  public async getSystemInfo(): Promise<SystemInfo> {
    try {
      return await this.invoke<SystemInfo>('get_system_info');
    } catch {
      return this.fallback.getSystemInfo();
    }
  }

  public async runFirstRunDetection(): Promise<FirstRunDetectionResult> {
    try {
      return await this.invoke<FirstRunDetectionResult>('first_run_detection');
    } catch {
      return this.fallback.runFirstRunDetection();
    }
  }

  // Central Versioned Configuration
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

  // Settings & Legacy Support
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

  // Games & Library
  public async scanGames(request?: ScanGamesRequest): Promise<ScanGamesResult> {
    try {
      return await this.invoke<ScanGamesResult>('scan_games', { request });
    } catch {
      return this.fallback.scanGames(request);
    }
  }

  public async getGames(filter?: GameFilter): Promise<Game[]> {
    try {
      return await this.invoke<Game[]>('get_games', { filter });
    } catch {
      return this.fallback.getGames(filter);
    }
  }

  public async getGameById(id: string): Promise<Game | null> {
    try {
      return await this.invoke<Game | null>('get_game_by_id', { id });
    } catch {
      return this.fallback.getGameById(id);
    }
  }

  public async toggleFavorite(gameId: string): Promise<boolean> {
    try {
      return await this.invoke<boolean>('toggle_favorite', { gameId });
    } catch {
      return this.fallback.toggleFavorite(gameId);
    }
  }

  // Platforms & Consoles
  public async getPlatforms(): Promise<Platform[]> {
    try {
      return await this.invoke<Platform[]>('get_platforms');
    } catch {
      return this.fallback.getPlatforms();
    }
  }

  // Emulators (CRUD)
  public async getEmulators(): Promise<Emulator[]> {
    try {
      return await this.invoke<Emulator[]>('get_emulators');
    } catch {
      return this.fallback.getEmulators();
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

  // Game Execution & Lifecycle
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

  // Gamepad Status
  public async getGamepadStatus(): Promise<GamepadStatus> {
    try {
      return await this.invoke<GamepadStatus>('get_gamepad_status');
    } catch {
      return this.fallback.getGamepadStatus();
    }
  }
}

export { TauriBackendService as TauriBackend };

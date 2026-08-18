import type { Game, Platform, Emulator, SystemSettings } from '@contracts/game.types';
import type { IEmuBoxBackend, LaunchResult } from '@contracts/backend.types';

interface TauriWindow {
  __TAURI__?: {
    invoke: <T = unknown>(cmd: string, args?: Record<string, unknown>) => Promise<T>;
  };
}

export class TauriBackendService implements IEmuBoxBackend {
  private fallbackBackend: IEmuBoxBackend;

  constructor(fallbackBackend: IEmuBoxBackend) {
    this.fallbackBackend = fallbackBackend;
  }

  private isTauriEnvironment(): boolean {
    return typeof window !== 'undefined' && !!(window as unknown as TauriWindow).__TAURI__;
  }

  public async getGames(filter?: { platform?: string; search?: string; favorite?: boolean; limit?: number }): Promise<Game[]> {
    if (!this.isTauriEnvironment()) {
      return this.fallbackBackend.getGames(filter);
    }
    try {
      const tauri = (window as unknown as TauriWindow).__TAURI__!;
      return await tauri.invoke<Game[]>('get_games', { filter });
    } catch (err) {
      console.warn('[TauriBackend] Error en get_games, usando fallback:', err);
      return this.fallbackBackend.getGames(filter);
    }
  }

  public async getGameById(id: string): Promise<Game | null> {
    if (!this.isTauriEnvironment()) {
      return this.fallbackBackend.getGameById(id);
    }
    try {
      const tauri = (window as unknown as TauriWindow).__TAURI__!;
      return await tauri.invoke<Game | null>('get_game_by_id', { id });
    } catch {
      return this.fallbackBackend.getGameById(id);
    }
  }

  public async getPlatforms(): Promise<Platform[]> {
    if (!this.isTauriEnvironment()) {
      return this.fallbackBackend.getPlatforms();
    }
    try {
      const tauri = (window as unknown as TauriWindow).__TAURI__!;
      return await tauri.invoke<Platform[]>('get_platforms');
    } catch {
      return this.fallbackBackend.getPlatforms();
    }
  }

  public async getEmulators(): Promise<Emulator[]> {
    if (!this.isTauriEnvironment()) {
      return this.fallbackBackend.getEmulators();
    }
    try {
      const tauri = (window as unknown as TauriWindow).__TAURI__!;
      return await tauri.invoke<Emulator[]>('get_emulators');
    } catch {
      return this.fallbackBackend.getEmulators();
    }
  }

  public async getSettings(): Promise<SystemSettings> {
    if (!this.isTauriEnvironment()) {
      return this.fallbackBackend.getSettings();
    }
    try {
      const tauri = (window as unknown as TauriWindow).__TAURI__!;
      return await tauri.invoke<SystemSettings>('get_settings');
    } catch {
      return this.fallbackBackend.getSettings();
    }
  }

  public async saveSettings(settings: SystemSettings): Promise<boolean> {
    if (!this.isTauriEnvironment()) {
      return this.fallbackBackend.saveSettings(settings);
    }
    try {
      const tauri = (window as unknown as TauriWindow).__TAURI__!;
      return await tauri.invoke<boolean>('save_settings', { settings });
    } catch {
      return this.fallbackBackend.saveSettings(settings);
    }
  }

  public async toggleFavorite(gameId: string): Promise<boolean> {
    if (!this.isTauriEnvironment()) {
      return this.fallbackBackend.toggleFavorite(gameId);
    }
    try {
      const tauri = (window as unknown as TauriWindow).__TAURI__!;
      return await tauri.invoke<boolean>('toggle_favorite', { gameId });
    } catch {
      return this.fallbackBackend.toggleFavorite(gameId);
    }
  }

  public async launchGame(gameId: string, emulatorId: string): Promise<LaunchResult> {
    if (!this.isTauriEnvironment()) {
      return this.fallbackBackend.launchGame(gameId, emulatorId);
    }
    try {
      const tauri = (window as unknown as TauriWindow).__TAURI__!;
      return await tauri.invoke<LaunchResult>('launch_game', { gameId, emulatorId });
    } catch (err) {
      console.warn('[TauriBackend] Error en launch_game, usando fallback:', err);
      return this.fallbackBackend.launchGame(gameId, emulatorId);
    }
  }
}

export { TauriBackendService as TauriBackend };

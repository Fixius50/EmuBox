import type { Game, Platform, Emulator, SystemSettings } from './game.types';

export interface LaunchResult {
  success: boolean;
  message: string;
  pid?: number;
  executable?: string;
}

export interface IEmuBoxBackend {
  getGames(filter?: { platform?: string; search?: string; favorite?: boolean; limit?: number }): Promise<Game[]>;
  getGameById(id: string): Promise<Game | null>;
  getPlatforms(): Promise<Platform[]>;
  getEmulators(): Promise<Emulator[]>;
  getSettings(): Promise<SystemSettings>;
  saveSettings(settings: SystemSettings): Promise<boolean>;
  toggleFavorite(gameId: string): Promise<boolean>;
  launchGame(gameId: string, emulatorId: string): Promise<LaunchResult>;
}

import type { Game, Platform, PlatformId } from '@contracts/game.types';
import type { IEmuBoxBackend, GameFilter } from '@contracts/backend.types';

export class GameService {
  constructor(private backend: IEmuBoxBackend) {}

  public async getGames(filter?: GameFilter): Promise<Game[]> {
    return this.backend.getGames(filter);
  }

  public async getGameById(id: string): Promise<Game | null> {
    return this.backend.getGame(id);
  }

  public async getGamesForPlatform(platformId: PlatformId): Promise<Game[]> {
    return this.backend.getGames({ platform: platformId });
  }

  public async toggleFavorite(gameId: string): Promise<boolean> {
    return this.backend.toggleFavorite(gameId);
  }

  public async getPlatforms(): Promise<Platform[]> {
    return this.backend.getPlatforms();
  }

  public async scanGames(request?: import('@contracts/backend.types').ScanGamesRequest): Promise<import('@contracts/backend.types').ScanGamesResult> {
    return this.backend.scanGames(request);
  }
}

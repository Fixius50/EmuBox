import type { Game, PlatformId } from '@contracts/game.types';
import type { ScanGamesRequest, ScanGamesResult } from '@contracts/backend.types';

export interface GameFile {
  path: string;
  filename: string;
  extension: string;
  sizeBytes: number;
  hashMd5?: string;
  platform: PlatformId;
}

/**
 * GameScannerService: Discovers and parses ROMs from storage locations without UI coupling.
 */
export class GameScannerService {
  private supportedExtensions: Record<string, PlatformId> = {
    '.sfc': 'snes',
    '.smc': 'snes',
    '.cue': 'ps1',
    '.bin': 'ps1',
    '.chd': 'ps1',
    '.iso': 'ps2',
    '.z64': 'n64',
    '.n64': 'n64',
    '.v64': 'n64',
    '.md': 'genesis',
    '.gen': 'genesis',
    '.gba': 'gba',
    '.gdi': 'dreamcast',
    '.cdi': 'dreamcast',
    '.zip': 'arcade'
  };

  public identifyPlatformByExtension(filename: string): PlatformId | null {
    const extMatch = filename.match(/\.[0-9a-z]+$/i);
    if (!extMatch) return null;
    const ext = extMatch[0].toLowerCase();
    return this.supportedExtensions[ext] || null;
  }

  public async scanDirectory(request?: ScanGamesRequest, existingGames: Game[] = []): Promise<{ games: Game[]; result: ScanGamesResult }> {
    // In-memory local metadata resolution simulation
    const scanned = existingGames.length;
    return {
      games: existingGames,
      result: {
        scannedCount: scanned,
        addedCount: 0,
        updatedCount: scanned,
        errors: []
      }
    };
  }
}

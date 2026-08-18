import type { Game, Platform, Emulator, SystemSettings } from '@contracts/game.types';
import type { IEmuBoxBackend, LaunchResult } from '@contracts/backend.types';

export class MockBackendService implements IEmuBoxBackend {
  private games: Game[] = [];
  private platforms: Platform[] = [
    { id: 'snes', name: 'Super Nintendo Entertainment System', shortName: 'SNES', manufacturer: 'Nintendo', generation: 4, releaseYear: 1990, color: '#e52521', icon: 'snes', defaultEmulatorId: 'snes9x' },
    { id: 'ps1', name: 'Sony PlayStation', shortName: 'PS1', manufacturer: 'Sony', generation: 5, releaseYear: 1994, color: '#006FCD', icon: 'ps1', defaultEmulatorId: 'duckstation' },
    { id: 'ps2', name: 'Sony PlayStation 2', shortName: 'PS2', manufacturer: 'Sony', generation: 6, releaseYear: 2000, color: '#003791', icon: 'ps2', defaultEmulatorId: 'pcsx2' },
    { id: 'n64', name: 'Nintendo 64', shortName: 'N64', manufacturer: 'Nintendo', generation: 5, releaseYear: 1996, color: '#e52521', icon: 'n64', defaultEmulatorId: 'mupen64plus' },
    { id: 'genesis', name: 'Sega Genesis / Mega Drive', shortName: 'Genesis', manufacturer: 'Sega', generation: 4, releaseYear: 1988, color: '#0088cc', icon: 'genesis', defaultEmulatorId: 'genesis_plus_gx' },
    { id: 'gba', name: 'Game Boy Advance', shortName: 'GBA', manufacturer: 'Nintendo', generation: 6, releaseYear: 2001, color: '#682a8b', icon: 'gba', defaultEmulatorId: 'mgba' },
    { id: 'dreamcast', name: 'Sega Dreamcast', shortName: 'Dreamcast', manufacturer: 'Sega', generation: 6, releaseYear: 1998, color: '#ff6600', icon: 'dreamcast', defaultEmulatorId: 'flycast' },
    { id: 'arcade', name: 'Arcade (MAME / FBNeo)', shortName: 'Arcade', manufacturer: 'Various', generation: 0, releaseYear: 1980, color: '#f59e0b', icon: 'arcade', defaultEmulatorId: 'fbneo' }
  ];

  private emulators: Emulator[] = [
    { id: 'snes9x', name: 'Snes9x (Libretro Core)', version: '1.62.3', supportedPlatforms: ['snes'], coreType: 'libretro', status: 'active', executable: '/usr/bin/retroarch', arguments: ['-L', '/usr/lib/libretro/snes9x_libretro.so'] },
    { id: 'duckstation', name: 'DuckStation (Vulkan - Standalone)', version: '0.1-6824', supportedPlatforms: ['ps1'], coreType: 'standalone', status: 'active', executable: '/usr/bin/duckstation-qt', arguments: ['-batch', '-fullscreen'] },
    { id: 'pcsx2', name: 'PCSX2 (AppImage / Standalone)', version: '2.0.2', supportedPlatforms: ['ps2'], coreType: 'standalone', status: 'active', executable: '/usr/bin/pcsx2-qt', arguments: ['-fullscreen', '-batch'] },
    { id: 'mupen64plus', name: 'Mupen64Plus-Next (GLideN64)', version: '2.5.9', supportedPlatforms: ['n64'], coreType: 'libretro', status: 'active', executable: '/usr/bin/retroarch', arguments: ['-L', '/usr/lib/libretro/mupen64plus_next_libretro.so'] },
    { id: 'genesis_plus_gx', name: 'Genesis Plus GX (Libretro Core)', version: '1.7.4', supportedPlatforms: ['genesis'], coreType: 'libretro', status: 'active', executable: '/usr/bin/retroarch', arguments: ['-L', '/usr/lib/libretro/genesis_plus_gx_libretro.so'] },
    { id: 'mgba', name: 'mGBA (Vulkan - Standalone)', version: '0.10.3', supportedPlatforms: ['gba'], coreType: 'standalone', status: 'active', executable: '/usr/bin/mgba-qt', arguments: ['-f'] },
    { id: 'flycast', name: 'Flycast (Vulkan Direct)', version: '2.4', supportedPlatforms: ['dreamcast'], coreType: 'standalone', status: 'active', executable: '/usr/bin/flycast', arguments: [] },
    { id: 'fbneo', name: 'FinalBurn Neo (Libretro Core)', version: '1.0.0.3', supportedPlatforms: ['arcade'], coreType: 'libretro', status: 'active', executable: '/usr/bin/retroarch', arguments: ['-L', '/usr/lib/libretro/fbneo_libretro.so'] }
  ];

  private settings: SystemSettings = {
    display: {
      resolution: '1920x1080',
      refreshRate: 60,
      vsync: true,
      fullscreen: true,
      crtShader: 'scanlines'
    },
    audio: {
      masterVolume: 85,
      uiSoundEffects: true,
      backgroundMusic: false,
      audioLatencyMs: 16
    },
    gamepad: {
      deadzone: 0.35,
      vibration: true,
      swapSouthEastButtons: false
    },
    library: {
      datasetLimit: 10000,
      showMissingCovers: true,
      defaultPlatform: 'snes'
    }
  };

  constructor(initialGames?: Game[]) {
    if (initialGames) {
      this.games = initialGames;
    }
  }

  public setGames(games: Game[]): void {
    this.games = games;
  }

  public async getGames(filter?: { platform?: string; search?: string; favorite?: boolean; limit?: number }): Promise<Game[]> {
    let result = this.games;

    if (filter?.platform && filter.platform !== 'all') {
      result = result.filter(g => g.platform === filter.platform);
    }

    if (filter?.search) {
      const q = filter.search.toLowerCase();
      result = result.filter(g => g.title.toLowerCase().includes(q) || g.genre.toLowerCase().includes(q));
    }

    if (filter?.favorite) {
      result = result.filter(g => g.favorite);
    }

    if (filter?.limit && filter.limit > 0) {
      result = result.slice(0, filter.limit);
    }

    return result;
  }

  public async getGameById(id: string): Promise<Game | null> {
    return this.games.find(g => g.id === id) || null;
  }

  public async getPlatforms(): Promise<Platform[]> {
    return this.platforms;
  }

  public async getEmulators(): Promise<Emulator[]> {
    return this.emulators;
  }

  public async getSettings(): Promise<SystemSettings> {
    return JSON.parse(JSON.stringify(this.settings));
  }

  public async saveSettings(settings: SystemSettings): Promise<boolean> {
    this.settings = JSON.parse(JSON.stringify(settings));
    return true;
  }

  public async toggleFavorite(gameId: string): Promise<boolean> {
    const game = this.games.find(g => g.id === gameId);
    if (game) {
      game.favorite = !game.favorite;
      return game.favorite;
    }
    return false;
  }

  public async launchGame(gameId: string, emulatorId: string): Promise<LaunchResult> {
    const game = await this.getGameById(gameId);
    const emu = this.emulators.find(e => e.id === emulatorId);

    if (!game || !emu) {
      return { success: false, message: 'Juego o emulador no encontrado' };
    }

    const simPid = Math.floor(Math.random() * 60000) + 10000;
    return {
      success: true,
      message: `Ejecutando ${game.title} con ${emu.name} (Simulado en Arch Linux Direct KMS)`,
      pid: simPid,
      executable: `${emu.executable} ${emu.arguments.join(' ')}`
    };
  }
}

export { MockBackendService as MockBackend };

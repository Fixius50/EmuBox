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

export class MockBackendService implements IEmuBoxBackend {
  private games: Game[] = [];
  private activePid: number | null = null;

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

  private config: EmuBoxConfig = {
    version: 1,
    paths: {
      roms: '~/.local/share/emubox/roms',
      saves: '~/.local/share/emubox/saves',
      states: '~/.local/share/emubox/states',
      screenshots: '~/.local/share/emubox/screenshots',
      covers: '~/.local/share/emubox/covers',
      logs: '~/.local/share/emubox/logs'
    },
    display: {
      resolution: '1920x1080',
      refreshRate: 60,
      fullscreen: true,
      vsync: true,
      gamescopeEnabled: true,
      gamescopeScaling: 'integer',
      crtShader: 'none'
    },
    audio: {
      volume: 85,
      uiSoundEffects: true,
      backgroundMusic: false,
      latencyMs: 16
    },
    input: {
      deadzone: 0.15,
      vibrationEnabled: true,
      swapSouthEastButtons: false,
      pollRateHz: 250
    },
    emulators: {
      defaultMapping: {
        all: '',
        snes: 'snes9x',
        ps1: 'duckstation',
        ps2: 'pcsx2',
        n64: 'mupen64plus',
        genesis: 'genesis_plus_gx',
        gba: 'mgba',
        dreamcast: 'flycast',
        arcade: 'fbneo',
        gamecube: 'dolphin',
        psp: 'ppsspp',
        nds: 'desmume'
      }
    },
    interface: {
      locale: 'es',
      theme: 'dark-cyber',
      animations: true,
      showFpsOverlay: false,
      performanceMode: 'high-performance'
    }
  };

  private settings: SystemSettings = {
    display: {
      resolution: '1920x1080',
      refreshRate: 60,
      vsync: true,
      fullscreen: true,
      crtShader: 'none'
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
    },
    system: {
      performanceMode: 'high-performance',
      vramLimit: '4 GB',
      showFps: false
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

  // System Info & First Run Detection
  public async getSystemInfo(): Promise<SystemInfo> {
    return {
      osName: 'EmuBox OS (Arch Linux Custom Kernel 6.8.9-zen)',
      kernelVersion: '6.8.9-zen1-1-zen',
      architecture: 'x86_64',
      gpuRenderer: 'AMD Radeon RX 7800 XT (RADV Vulkan 1.3.275)',
      cpuModel: 'AMD Ryzen 7 7800X3D 8-Core Processor',
      cpuCores: 16,
      totalMemoryMb: 32150,
      usedMemoryMb: 4120,
      gamescopeAvailable: true,
      activeCompositor: 'gamescope-wayland',
      isPluggedIn: true
    };
  }

  public async runFirstRunDetection(): Promise<FirstRunDetectionResult> {
    return {
      gpuVendor: 'amd',
      gpuRenderer: 'AMD Radeon Graphics (Vulkan 1.3)',
      vulkanSupported: true,
      gamepadsDetected: ['Xbox Wireless Controller (USB)'],
      installedEmulators: ['retroarch', 'duckstation-qt', 'pcsx2-qt'],
      romsDirectoryFound: true,
      configGenerated: true
    };
  }

  // Central Versioned Configuration
  public async getConfig(): Promise<EmuBoxConfig> {
    return JSON.parse(JSON.stringify(this.config));
  }

  public async saveConfig(config: EmuBoxConfig): Promise<void> {
    this.config = JSON.parse(JSON.stringify(config));
  }

  // Settings & Legacy Support
  public async getSettings(): Promise<SystemSettings> {
    return JSON.parse(JSON.stringify(this.settings));
  }

  public async saveSettings(settings: SystemSettings): Promise<boolean> {
    this.settings = JSON.parse(JSON.stringify(settings));
    return true;
  }

  // Games & Library
  public async scanGames(request?: ScanGamesRequest): Promise<ScanGamesResult> {
    return {
      scannedCount: this.games.length,
      addedCount: 0,
      updatedCount: this.games.length,
      errors: []
    };
  }

  public async getGames(filter?: GameFilter): Promise<Game[]> {
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

  public async toggleFavorite(gameId: string): Promise<boolean> {
    const game = this.games.find(g => g.id === gameId);
    if (game) {
      game.favorite = !game.favorite;
      return game.favorite;
    }
    return false;
  }

  // Platforms & Consoles
  public async getPlatforms(): Promise<Platform[]> {
    return this.platforms;
  }

  // Emulators (CRUD)
  public async getEmulators(): Promise<Emulator[]> {
    return this.emulators;
  }

  public async saveEmulator(emulator: Emulator): Promise<void> {
    const idx = this.emulators.findIndex(e => e.id === emulator.id);
    if (idx >= 0) {
      this.emulators[idx] = emulator;
    } else {
      this.emulators.push(emulator);
    }
  }

  public async deleteEmulator(id: string): Promise<void> {
    this.emulators = this.emulators.filter(e => e.id !== id);
  }

  // Game Execution & Lifecycle
  public async launchGame(gameIdOrRequest: string | LaunchGameRequest, emulatorId?: string): Promise<LaunchResult> {
    const gameId = typeof gameIdOrRequest === 'string' ? gameIdOrRequest : gameIdOrRequest.gameId;
    const targetEmuId = typeof gameIdOrRequest === 'string' ? emulatorId : (gameIdOrRequest.emulatorId || emulatorId);

    const game = await this.getGameById(gameId);
    const emu = this.emulators.find(e => e.id === targetEmuId) || this.emulators[0];

    if (!game || !emu) {
      return { success: false, message: 'Juego o emulador no encontrado' };
    }

    const simPid = Math.floor(Math.random() * 60000) + 10000;
    this.activePid = simPid;

    return {
      success: true,
      message: `Ejecutando ${game.title} con ${emu.name} bajo Gamescope (Simulado)`,
      pid: simPid,
      executable: `${emu.executable} ${emu.arguments.join(' ')}`,
      startTime: Date.now()
    };
  }

  public async stopGame(): Promise<void> {
    this.activePid = null;
  }

  // Gamepad Status
  public async getGamepadStatus(): Promise<GamepadStatus> {
    return {
      connectedCount: 1,
      primaryDeviceIndex: 0,
      devices: [
        {
          index: 0,
          id: 'xinput-pad-0',
          name: 'Xbox Wireless Controller',
          connected: true,
          buttonsCount: 16,
          axesCount: 4,
          hasVibration: true,
          isPrimary: true
        }
      ]
    };
  }
}

export { MockBackendService as MockBackend };

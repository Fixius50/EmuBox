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
import { PathService } from '@services/system/path.service';
import { normalizeArchitecture, type Architecture } from '@contracts/architecture.types';
import capabilityDefinitions from '@data/emulator-capabilities.json';

const paths = new PathService();

export class MockBackendService implements IEmuBoxBackend {
  private architecture: Architecture;
  private games: Game[] = [];
  private activeRunningGame: RunningGameInfo | null = null;
  private logs: LogEntry[] = [];
  private downloads: DownloadJob[] = [];

  private updateInfo: UpdateInfo = {
    currentVersion: 'v1.0.0',
    channel: 'stable',
    lastChecked: Date.now() - 3600000,
    hasUpdate: true,
    latestVersion: 'v1.0.1',
    releaseDate: '18 de Agosto, 2026',
    downloadSizeBytes: 48500000,
    releaseNotes: [
      'Optimizacion de latencia en compositor Gamescope para 1080p 60/144Hz.',
      'Deteccion automatica y calibracion de disparadores en DualSense y Xbox Series.',
      'Soporte para actualizacion atomica OTA de la app sin reiniciar Arch Linux.',
      'Proteccion estricta de ROMs, partidas guardadas y BIOS durante la actualizacion.'
    ],
    installedReleases: [
      {
        version: 'v1.0.0',
        releaseDate: '10 de Agosto, 2026',
        installedAt: Date.now() - 604800000,
        commitHash: '8f3a1c2',
        isCurrent: true,
        installPath: '/opt/emubox/releases/v1.0.0'
      },
      {
        version: 'v0.9.9',
        releaseDate: '1 de Agosto, 2026',
        installedAt: Date.now() - 1209600000,
        commitHash: '7b29e01',
        isCurrent: false,
        installPath: '/opt/emubox/releases/v0.9.9'
      }
    ]
  };

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
    { id: 'rpcs3', name: 'RPCS3', version: '0.0.34', supportedPlatforms: ['ps3'], coreType: 'standalone', status: 'active', executable: 'rpcs3', arguments: [] },
    { id: 'fbneo', name: 'FinalBurn Neo (Libretro Core)', version: '1.0.0.3', supportedPlatforms: ['arcade'], coreType: 'libretro', status: 'active', executable: '/usr/bin/retroarch', arguments: ['-L', '/usr/lib/libretro/fbneo_libretro.so'] }
  ];

  private associations: Map<string, CompatibilityAssociation[]> = new Map();

  private config: EmuBoxConfig = {
    version: 1,
    paths: {
      roms: paths.getRomsDir(),
      saves: paths.getSavesDir(),
      states: paths.getStatesDir(),
      screenshots: paths.getScreenshotsDir(),
      covers: paths.getCoversDir(),
      logs: paths.getLogsDir()
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
        ps3: 'rpcs3',
        n64: 'mupen64plus',
        genesis: 'genesis_plus_gx',
        gba: 'mgba',
        dreamcast: 'flycast',
        arcade: 'fbneo',
        gamecube: 'dolphin',
        wii: 'dolphin',
        wiiu: 'cemu',
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
    },
    updates: {
      autoUpdate: true,
      channel: 'stable',
      checkOnStartup: true
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
    },
    updates: {
      autoUpdate: true,
      channel: 'stable',
      checkOnStartup: true
    }
  };

  constructor(initialGames?: Game[], architecture: string = 'x86_64') {
    this.architecture = normalizeArchitecture(architecture);
    if (initialGames) {
      this.games = initialGames.map(game => ({ ...game, installed: game.installed ?? false }));
    }
  }

  public setGames(games: Game[]): void {
    this.games = games.map(game => ({ ...game, installed: game.installed ?? false }));
  }

  // 1. Sistema & Hardware Telemetry
  public async getSystemInfo(): Promise<SystemInfo> {
    return {
      osName: this.architecture === 'aarch64' ? 'Arch Linux ARM (mock)' : 'Arch Linux (mock)',
      kernelVersion: '6.8.9-zen1-1-zen',
      architecture: this.architecture,
      kernelArchitecture: this.architecture,
      hostname: 'emubox-console',
      uptimeSeconds: 7200,
      hardware: await this.getHardwareInfo(),
      display: await this.getDisplayInfo(),
      audio: await this.getAudioInfo(),
      isPluggedIn: true
    };
  }

  public async getHardwareInfo(): Promise<HardwareInfo> {
    return {
      gpuVendor: this.architecture === 'aarch64' ? 'arm' : 'generic',
      gpuRenderer: `Mock GPU (${this.architecture})`,
      vulkanDriverVersion: 'mock',
      vulkanSupported: this.architecture !== 'unsupported',
      drmAvailable: true,
      gamescopeAvailable: true,
      recommendedCompositor: this.architecture === 'unsupported' ? 'cage' : 'gamescope',
      deviceModel: 'Mock device',
      cpuModel: `Mock CPU (${this.architecture})`,
      cpuCores: 16,
      cpuArchitecture: this.architecture,
      totalMemoryMb: 32150,
      freeMemoryMb: 28030
    };
  }

  public async getDisplayInfo(): Promise<DisplayInfo> {
    return {
      resolution: '1920x1080',
      width: 1920,
      height: 1080,
      refreshRate: 60,
      devicePixelRatio: 1.0,
      colorDepth: 24,
      hdrSupported: false,
      activeCompositor: 'gamescope',
      gamescopeActive: true
    };
  }

  public async getAudioInfo(): Promise<AudioInfo> {
    return {
      masterVolume: this.settings.audio.masterVolume,
      uiSoundEffects: this.settings.audio.uiSoundEffects,
      backgroundMusic: this.settings.audio.backgroundMusic,
      latencyMs: 16,
      sampleRate: 48000,
      devices: [
        { id: 'default-sink', name: 'PipeWire Low-Latency Audio Sink', isDefault: true, type: 'sink' }
      ]
    };
  }

  public async runFirstRunDetection(): Promise<FirstRunDetectionResult> {
    const hardware = await this.getHardwareInfo();
    return {
      gpuVendor: hardware.gpuVendor,
      gpuRenderer: hardware.gpuRenderer,
      vulkanSupported: hardware.vulkanSupported ?? false,
      gamepadsDetected: ['Xbox Wireless Controller (USB)'],
      installedEmulators: (await this.getEmulators()).filter(emulator => emulator.compatibility?.status === 'supported').map(emulator => emulator.id),
      romsDirectoryFound: true,
      configGenerated: true
    };
  }

  // 2. Configuración
  public async getConfig(): Promise<EmuBoxConfig> {
    return JSON.parse(JSON.stringify(this.config));
  }

  public async saveConfig(config: EmuBoxConfig): Promise<void> {
    this.config = JSON.parse(JSON.stringify(config));
  }

  public async getSettings(): Promise<SystemSettings> {
    return JSON.parse(JSON.stringify(this.settings));
  }

  public async saveSettings(settings: SystemSettings): Promise<boolean> {
    this.settings = JSON.parse(JSON.stringify(settings));
    return true;
  }

  // 3. Biblioteca & Juegos
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

  public async getGame(id: string): Promise<Game | null> {
    return this.games.find(g => g.id === id) || null;
  }

  public async getGameById(id: string): Promise<Game | null> {
    return this.getGame(id);
  }

  public async scanGames(_request?: ScanGamesRequest): Promise<ScanGamesResult> {
    return {
      scannedCount: this.games.length,
      addedCount: 0,
      updatedCount: this.games.length,
      removedCount: 0,
      totalCount: this.games.length,
      errors: []
    };
  }

  public async getPlatforms(): Promise<Platform[]> {
    return this.platforms;
  }

  public async toggleFavorite(gameId: string): Promise<boolean> {
    const game = this.games.find(g => g.id === gameId);
    if (game) {
      game.favorite = !game.favorite;
      return game.favorite;
    }
    return false;
  }

  // 4. Emuladores (CRUD)
  public async getEmulators(): Promise<Emulator[]> {
    const definitions = capabilityDefinitions as Record<string, { architectures: Architecture[]; requirements?: Emulator['requirements'] }>;
    return this.emulators.map(emulator => {
      const definition = definitions[emulator.id.replaceAll('_', '-')];
      const architectures = emulator.architectures ?? definition?.architectures ?? [];
      const compatible = architectures.includes(this.architecture);
      const installed = emulator.status === 'active' && Boolean(emulator.executable);
      return {
        ...emulator, architectures, requirements: emulator.requirements ?? definition?.requirements ?? {},
        compatibility: {
          status: !compatible ? 'unsupported_architecture' : installed ? 'supported' : 'not_installed',
          reason: !compatible ? `${emulator.name} no admite ${this.architecture}` : installed ? '' : `${emulator.name} no esta instalado`,
          hostArchitecture: this.architecture,
          binaryArchitecture: installed && compatible ? this.architecture : null
        }
      };
    });
  }

  public async getEmulator(id: string): Promise<Emulator | null> {
    return (await this.getEmulators()).find(e => e.id === id) || null;
  }

  public async scanEmulators(): Promise<Emulator[]> {
    return this.getEmulators();
  }

  public async getEmulatorStatus(id: string): Promise<'active' | 'inactive' | 'missing_bios'> {
    const emu = await this.getEmulator(id);
    return emu ? emu.status : 'inactive';
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

  // 4.1 Asociaciones Juego <-> Emulador
  public async getGameAssociations(gameId: string): Promise<CompatibilityAssociation[]> {
    const list = this.associations.get(gameId) || [];
    return [...list].sort((a, b) => {
      if (a.isDefault && !b.isDefault) return -1;
      if (!a.isDefault && b.isDefault) return 1;
      return b.priority - a.priority;
    });
  }

  public async setGameAssociation(association: CompatibilityAssociation): Promise<void> {
    const list = this.associations.get(association.gameId) || [];
    const filtered = list.filter(a => a.emulatorId !== association.emulatorId);
    if (association.isDefault) {
      filtered.forEach(a => a.isDefault = false);
    }
    filtered.push(association);
    this.associations.set(association.gameId, filtered);
  }

  public async removeGameAssociation(gameId: string, emulatorId: string): Promise<void> {
    const list = this.associations.get(gameId) || [];
    const filtered = list.filter(a => a.emulatorId !== emulatorId);
    this.associations.set(gameId, filtered);
  }

  // 5. Ejecución & Procesos
  public async launchGame(gameIdOrRequest: string | LaunchGameRequest, emulatorId?: string): Promise<LaunchResult> {
    const gameId = typeof gameIdOrRequest === 'string' ? gameIdOrRequest : gameIdOrRequest.gameId;
    const targetEmuId = typeof gameIdOrRequest === 'string' ? emulatorId : (gameIdOrRequest.emulatorId || emulatorId);

    const game = await this.getGame(gameId);
    const emulators = await this.getEmulators();
    const emu = targetEmuId ? emulators.find(emulator => emulator.id === targetEmuId)
      : emulators.find(emulator => emulator.supportedPlatforms.includes(game?.platform ?? 'all') && emulator.compatibility?.status === 'supported');

    if (!game || !emu) {
      return { success: false, message: 'Juego o emulador no encontrado' };
    }
    if (emu.compatibility?.status !== 'supported') {
      return { success: false, message: emu.compatibility?.reason ?? 'Emulador no disponible' };
    }

    const simPid = Math.floor(Math.random() * 60000) + 10000;
    this.activeRunningGame = {
      pid: simPid,
      gameId: game.id,
      gameTitle: game.title,
      platformId: game.platform,
      emulatorId: emu.id,
      emulatorName: emu.name,
      executable: emu.executable,
      arguments: emu.arguments,
      startTime: Date.now(),
      cpuPercent: 12.5,
      memoryMb: 450,
      status: 'running'
    };

    return {
      success: true,
      message: `Ejecutando ${game.title} con ${emu.name} bajo Gamescope (Simulado)`,
      pid: simPid,
      executable: `${emu.executable} ${emu.arguments.join(' ')}`,
      startTime: Date.now()
    };
  }

  public async stopGame(): Promise<void> {
    this.activeRunningGame = null;
  }

  public async isGameRunning(): Promise<boolean> {
    return this.activeRunningGame !== null;
  }

  public async getRunningGame(): Promise<RunningGameInfo | null> {
    return this.activeRunningGame;
  }

  public async getProcessStatus(): Promise<ProcessStatus> {
    return {
      hasActiveGame: this.activeRunningGame !== null,
      runningGame: this.activeRunningGame,
      activeChildPids: this.activeRunningGame ? [this.activeRunningGame.pid] : []
    };
  }

  public async killProcess(pid: number): Promise<boolean> {
    if (this.activeRunningGame && this.activeRunningGame.pid === pid) {
      this.activeRunningGame = null;
      return true;
    }
    return false;
  }

  // 6. Input / Gamepads
  public async getGamepads(): Promise<GamepadDevice[]> {
    return [
      {
        index: 0,
        id: 'xinput-pad-0',
        name: 'Xbox Wireless Controller',
        connected: true,
        vendorId: undefined,
        productId: undefined,
        buttonsCount: 16,
        axesCount: 4,
        hasVibration: true,
        isPrimary: true
      }
    ];
  }

  public async getGamepadStatus(): Promise<GamepadStatus> {
    const pads = await this.getGamepads();
    return {
      connectedCount: pads.length,
      primaryDeviceIndex: 0,
      devices: pads
    };
  }

  // 7. Sistema Operativo & Energía
  public async shutdown(): Promise<void> {
    console.log('[MockBackend] Shutdown request simulated');
  }

  public async restart(): Promise<void> {
    console.log('[MockBackend] Restart request simulated');
  }

  public async sleep(): Promise<void> {
    console.log('[MockBackend] Sleep request simulated');
  }

  public async logout(): Promise<void> {
    console.log('[MockBackend] Logout request simulated');
  }

  public async restartAppSession(): Promise<void> {
    console.log('[MockBackend] EmuBox session restart simulated (Arch Linux remains untouched)');
  }

  public async exitToLinuxShell(): Promise<void> {
    console.log('[MockBackend] Exit to Linux Shell simulated');
  }

  // 8. Almacenamiento & XDG
  public async getStorageInfo(): Promise<StorageInfo> {
    return {
      drives: [
        {
          id: 'nvme0n1p2',
          name: 'EmuBox System SSD',
          mountPoint: '/',
          filesystem: 'ext4',
          totalBytes: 512000000000,
          availableBytes: 384000000000,
          usedBytes: 128000000000,
          isRemovable: false,
          isSystemDrive: true
        }
      ],
      locations: await this.getStorageLocations(),
      totalGamesStorageBytes: 120000000000,
      totalSavesStorageBytes: 450000000
    };
  }

  public async getStorageLocations(): Promise<Record<string, StorageLocation>> {
    return {
      roms: { id: 'roms', label: 'ROMs Directory', path: paths.getRomsDir(), totalFiles: this.games.length, totalBytes: 120000000000, accessible: true, isWritable: true },
      saves: { id: 'saves', label: 'Saves Directory', path: paths.getSavesDir(), totalFiles: 45, totalBytes: 450000000, accessible: true, isWritable: true },
      states: { id: 'states', label: 'States Directory', path: paths.getStatesDir(), totalFiles: 20, totalBytes: 250000000, accessible: true, isWritable: true },
      screenshots: { id: 'screenshots', label: 'Screenshots Directory', path: paths.getScreenshotsDir(), totalFiles: 12, totalBytes: 15000000, accessible: true, isWritable: true },
      covers: { id: 'covers', label: 'Covers Cache', path: paths.getCoversDir(), totalFiles: 10000, totalBytes: 500000000, accessible: true, isWritable: true },
      bios: { id: 'bios', label: 'BIOS Directory', path: paths.getBiosDir(), totalFiles: 5, totalBytes: 35000000, accessible: true, isWritable: true },
      logs: { id: 'logs', label: 'Logs Directory', path: paths.getLogsDir(), totalFiles: 3, totalBytes: 120000, accessible: true, isWritable: true },
      cache: { id: 'cache', label: 'Vulkan Shaders Cache', path: paths.getShadersDir(), totalFiles: 50, totalBytes: 80000000, accessible: true, isWritable: true }
    };
  }

  // 9. Diagnóstico & Logs
  public async getSystemLogs(limit: number = 50): Promise<LogEntry[]> {
    return this.logs.slice(-limit);
  }

  public async getEmuBoxLogs(limit: number = 50): Promise<LogEntry[]> {
    return this.logs.filter(l => l.source === 'frontend' || l.source === 'tauri').slice(-limit);
  }

  public async getDiagnostics(): Promise<DiagnosticReport> {
    const hardware = await this.getHardwareInfo();
    const emulators = await this.getEmulators();
    const installed = emulators.filter(emulator => emulator.compatibility?.status === 'supported').length;
    return {
      generatedAt: Date.now(),
      osInfo: 'EmuBox OS 1.0 (Arch Linux)',
      kernelVersion: '6.8.9-zen1-1-zen',
      architecture: this.architecture,
      gpuAdapter: hardware.gpuRenderer,
      vulkanReady: hardware.vulkanSupported ?? false,
      gamescopeReady: hardware.recommendedCompositor === 'gamescope',
      pipewireReady: true,
      storageMounted: true,
      emulatorsInstalledCount: installed,
      emulatorsMissingCount: emulators.length - installed,
      connectedGamepadsCount: 1,
      recentErrors: this.logs.filter(l => l.level === 'error'),
      rawSummaryText: `Mock diagnostics: architecture=${this.architecture}, renderer=${hardware.gpuRenderer}`
    };
  }

  // 10. BIOS Scanner
  public async getBiosRequirements(): Promise<BiosStatus> {
    return {
      totalRequired: 2,
      totalFound: 2,
      missingRequiredCount: 0,
      platforms: {
        ps1: {
          platformId: 'ps1',
          platformName: 'Sony PlayStation',
          emulatorId: 'duckstation',
          allRequiredPresent: true,
          biosFiles: [{ filename: 'scph1001.bin', description: 'US PlayStation BIOS v4.1', state: 'found_valid' }]
        },
        ps2: {
          platformId: 'ps2',
          platformName: 'Sony PlayStation 2',
          emulatorId: 'pcsx2',
          allRequiredPresent: true,
          biosFiles: [{ filename: 'SCPH-70012.bin', description: 'PlayStation 2 v12 BIOS', state: 'found_valid' }]
        }
      }
    };
  }

  public async scanBios(): Promise<BiosStatus> {
    return this.getBiosRequirements();
  }

  // 11. Actualización OTA & Mantenimiento Desacoplado
  public async getUpdateInfo(): Promise<UpdateInfo> {
    return JSON.parse(JSON.stringify(this.updateInfo));
  }

  public async checkForUpdates(channel: UpdateChannel = 'stable'): Promise<UpdateCheckResult> {
    this.updateInfo.channel = channel;
    this.updateInfo.lastChecked = Date.now();
    this.updateInfo.hasUpdate = true;
    this.updateInfo.latestVersion = 'v1.0.1';
    return {
      updateAvailable: true,
      currentVersion: this.updateInfo.currentVersion,
      targetVersion: 'v1.0.1',
      releaseDate: '18 de Agosto, 2026',
      downloadUrl: 'https://github.com/Fixius50/EmuBox/releases/download/v1.0.1/emubox-v1.0.1-x86_64.tar.gz',
      checksumSha256: 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
      downloadSizeBytes: 48500000,
      releaseNotes: this.updateInfo.releaseNotes || []
    };
  }

  public async applyUpdate(targetVersion: string = 'v1.0.1'): Promise<UpdateProgress> {
    this.updateInfo.installedReleases.forEach(r => { r.isCurrent = false; });
    this.updateInfo.installedReleases.unshift({
      version: targetVersion,
      releaseDate: '18 de Agosto, 2026',
      installedAt: Date.now(),
      commitHash: '9a4b3d7',
      isCurrent: true,
      installPath: `/opt/emubox/releases/${targetVersion}`
    });
    this.updateInfo.currentVersion = targetVersion;
    this.updateInfo.hasUpdate = false;

    return {
      stage: 'ready_to_restart',
      percent: 100,
      bytesDownloaded: 48500000,
      totalBytes: 48500000,
      message: `Actualización ${targetVersion} instalada con éxito en /opt/emubox/releases/${targetVersion}. Enlace atómico /opt/emubox/current actualizado.`
    };
  }

  public async rollbackToVersion(version: string): Promise<RollbackResult> {
    const target = this.updateInfo.installedReleases.find(r => r.version === version);
    if (!target) {
      return { success: false, restoredVersion: this.updateInfo.currentVersion, message: `Versión ${version} no encontrada en el historial local.` };
    }
    this.updateInfo.installedReleases.forEach(r => {
      r.isCurrent = r.version === version;
    });
    this.updateInfo.currentVersion = version;
    this.updateInfo.hasUpdate = true;

    return {
      success: true,
      restoredVersion: version,
      message: `Rollback completado con éxito. /opt/emubox/current apunta ahora a ${target.installPath}.`
    };
  }

  public async executeCommand(cmd: string): Promise<string> {
    const trimmed = cmd.trim();
    if (trimmed === 'ip addr' || trimmed === 'ip a' || trimmed === 'hostname -I') {
      return '127.0.0.1 (lo)\n192.168.1.150/24 (enp0s3)\ninet 192.168.1.150 brd 192.168.1.255 scope global dynamic enp0s3';
    }
    if (trimmed.includes('sshd') || trimmed.includes('ssh')) {
      return '● sshd.service - OpenSSH Daemon\n     Loaded: loaded (/usr/lib/systemd/system/sshd.service; enabled)\n     Active: active (running) on port 22';
    }
    return `[Mock Terminal] Comando simulado ejecutado: ${cmd}\nResultado: OK (Simulación en navegador)`;
  }

  public async createDownloadSource(source: DownloadSource): Promise<DownloadSource> {
    if (source.sourceType !== 'http') throw new Error('Solo HTTP está disponible en el modo local');
    return source;
  }

  public async createDownloadJob(request: CreateDownloadRequest): Promise<DownloadJob> {
    const job: DownloadJob = {
      id: `download-${Date.now()}`,
      gameId: request.gameId,
      sourceId: request.source.id,
      platform: request.platform,
      destinationPath: `${paths.getRomsDir(request.platform)}/download.bin`,
      status: 'queued',
      progress: 0,
      downloadedBytes: 0,
      totalBytes: request.source.sizeBytes,
      speedBytesPerSecond: 0,
    };
    this.downloads.push(job);
    return job;
  }

  public async getDownloadJobs(): Promise<DownloadJob[]> {
    return this.downloads;
  }

  private updateDownload(id: string, status: DownloadJob['status']): DownloadJob {
    const job = this.downloads.find((download) => download.id === id);
    if (!job) throw new Error(`Descarga no encontrada: ${id}`);
    job.status = status;
    return job;
  }

  public async startDownload(id: string): Promise<DownloadJob> { return this.updateDownload(id, 'downloading'); }
  public async pauseDownload(id: string): Promise<DownloadJob> { return this.updateDownload(id, 'paused'); }
  public async resumeDownload(id: string): Promise<DownloadJob> { return this.updateDownload(id, 'downloading'); }
  public async cancelDownload(id: string): Promise<DownloadJob> { return this.updateDownload(id, 'cancelled'); }

  public async downloadGame(gameId: string): Promise<DownloadJob> {
    const game = this.games.find(g => g.id === gameId);
    if (game) game.installed = true;
    const job: DownloadJob = {
      id: `download-${gameId}`,
      gameId,
      sourceId: `source-${gameId}`,
      platform: game?.platform || 'all',
      destinationPath: `${paths.getRomsDir(game?.platform)}/${gameId}`,
      status: 'completed',
      progress: 100,
      downloadedBytes: 0,
      totalBytes: 0,
      speedBytesPerSecond: 0,
    };
    this.downloads.push(job);
    return job;
  }

  public async importDownloadsFromJson(jsonContent: string): Promise<DownloadJob[]> {
    const parsed = typeof jsonContent === 'string' ? JSON.parse(jsonContent) : jsonContent;
    const newJobs: DownloadJob[] = [];

    const downloads = parsed.downloads || (Array.isArray(parsed) && parsed[0]?.uris ? parsed : undefined);
    if (downloads && Array.isArray(downloads)) {
      for (const item of downloads) {
        const title = item.title?.trim();
        if (!title) continue;
        const uris: string[] = Array.isArray(item.uris) ? item.uris : [];
        if (uris.length === 0) continue;

        const platform = inferMockPlatform(title, uris, item.platform, parsed.platform, parsed.name);
        const totalBytes = parseMockFileSize(item.fileSize);
        const primaryUri = uris.find((u: string) => u.startsWith('http://') || u.startsWith('https://')) || uris[0];

        const gameId = item.gameId || `download-${platform}-${title.toLowerCase().replace(/[^a-z0-9]+/g, '-')}`;
        const sourceId = item.sourceId || `source-${platform}-${primaryUri.toLowerCase().replace(/[^a-z0-9]+/g, '-')}`;

        let game = this.games.find(g => g.id === gameId);
        if (!game) {
          game = {
            id: gameId,
            title,
            platform: (platform as any),
            platformName: platform.toUpperCase(),
            releaseYear: item.uploadDate ? parseInt(item.uploadDate.slice(0, 4), 10) || 2020 : 2020,
            genre: item.genre || 'Desconocido',
            developer: item.developer || 'Desconocido',
            publisher: item.publisher || 'Desconocido',
            rating: item.rating || 4.5,
            playTimeMinutes: 0,
            favorite: false,
            coverImage: item.coverImage || '',
            backdropImage: item.backdropImage,
            description: item.description || '',
            installed: false,
          };
          this.games.push(game);
        }

        const job: DownloadJob = {
          id: `download-${Date.now()}-${newJobs.length}`,
          gameId,
          sourceId,
          platform,
          destinationPath: `${paths.getRomsDir(platform)}/${title.replace(/[\/\\]/g, '_')}.bin`,
          status: 'queued',
          progress: 0,
          downloadedBytes: 0,
          totalBytes,
          speedBytesPerSecond: 0,
        };
        this.downloads.push(job);
        newJobs.push(job);
      }
      return newJobs;
    }

    const games = parsed.games || (Array.isArray(parsed) ? parsed : []);
    for (const entry of games) {
      const platform = entry.platform || 'pc';
      const name = entry.name || entry.title || 'Untitled';
      const uri = entry.url || entry.uri || '';
      if (!uri) continue;
      const gameId = entry.gameId || `download-${platform}-${name.toLowerCase().replace(/[^a-z0-9]+/g, '-')}`;

      let game = this.games.find(g => g.id === gameId);
      if (!game) {
        game = {
          id: gameId,
          title: name,
          platform: (platform as any),
          platformName: platform.toUpperCase(),
          releaseYear: entry.releaseYear || 2020,
          genre: entry.genre || 'Desconocido',
          developer: entry.developer || 'Desconocido',
          publisher: entry.publisher || 'Desconocido',
          rating: entry.rating || 4.5,
          playTimeMinutes: 0,
          favorite: false,
          coverImage: entry.coverImage || '',
          description: entry.description || '',
          installed: false,
        };
        this.games.push(game);
      }

      const job: DownloadJob = {
        id: `download-${Date.now()}-${newJobs.length}`,
        gameId,
        sourceId: entry.sourceId || `source-${gameId}`,
        platform,
        destinationPath: `${paths.getRomsDir(platform)}/${name}.bin`,
        status: 'queued',
        progress: 0,
        downloadedBytes: 0,
        totalBytes: entry.sizeBytes || parseMockFileSize(entry.fileSize),
        speedBytesPerSecond: 0,
      };
      this.downloads.push(job);
      newJobs.push(job);
    }
    return newJobs;
  }

  public async importDownloadsFromUrl(url: string): Promise<DownloadJob[]> {
    const res = await fetch(url);
    const json = await res.json();
    return this.importDownloadsFromJson(JSON.stringify(json));
  }

  public async importDownloadLinks(): Promise<DownloadJob[]> {
    return this.downloads;
  }
}

function parseMockFileSize(fileSize?: string | number): number | undefined {
  if (fileSize === undefined || fileSize === null) return undefined;
  if (typeof fileSize === 'number') return fileSize;
  const trimmed = fileSize.trim();
  if (!trimmed) return undefined;
  if (/^\d+$/.test(trimmed)) return parseInt(trimmed, 10);

  const match = trimmed.match(/^([\d.,]+)\s*([A-Za-z]+)?$/);
  if (!match) return undefined;

  const num = parseFloat(match[1].replace(',', '.'));
  if (isNaN(num)) return undefined;

  const unit = (match[2] || '').toUpperCase();
  const multipliers: Record<string, number> = {
    TB: 1024 ** 4,
    TIB: 1024 ** 4,
    GB: 1024 ** 3,
    GIB: 1024 ** 3,
    MB: 1024 ** 2,
    MIB: 1024 ** 2,
    KB: 1024,
    KIB: 1024,
    B: 1,
    BYTES: 1,
    BYTE: 1,
  };

  return Math.round(num * (multipliers[unit] || 1));
}

function inferMockPlatform(title: string, uris: string[], itemPlatform?: string, manifestPlatform?: string, manifestHint?: string): string {
  if (itemPlatform) return itemPlatform.toLowerCase();
  if (manifestPlatform) return manifestPlatform.toLowerCase();

  const titleLower = title.toLowerCase();
  if (titleLower.includes('[pc]') || titleLower.includes('(pc)') || titleLower.includes('steamrip') || titleLower.includes('gog')) return 'pc';
  if (titleLower.includes('[ps3]') || titleLower.includes('(ps3)') || titleLower.includes('ps3') || titleLower.includes('rpcs3')) return 'ps3';
  if (titleLower.includes('[ps2]') || titleLower.includes('(ps2)') || titleLower.includes('pcsx2')) return 'ps2';
  if (titleLower.includes('[ps1]') || titleLower.includes('(ps1)') || titleLower.includes('[psx]') || titleLower.includes('(psx)') || titleLower.includes('duckstation')) return 'ps1';
  if (titleLower.includes('[psp]') || titleLower.includes('(psp)') || titleLower.includes('ppsspp')) return 'psp';
  if (titleLower.includes('[wiiu]') || titleLower.includes('(wiiu)') || titleLower.includes('cemu')) return 'wiiu';
  if (titleLower.includes('[wii]') || titleLower.includes('(wii)')) return 'wii';
  if (titleLower.includes('[gamecube]') || titleLower.includes('(gamecube)') || titleLower.includes('[gcn]') || titleLower.includes('dolphin')) return 'gamecube';
  if (titleLower.includes('[snes]') || titleLower.includes('(snes)') || titleLower.includes('super nintendo')) return 'snes';
  if (titleLower.includes('[gba]') || titleLower.includes('(gba)') || titleLower.includes('game boy advance') || titleLower.includes('mgba')) return 'gba';
  if (titleLower.includes('[n64]') || titleLower.includes('(n64)') || titleLower.includes('nintendo 64')) return 'n64';
  if (titleLower.includes('[nds]') || titleLower.includes('(nds)') || titleLower.includes('nintendo ds') || titleLower.includes('melonds')) return 'nds';
  if (titleLower.includes('[genesis]') || titleLower.includes('(genesis)') || titleLower.includes('megadrive')) return 'genesis';
  if (titleLower.includes('[dreamcast]') || titleLower.includes('(dreamcast)') || titleLower.includes('flycast')) return 'dreamcast';
  if (titleLower.includes('[arcade]') || titleLower.includes('(arcade)') || titleLower.includes('mame')) return 'arcade';

  for (const uri of uris) {
    const u = uri.toLowerCase();
    if (u.includes('.pkg')) return 'ps3';
    if (u.includes('.sfc') || u.includes('.smc')) return 'snes';
    if (u.includes('.gba')) return 'gba';
    if (u.includes('.z64') || u.includes('.n64') || u.includes('.v64')) return 'n64';
    if (u.includes('.nds')) return 'nds';
    if (u.includes('.cdi') || u.includes('.gdi')) return 'dreamcast';
    if (u.includes('.rvz') || u.includes('.gcm')) return 'gamecube';
    if (u.includes('.wua')) return 'wiiu';
    if (u.includes('.pbp')) return 'psp';
    if (u.includes('steamrip') || u.includes('gog') || u.includes('.exe')) return 'pc';
  }

  if (manifestHint) {
    const h = manifestHint.toLowerCase();
    if (h.includes('psx-roms') || h.includes('ps1')) return 'ps1';
    if (h.includes('ps2')) return 'ps2';
    if (h.includes('ps3')) return 'ps3';
    if (h.includes('psp')) return 'psp';
    if (h.includes('snes')) return 'snes';
    if (h.includes('gba')) return 'gba';
    if (h.includes('n64')) return 'n64';
    if (h.includes('nds')) return 'nds';
    if (h.includes('linux') || h.includes('pc') || h.includes('repack')) return 'pc';
    if (h.includes('psx')) return 'ps1';
  }

  return 'pc';
}

export { MockBackendService as MockBackend };

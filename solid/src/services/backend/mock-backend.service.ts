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
  UpdateChannel,
  InstalledReleaseInfo
} from '@contracts/backend.types';

export class MockBackendService implements IEmuBoxBackend {
  private games: Game[] = [];
  private activeRunningGame: RunningGameInfo | null = null;
  private logs: LogEntry[] = [];

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
    { id: 'fbneo', name: 'FinalBurn Neo (Libretro Core)', version: '1.0.0.3', supportedPlatforms: ['arcade'], coreType: 'libretro', status: 'active', executable: '/usr/bin/retroarch', arguments: ['-L', '/usr/lib/libretro/fbneo_libretro.so'] }
  ];

  private associations: Map<string, CompatibilityAssociation[]> = new Map();

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

  constructor(initialGames?: Game[]) {
    if (initialGames) {
      this.games = initialGames;
    }
  }

  public setGames(games: Game[]): void {
    this.games = games;
  }

  // 1. Sistema & Hardware Telemetry
  public async getSystemInfo(): Promise<SystemInfo> {
    return {
      osName: 'EmuBox OS (Arch Linux Custom Kernel 6.8.9-zen)',
      kernelVersion: '6.8.9-zen1-1-zen',
      architecture: 'x86_64',
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
      gpuVendor: 'amd',
      gpuRenderer: 'AMD Radeon RX 7800 XT (RADV Vulkan 1.3.275)',
      vulkanDriverVersion: '24.0.4',
      cpuModel: 'AMD Ryzen 7 7800X3D 8-Core Processor',
      cpuCores: 16,
      cpuArchitecture: 'x86_64',
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

  public async scanGames(request?: ScanGamesRequest): Promise<ScanGamesResult> {
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
    return this.emulators;
  }

  public async getEmulator(id: string): Promise<Emulator | null> {
    return this.emulators.find(e => e.id === id) || null;
  }

  public async scanEmulators(): Promise<Emulator[]> {
    return this.emulators;
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
    const emu = this.emulators.find(e => e.id === targetEmuId) || this.emulators[0];

    if (!game || !emu) {
      return { success: false, message: 'Juego o emulador no encontrado' };
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
      roms: { id: 'roms', label: 'ROMs Directory', path: '~/.local/share/emubox/roms', totalFiles: this.games.length, totalBytes: 120000000000, accessible: true, isWritable: true },
      saves: { id: 'saves', label: 'Saves Directory', path: '~/.local/share/emubox/saves', totalFiles: 45, totalBytes: 450000000, accessible: true, isWritable: true },
      states: { id: 'states', label: 'States Directory', path: '~/.local/share/emubox/states', totalFiles: 20, totalBytes: 250000000, accessible: true, isWritable: true },
      screenshots: { id: 'screenshots', label: 'Screenshots Directory', path: '~/.local/share/emubox/screenshots', totalFiles: 12, totalBytes: 15000000, accessible: true, isWritable: true },
      covers: { id: 'covers', label: 'Covers Cache', path: '~/.local/share/emubox/covers', totalFiles: 10000, totalBytes: 500000000, accessible: true, isWritable: true },
      bios: { id: 'bios', label: 'BIOS Directory', path: '~/.local/share/emubox/bios', totalFiles: 5, totalBytes: 35000000, accessible: true, isWritable: true },
      logs: { id: 'logs', label: 'Logs Directory', path: '~/.local/share/emubox/logs', totalFiles: 3, totalBytes: 120000, accessible: true, isWritable: true },
      cache: { id: 'cache', label: 'Vulkan Shaders Cache', path: '~/.cache/emubox', totalFiles: 50, totalBytes: 80000000, accessible: true, isWritable: true }
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
    return {
      generatedAt: Date.now(),
      osInfo: 'EmuBox OS 1.0 (Arch Linux)',
      kernelVersion: '6.8.9-zen1-1-zen',
      architecture: 'x86_64',
      gpuAdapter: 'AMD Radeon RX 7800 XT (RADV Vulkan 1.3)',
      vulkanReady: true,
      gamescopeReady: true,
      pipewireReady: true,
      storageMounted: true,
      emulatorsInstalledCount: this.emulators.length,
      emulatorsMissingCount: 0,
      connectedGamepadsCount: 1,
      recentErrors: this.logs.filter(l => l.level === 'error'),
      rawSummaryText: 'EmuBox OS Diagnostics: All systems active and compliant.'
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
    const prevVersion = this.updateInfo.currentVersion;
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
}

export { MockBackendService as MockBackend };

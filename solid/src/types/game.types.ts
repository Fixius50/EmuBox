export type PlatformId =
  | 'snes'
  | 'ps1'
  | 'ps2'
  | 'ps3'
  | 'n64'
  | 'genesis'
  | 'gba'
  | 'dreamcast'
  | 'arcade'
  | 'gamecube'
  | 'wii'
  | 'wiiu'
  | 'psp'
  | 'nds'
  | 'pc'
  | 'linux'
  | 'all';

export interface Platform {
  id: PlatformId;
  name: string;
  shortName: string;
  manufacturer: string;
  generation: number;
  releaseYear: number;
  color: string;
  icon: string;
  defaultEmulatorId: string;
}

export interface Game {
  id: string;
  title: string;
  platform: PlatformId;
  platformName: string;
  releaseYear: number;
  genre: string;
  developer: string;
  publisher: string;
  rating: number;
  playTimeMinutes: number;
  favorite: boolean;
  coverImage: string;
  backdropImage?: string;
  description: string;
  romPath?: string;
  installed: boolean;
}

export interface Emulator {
  id: string;
  name: string;
  version: string;
  supportedPlatforms: PlatformId[];
  coreType: 'libretro' | 'standalone';
  status: 'active' | 'inactive' | 'missing_bios';
  executable: string;
  arguments: string[];
  architectures?: import('./architecture.types').Architecture[];
  requirements?: { minCpuCores?: number; minMemoryMb?: number; vulkan?: boolean };
  compatibility?: {
    status: 'supported' | 'unsupported_architecture' | 'not_installed' | 'invalid_binary' | 'requirements_not_met';
    reason: string;
    hostArchitecture: import('./architecture.types').Architecture;
    binaryArchitecture?: string | null;
  };
}

export interface SystemDefinition extends Platform {
  extensions: string[];
  gamesDirectory: string;
}

export interface CompatibilityAssociation {
  gameId: string;
  emulatorId: string;
  priority: number;
  isDefault: boolean;
  customArgs?: string[];
  customConfigPath?: string;
  enabled: boolean;
}

export interface ExecutionTarget {
  game: Game;
  emulator: Emulator;
  command: string;
  args: string[];
  configPath?: string;
}

export type PerformanceMode = 'high-performance' | 'balanced' | 'power-saver' | 'ultra-boost';

export interface SystemSettings {
  display: {
    resolution: '1920x1080' | '3840x2160' | '1280x720' | string;
    refreshRate: 60 | 120 | 144 | 165 | 240 | number;
    vsync: boolean;
    fullscreen: boolean;
    crtShader: 'none' | 'scanlines' | 'curved_crt' | 'phosphor';
  };
  audio: {
    masterVolume: number;
    uiSoundEffects: boolean;
    backgroundMusic: boolean;
    audioLatencyMs: number;
  };
  gamepad: {
    deadzone: number;
    vibration: boolean;
    swapSouthEastButtons: boolean;
  };
  library: {
    datasetLimit: number;
    showMissingCovers: boolean;
    defaultPlatform: PlatformId;
  };
  system?: {
    performanceMode?: PerformanceMode;
    vramLimit?: string;
    showFps?: boolean;
  };
  updates?: {
    autoUpdate: boolean;
    channel: 'stable' | 'beta' | 'nightly';
    checkOnStartup: boolean;
  };
}

/**
 * Single, Central, Versioned EmuBox Configuration Model.
 * Mirrors /etc/emubox/config.json in production Arch Linux installations.
 */
export interface EmuBoxConfig {
  version: number;
  paths: {
    roms: string;
    saves: string;
    states: string;
    screenshots: string;
    covers: string;
    logs: string;
  };
  display: {
    resolution: string;
    refreshRate: number | string;
    fullscreen: boolean;
    vsync: boolean;
    gamescopeEnabled: boolean;
    gamescopeScaling: 'integer' | 'fit' | 'stretch';
    crtShader: 'none' | 'scanlines' | 'curved_crt' | 'phosphor';
  };
  audio: {
    volume: number;
    uiSoundEffects: boolean;
    backgroundMusic: boolean;
    latencyMs: number;
  };
  input: {
    deadzone: number;
    vibrationEnabled: boolean;
    swapSouthEastButtons: boolean;
    pollRateHz: number;
  };
  emulators: {
    defaultMapping: Record<PlatformId, string>;
    customBinariesPath?: string;
  };
  interface: {
    locale: string;
    theme: 'dark-cyber' | 'glassmorphism' | 'pure-oled';
    animations: boolean;
    showFpsOverlay: boolean;
    performanceMode: 'high-performance' | 'balanced' | 'power-saver' | 'ultra-boost';
  };
  updates: {
    autoUpdate: boolean;
    channel: 'stable' | 'beta' | 'nightly';
    checkOnStartup: boolean;
  };
}

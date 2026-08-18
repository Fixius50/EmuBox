export interface HardwareInfo {
  gpuVendor: 'amd' | 'nvidia' | 'intel' | 'generic';
  gpuRenderer: string;
  vulkanDriverVersion?: string;
  cpuModel: string;
  cpuCores: number;
  cpuArchitecture: string;
  totalMemoryMb: number;
  freeMemoryMb: number;
}

export interface DisplayInfo {
  resolution: string;
  width: number;
  height: number;
  refreshRate: number;
  devicePixelRatio: number;
  colorDepth: number;
  hdrSupported: boolean;
  activeCompositor: 'gamescope' | 'wayland' | 'x11' | 'browser';
  gamescopeActive: boolean;
}

export interface AudioDevice {
  id: string;
  name: string;
  isDefault: boolean;
  type: 'sink' | 'source';
}

export interface AudioInfo {
  masterVolume: number;
  uiSoundEffects: boolean;
  backgroundMusic: boolean;
  latencyMs: number;
  sampleRate: number;
  devices: AudioDevice[];
}

export interface SystemInfo {
  osName: string;
  kernelVersion: string;
  architecture: string;
  hostname: string;
  uptimeSeconds: number;
  hardware: HardwareInfo;
  display: DisplayInfo;
  audio: AudioInfo;
  batteryLevelPercent?: number;
  isPluggedIn?: boolean;
}

export type PowerAction = 'shutdown' | 'restart' | 'sleep' | 'logout';

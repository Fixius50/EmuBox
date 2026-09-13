import type { GraphicsBackend, GpuKind, NativeGraphicsEvidence } from './graphics.types';

export interface HardwareInfo {
  graphics?: NativeGraphicsEvidence;
  gpuVendor: 'amd' | 'nvidia' | 'intel' | 'generic' | 'unknown' | 'virtual' | 'mali' | 'broadcom' | 'qualcomm' | 'apple';
  gpuRenderer: string;
  vulkanDriverVersion?: string;
  vulkanSupported?: boolean;
  openglSupported?: boolean;
  openglRenderer?: string | null;
  openglAccelerated?: boolean;
  graphicsAccelerated?: boolean;
  graphicsBackend?: GraphicsBackend;
  gpuKind?: GpuKind;
  isVirtualMachine?: boolean;
  gamescopeReady?: boolean;
  drmAvailable?: boolean;
  gamescopeAvailable?: boolean;
  recommendedCompositor?: 'gamescope' | 'cage' | 'unavailable';
  deviceModel?: string;
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
  refreshRate: number | null;
  devicePixelRatio: number | null;
  colorDepth: number | null;
  hdrSupported: boolean | null;
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
  masterVolume: number | null;
  uiSoundEffects: boolean;
  backgroundMusic: boolean;
  latencyMs: number | null;
  sampleRate: number | null;
  devices: AudioDevice[];
}

export interface SystemInfo {
  osName: string;
  kernelVersion: string;
  architecture: string;
  kernelArchitecture?: string;
  hostname: string;
  uptimeSeconds: number;
  hardware: HardwareInfo;
  display: DisplayInfo | null;
  audio: AudioInfo | null;
  batteryLevelPercent?: number;
  isPluggedIn?: boolean;
}

export type PowerAction = 'shutdown' | 'restart' | 'sleep' | 'logout';

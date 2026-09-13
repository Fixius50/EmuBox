export type RenderPipelineMode = 'accelerated' | 'cpu-compatible' | 'indeterminate';
export type GraphicsBackend = 'vulkan' | 'opengl' | 'webgl' | 'software' | 'auto';
export type GpuKind = 'physical' | 'virtual' | 'software' | 'unknown';
export type DetectionState = 'accelerated' | 'software' | 'indeterminate';

export interface GraphicsDevice {
  id: string;
  cardNodes: string[];
  renderNodes: string[];
  renderIdentifiers: string[];
  pciAddress: string | null;
  driver: string;
  vendor: string;
}

export interface GraphicsObservation {
  api: string;
  state: DetectionState;
  renderer: string;
  deviceId: string | null;
  correlation: 'device_identifier' | 'single_device_inference' | 'unknown';
  reportedDevice: string | null;
}

export interface GraphicsProbe {
  api: string;
  state: DetectionState;
  reason: string;
  exitCode: number | null;
  observations: GraphicsObservation[];
}

export interface NativeGraphicsEvidence {
  detectionState: DetectionState;
  inventoryComplete: boolean;
  inventoryReason: string | null;
  devices: GraphicsDevice[];
  probes: GraphicsProbe[];
  selectedDeviceId: string | null;
  activeDeviceId: string | null;
  backend: GraphicsBackend;
  operationalBackend: GraphicsBackend;
  fallbackReason: string | null;
}

export interface GraphicsCapabilities {
  pipeline: RenderPipelineMode;
  isGpuAccelerated: boolean | null;
  detectionState: DetectionState;
  operationalBackend: GraphicsBackend;
  evidence?: NativeGraphicsEvidence;
  selectedBackend: GraphicsBackend;
  gpuKind: GpuKind;
  renderer: string;
  vendor: string;
  isVirtualMachine: boolean;
  recommendedBlur: boolean;
  probeTimeMs: number;
}

export interface GraphicsDetectorOptions {
  customDocument?: Document;
}

export type RenderPipelineMode = 'accelerated' | 'cpu-compatible';
export type GraphicsBackend = 'vulkan' | 'opengl' | 'webgl' | 'software';
export type GpuKind = 'physical' | 'virtual' | 'software' | 'unknown';

export interface GraphicsCapabilities {
  pipeline: RenderPipelineMode;
  isGpuAccelerated: boolean;
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

export type RenderPipelineMode = 'accelerated' | 'cpu-compatible';

export interface GraphicsCapabilities {
  pipeline: RenderPipelineMode;
  isGpuAccelerated: boolean;
  renderer: string;
  vendor: string;
  isVirtualMachine: boolean;
  recommendedBlur: boolean;
  probeTimeMs: number;
}

export interface GraphicsDetectorOptions {
  forceMode?: RenderPipelineMode;
  customDocument?: Document;
}

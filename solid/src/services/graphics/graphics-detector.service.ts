import type { GraphicsCapabilities, GraphicsDetectorOptions, RenderPipelineMode } from '@contracts/graphics.types';

const SOFTWARE_RENDERER_PATTERNS = [
  'llvmpipe',
  'softpipe',
  'swrast',
  'software',
  'vmware',
  'virtualbox',
  'microsoft basic render',
  'gdi generic',
  'mesa software'
];

export class GraphicsDetectorService {
  private capabilities: GraphicsCapabilities | null = null;

  public detect(options?: GraphicsDetectorOptions): GraphicsCapabilities {
    const startTime = performance.now();

    if (options?.forceMode) {
      const isAcc = options.forceMode === 'accelerated';
      this.capabilities = {
        pipeline: options.forceMode,
        isGpuAccelerated: isAcc,
        renderer: options.forceMode === 'accelerated' ? 'Forced Hardware GPU' : 'Forced Software Fallback',
        vendor: 'Forced',
        isVirtualMachine: !isAcc,
        recommendedBlur: isAcc,
        probeTimeMs: Math.round((performance.now() - startTime) * 100) / 100
      };
      this.applyToDocument(options.customDocument);
      return this.capabilities;
    }

    let renderer = 'Generic / Unknown';
    let vendor = 'Unknown';
    let isAccelerated = true;
    let isVm = false;

    if (typeof window !== 'undefined' && typeof document !== 'undefined') {
      try {
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl') || canvas.getContext('experimental-webgl');

        if (gl && 'getExtension' in gl) {
          const webgl = gl as WebGLRenderingContext;
          const debugInfo = webgl.getExtension('WEBGL_debug_renderer_info');
          if (debugInfo) {
            renderer = webgl.getParameter(debugInfo.UNMASKED_RENDERER_WEBGL) || renderer;
            vendor = webgl.getParameter(debugInfo.UNMASKED_VENDOR_WEBGL) || vendor;
          } else {
            renderer = webgl.getParameter(webgl.RENDERER) || renderer;
            vendor = webgl.getParameter(webgl.VENDOR) || vendor;
          }
        } else {
          isAccelerated = false;
        }
      } catch {
        isAccelerated = false;
      }
    }

    const rendererLower = renderer.toLowerCase();
    const isKnownSoftware = SOFTWARE_RENDERER_PATTERNS.some((pattern) => rendererLower.includes(pattern));

    if (isKnownSoftware) {
      isAccelerated = false;
      isVm = rendererLower.includes('vmware') || rendererLower.includes('virtualbox');
    }

    const pipeline: RenderPipelineMode = isAccelerated ? 'accelerated' : 'cpu-compatible';

    this.capabilities = {
      pipeline,
      isGpuAccelerated: isAccelerated,
      renderer,
      vendor,
      isVirtualMachine: isVm,
      recommendedBlur: isAccelerated,
      probeTimeMs: Math.round((performance.now() - startTime) * 100) / 100
    };

    this.applyToDocument(options?.customDocument);
    return this.capabilities;
  }

  public getCapabilities(): GraphicsCapabilities {
    return this.capabilities ?? this.detect();
  }

  public applyToDocument(customDoc?: Document): void {
    const doc = customDoc || (typeof document !== 'undefined' ? document : null);
    if (!doc?.documentElement || !this.capabilities) {
      return;
    }

    doc.documentElement.setAttribute('data-render-pipeline', this.capabilities.pipeline);
    doc.documentElement.setAttribute('data-blur-mode', this.capabilities.recommendedBlur ? 'hardware' : 'software');
  }
}

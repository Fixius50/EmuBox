import type { GraphicsCapabilities, GraphicsDetectorOptions, RenderPipelineMode } from '@contracts/graphics.types';
import type { HardwareInfo } from '@contracts/system.types';

const SOFTWARE_RENDERER_PATTERNS = [
  'llvmpipe',
  'softpipe',
  'swrast',
  'software',
  'microsoft basic render',
  'gdi generic',
  'mesa software'
];

export class GraphicsDetectorService {
  private capabilities: GraphicsCapabilities | null = null;

  public detectFromHardware(hardware: HardwareInfo, customDocument?: Document): GraphicsCapabilities {
    const accelerated = hardware.vulkanSupported === true || hardware.openglAccelerated === true;
    this.capabilities = {
      pipeline: accelerated ? 'accelerated' : 'cpu-compatible',
      isGpuAccelerated: accelerated,
      renderer: hardware.gpuRenderer,
      vendor: hardware.gpuVendor,
      isVirtualMachine: hardware.gpuVendor === 'virtual',
      recommendedBlur: accelerated,
      probeTimeMs: 0,
    };
    this.applyToDocument(customDocument);
    return this.capabilities;
  }

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
    isVm = /vmware|virtualbox|svga3d|virgl/.test(`${rendererLower} ${vendor.toLowerCase()}`);
    const isKnownSoftware = SOFTWARE_RENDERER_PATTERNS.some((pattern) => rendererLower.includes(pattern));

    if (isKnownSoftware) {
      isAccelerated = false;
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

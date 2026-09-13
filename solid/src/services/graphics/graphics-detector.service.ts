import type { GraphicsCapabilities, GraphicsDetectorOptions, RenderPipelineMode } from '@contracts/graphics.types';
import type { HardwareInfo } from '@contracts/system.types';

const SOFTWARE_RENDERER_PATTERNS = [
  'llvmpipe',
  'softpipe',
  'swrast',
  'swiftshader',
  'lavapipe',
  'software',
  'microsoft basic render',
  'gdi generic',
  'mesa software'
];

export class GraphicsDetectorService {
  private capabilities: GraphicsCapabilities | null = null;

  public detectFromHardware(hardware: HardwareInfo, customDocument?: Document): GraphicsCapabilities {
    const software = SOFTWARE_RENDERER_PATTERNS.some(pattern => hardware.gpuRenderer.toLowerCase().includes(pattern));
    const accelerated = !software && (hardware.graphicsAccelerated ?? (hardware.vulkanSupported === true || hardware.openglAccelerated === true));
    const virtualGpu = /vmware|virtualbox|svga3d|virgl|virtio|venus/.test(`${hardware.gpuRenderer} ${hardware.gpuVendor}`.toLowerCase()) || hardware.gpuVendor === 'virtual';
    this.capabilities = {
      pipeline: accelerated ? 'accelerated' : 'cpu-compatible',
      isGpuAccelerated: accelerated,
      selectedBackend: accelerated ? hardware.graphicsBackend ?? (hardware.openglAccelerated ? 'opengl' : 'vulkan') : 'software',
      gpuKind: accelerated ? hardware.gpuKind ?? (virtualGpu ? 'virtual' : 'physical') : 'software',
      renderer: hardware.gpuRenderer,
      vendor: hardware.gpuVendor,
      isVirtualMachine: hardware.isVirtualMachine ?? virtualGpu,
      recommendedBlur: accelerated,
      probeTimeMs: 0,
    };
    this.applyToDocument(customDocument);
    return this.capabilities;
  }

  public detect(options?: GraphicsDetectorOptions): GraphicsCapabilities {
    const startTime = performance.now();

    let renderer = 'Generic / Unknown';
    let vendor = 'Unknown';
    let isAccelerated = false;
    let isVm = false;

    if (typeof window !== 'undefined' && typeof document !== 'undefined') {
      try {
        const canvas = document.createElement('canvas');
        const gl = canvas.getContext('webgl', { failIfMajorPerformanceCaveat: true });

        if (gl && 'getExtension' in gl) {
          const webgl = gl as WebGLRenderingContext;
          isAccelerated = true;
          const debugInfo = webgl.getExtension('WEBGL_debug_renderer_info');
          if (debugInfo) {
            renderer = webgl.getParameter(debugInfo.UNMASKED_RENDERER_WEBGL) || renderer;
            vendor = webgl.getParameter(debugInfo.UNMASKED_VENDOR_WEBGL) || vendor;
          } else {
            renderer = webgl.getParameter(webgl.RENDERER) || renderer;
            vendor = webgl.getParameter(webgl.VENDOR) || vendor;
          }
          webgl.getExtension('WEBGL_lose_context')?.loseContext();
        } else {
          isAccelerated = false;
        }
      } catch {
        isAccelerated = false;
      }
    }

    const rendererLower = renderer.toLowerCase();
    isVm = /vmware|virtualbox|svga3d|virgl|virtio|venus/.test(`${rendererLower} ${vendor.toLowerCase()}`);
    const isKnownSoftware = SOFTWARE_RENDERER_PATTERNS.some((pattern) => rendererLower.includes(pattern));

    if (isKnownSoftware) {
      isAccelerated = false;
    }

    const pipeline: RenderPipelineMode = isAccelerated ? 'accelerated' : 'cpu-compatible';

    this.capabilities = {
      pipeline,
      isGpuAccelerated: isAccelerated,
      selectedBackend: isAccelerated ? 'webgl' : 'software',
      gpuKind: isAccelerated ? (isVm ? 'virtual' : 'unknown') : 'software',
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
    doc.documentElement.setAttribute('data-render-backend', this.capabilities.selectedBackend);
    doc.documentElement.setAttribute('data-blur-mode', this.capabilities.recommendedBlur ? 'hardware' : 'software');
  }
}

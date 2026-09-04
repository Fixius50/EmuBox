export interface RealHardwareInfo {
  gpuRenderer: string;
  cpuCores: number;
  screenResolution: string;
  screenRefreshRate: number;
  devicePixelRatio: number;
  colorDepth: number;
  memoryEstimate: string;
  platformOS: string;
}

export interface RealGamepadInfo {
  index: number;
  id: string;
  connected: boolean;
  timestamp: number;
  buttonsCount: number;
  axesCount: number;
  hasVibration: boolean;
}

export class HardwareProbeService {
  private cachedGpu: string | null = null;
  private cachedRefreshRate: number = 60;

  constructor() {
    this.detectRefreshRate();
  }

  public getGpuRenderer(): string {
    if (this.cachedGpu) return this.cachedGpu;
    try {
      const canvas = document.createElement('canvas');
      const gl = canvas.getContext('webgl') || canvas.getContext('experimental-webgl');
      if (gl) {
        const debugInfo = (gl as WebGLRenderingContext).getExtension('WEBGL_debug_renderer_info');
        if (debugInfo) {
          const renderer = (gl as WebGLRenderingContext).getParameter(debugInfo.UNMASKED_RENDERER_WEBGL);
          if (renderer) {
            this.cachedGpu = renderer.replace(/ANGLE \(|, Direct3D.*|\)/g, '').trim();
            return this.cachedGpu;
          }
        }
      }
    } catch {
      // Fallback
    }
    this.cachedGpu = 'GPU Acelerada por Hardware (DirectX/Vulkan)';
    return this.cachedGpu;
  }

  private detectRefreshRate() {
    let count = 0;
    const startTime = performance.now();

    const step = (time: number) => {
      count++;
      if (time - startTime < 500) {
        requestAnimationFrame(step);
      } else {
        const fps = Math.round((count * 1000) / (time - startTime));
        if (fps >= 230) this.cachedRefreshRate = 240;
        else if (fps >= 155) this.cachedRefreshRate = 165;
        else if (fps >= 135) this.cachedRefreshRate = 144;
        else if (fps >= 115) this.cachedRefreshRate = 120;
        else if (fps >= 70) this.cachedRefreshRate = 75;
        else this.cachedRefreshRate = 60;
      }
    };
    requestAnimationFrame(step);
  }

  public getRealHardwareInfo(): RealHardwareInfo {
    const width = window.screen?.width || window.innerWidth;
    const height = window.screen?.height || window.innerHeight;
    const dpr = window.devicePixelRatio || 1;
    const physicalWidth = Math.round(width * dpr);
    const physicalHeight = Math.round(height * dpr);

    // Memory info
    let mem = '8 GB DDR5 / VRAM Dinámica';
    if ((performance as any)?.memory?.jsHeapSizeLimit) {
      const limitGb = Math.round((performance as any).memory.jsHeapSizeLimit / (1024 * 1024 * 1024));
      mem = `${Math.max(4, limitGb * 2)} GB RAM Asignada`;
    }

    return {
      gpuRenderer: this.getGpuRenderer(),
      cpuCores: navigator.hardwareConcurrency || 8,
      screenResolution: `${physicalWidth} x ${physicalHeight} (${Math.round(dpr * 100)}% Escala DPI)`,
      screenRefreshRate: this.cachedRefreshRate,
      devicePixelRatio: dpr,
      colorDepth: window.screen?.colorDepth || 24,
      memoryEstimate: mem,
      platformOS: navigator.platform || 'EmuBox Arch Linux x86_64'
    };
  }

  public getConnectedGamepads(): RealGamepadInfo[] {
    const list: RealGamepadInfo[] = [];
    try {
      const pads = navigator.getGamepads ? navigator.getGamepads() : [];
      for (let i = 0; i < pads.length; i++) {
        const pad = pads[i];
        if (pad && pad.connected) {
          list.push({
            index: pad.index,
            id: pad.id || `Mando USB / Bluetooth (Puerto ${pad.index + 1})`,
            connected: true,
            timestamp: pad.timestamp,
            buttonsCount: pad.buttons?.length || 16,
            axesCount: pad.axes?.length || 4,
            hasVibration: !!(pad as any).vibrationActuator || !!(pad as any).hapticActuators?.length
          });
        }
      }
    } catch {
      // ignore
    }
    return list;
  }
}

import assert from 'node:assert/strict';
import { GraphicsDetectorService } from '../solid/src/services/graphics/graphics-detector.service';
import type { HardwareInfo } from '../solid/src/types/system.types';

const hardware: HardwareInfo = { gpuVendor: 'virtual', gpuRenderer: 'SVGA3D; LLVM;',
  cpuModel: 'Fixture CPU', cpuCores: 4, cpuArchitecture: 'x86_64', totalMemoryMb: 8192, freeMemoryMb: 4096 };
const detector = new GraphicsDetectorService();
assert.equal(detector.detectFromHardware({ ...hardware, vulkanSupported: false, openglAccelerated: true }).isGpuAccelerated, true);
assert.equal(detector.getCapabilities().isVirtualMachine, true);
assert.equal(detector.detectFromHardware({ ...hardware, gpuVendor: 'amd', vulkanSupported: true }).pipeline, 'accelerated');
assert.equal(detector.detectFromHardware({ ...hardware, gpuVendor: 'mali', cpuArchitecture: 'aarch64', openglAccelerated: true }).pipeline, 'accelerated');
assert.equal(detector.getCapabilities().selectedBackend, 'opengl');
assert.equal(detector.detectFromHardware({ ...hardware, graphicsAccelerated: false, vulkanSupported: true }).pipeline, 'cpu-compatible');
for (const gpuRenderer of ['llvmpipe', 'softpipe', 'swrast', 'SwiftShader', 'lavapipe']) {
  assert.equal(detector.detectFromHardware({ ...hardware, gpuRenderer, vulkanSupported: true }).isGpuAccelerated, false);
}
for (const cpuArchitecture of ['x86_64', 'aarch64']) {
  for (const gpuRenderer of ['virgl', 'SVGA3D; LLVM;', 'VirtualBox 3D']) {
    const capabilities = detector.detectFromHardware({ ...hardware, cpuArchitecture, gpuRenderer, openglAccelerated: true });
    assert.equal(capabilities.selectedBackend, 'opengl');
    assert.equal(capabilities.gpuKind, 'virtual');
  }
}
assert.equal(detector.detectFromHardware({ ...hardware, vulkanSupported: false, openglAccelerated: false }).pipeline, 'cpu-compatible');

const originalWindow = Object.getOwnPropertyDescriptor(globalThis, 'window');
const originalDocument = Object.getOwnPropertyDescriptor(globalThis, 'document');
try {
  Object.defineProperty(globalThis, 'window', { configurable: true, value: {} });
  Object.defineProperty(globalThis, 'document', { configurable: true, value: {
    documentElement: { setAttribute() {} },
    createElement: () => ({ getContext: () => ({ getExtension: () => null, RENDERER: 1, VENDOR: 2,
      getParameter: (parameter: number) => parameter === 1 ? 'SVGA3D; build RELEASE; LLVM;' : 'VMware, Inc.' }) }),
  } });
  assert.equal(detector.detect().isGpuAccelerated, true);
  assert.equal(detector.getCapabilities().isVirtualMachine, true);
  assert.equal(detector.getCapabilities().selectedBackend, 'webgl');
} finally {
  if (originalWindow) Object.defineProperty(globalThis, 'window', originalWindow);
  else Reflect.deleteProperty(globalThis, 'window');
  if (originalDocument) Object.defineProperty(globalThis, 'document', originalDocument);
  else Reflect.deleteProperty(globalThis, 'document');
}
assert.equal(detector.detect().isGpuAccelerated, false);
console.log('Graphics policy: native GPU and accelerated VM preferred, CPU fallback independent of architecture');
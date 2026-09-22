import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
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
assert.equal(detector.detectFromHardware({ ...hardware, graphicsAccelerated: false, vulkanSupported: true }).pipeline, 'indeterminate');
for (const gpuRenderer of ['llvmpipe', 'softpipe', 'swrast', 'SwiftShader', 'lavapipe']) {
  assert.equal(detector.detectFromHardware({ ...hardware, gpuRenderer, vulkanSupported: true }).isGpuAccelerated, false);
}
for (const cpuArchitecture of ['x86_64', 'aarch64']) {
  for (const gpuRenderer of ['virgl', 'SVGA3D; LLVM;', 'VirtualBox 3D']) {
    const capabilities = detector.detectFromHardware({ ...hardware, cpuArchitecture, gpuRenderer, openglAccelerated: true });
    assert.equal(capabilities.selectedBackend, 'opengl');
    assert.equal(capabilities.gpuKind, 'virtual');
    assert.equal(capabilities.recommendedBlur, false);
  }
}
assert.equal(detector.detectFromHardware({ ...hardware, vulkanSupported: false, openglAccelerated: false }).pipeline, 'indeterminate');

const evidence = { detectionState: 'indeterminate' as const, inventoryComplete: false, inventoryReason: 'inventory_unavailable', devices: [], probes: [],
  selectedDeviceId: null, activeDeviceId: null, backend: 'auto' as const, operationalBackend: 'software' as const, fallbackReason: 'explicit_software_fallback' };
const inconclusive = detector.detectFromHardware({ ...hardware, graphics: evidence });
assert.equal(inconclusive.isGpuAccelerated, null);
assert.equal(inconclusive.pipeline, 'indeterminate');
assert.equal(inconclusive.selectedBackend, 'auto');
assert.equal(inconclusive.operationalBackend, 'software');
assert.equal(inconclusive.evidence?.activeDeviceId, null);
assert.equal(detector.detectFromHardware({ ...hardware, graphics: { ...evidence, detectionState: 'software', backend: 'software' } }).pipeline, 'cpu-compatible');

assert.equal(detector.detect().isGpuAccelerated, null);
const attributes = new Map<string, string>();
const documentTarget = { documentElement: { setAttribute: (name: string, value: string) => attributes.set(name, value) } } as unknown as Document;
detector.detectFromHardware({ ...hardware, openglAccelerated: true }, documentTarget);
assert.equal(attributes.get('data-gpu-kind'), 'virtual');
assert.equal(attributes.get('data-render-pipeline'), 'accelerated');
assert.equal(attributes.get('data-blur-mode'), 'software');
detector.detectFromHardware({ ...hardware, gpuRenderer: 'AMD Radeon', gpuVendor: 'amd', gpuKind: 'physical', isVirtualMachine: false, openglAccelerated: true }, documentTarget);
assert.equal(attributes.get('data-gpu-kind'), 'physical');
assert.equal(attributes.get('data-blur-mode'), 'hardware');
const variablesCss = readFileSync(new URL('../solid/src/styles/variables.css', import.meta.url), 'utf8');
assert.match(variablesCss, /\[data-blur-mode="software"\]\s*\{[^}]*--glass-blur-sm:\s*none;/);
const xmbCss = readFileSync(new URL('../solid/src/styles/xmb.css', import.meta.url), 'utf8');
assert.match(xmbCss, /background:\s*var\(--xmb-background\) var\(--xmb-bg\)/);
assert.match(xmbCss, /html\[data-gpu-kind="virtual"\] \.xmb-wave\s*\{\s*animation: none;/);
assert.ok(!/\.xmb-atmosphere\s*\{[^}]*z-index:\s*-/.test(xmbCss));
const emulatorRuntime = readFileSync(new URL('../src-tauri/src/services/runtime/emulator.rs', import.meta.url), 'utf8');
const emulatorProfiles = ['retroarch', 'pcsx2', 'duckstation', 'dolphin', 'ppsspp'];
assert.match(emulatorRuntime, /renderer_preference\(&hardware\.graphics\.operational_backend\)/);
for (const profile of emulatorProfiles) {
  const source = readFileSync(new URL(`../src-tauri/src/services/emulators/${profile}.rs`, import.meta.url), 'utf8');
  assert.match(source, /RendererPreference/);
  assert.doesNotMatch(source, /vulkan_ok\(/);
  assert.match(source, /RendererPreference::Conservative\s*=>\s*(?:\(\)|return Ok\(\(\)\))/);
}
console.log('Graphics policy: native GPU and accelerated VM preferred, CPU fallback independent of architecture');
import assert from 'node:assert/strict';
import { evaluateAppliance, isCompositorCommand } from '../scripts/check-appliance.mjs';
import { fileURLToPath } from 'node:url';

const adapter = fileURLToPath(new URL('../bin/libemubox-vmwgfx.so', import.meta.url));
const cageCommand = ['/lib64/ld-linux-x86-64.so.2', '--preload', adapter, '/usr/bin/cage', '--', '/opt/emubox/bin/emubox'];
assert.equal(isCompositorCommand(cageCommand), true);
assert.equal(isCompositorCommand(['/usr/bin/cage', '--', 'app']), true);
assert.equal(isCompositorCommand(['/usr/bin/gamescope']), true);
assert.equal(isCompositorCommand([]), false);
assert.equal(isCompositorCommand(['/usr/bin/echo', ...cageCommand]), false);
assert.equal(isCompositorCommand(cageCommand.map(value => value === '/usr/bin/cage' ? '/usr/bin/other' : value)), false);
assert.equal(isCompositorCommand(cageCommand.map(value => value === adapter ? '/tmp/other.so' : value)), false);

const ready = {
  architecture: 'x86_64', distributionSupported: true, systemd: true, uid: 1000, inspectorUid: 1000,
  binaryCompatible: true, librariesAvailable: true,
  directoryAccess: [{ path: '/var/lib/emubox', writable: true }], runtimePersistence: true,
  autologin: true, singleStartup: true, launcherInstalled: true, profileConfigured: true,
  ttyActive: true, drmAccessible: true, inputAccessible: true, waylandSocket: true,
  ttyRecovery: true, bootFilesystem: 'vfat', bootPrivate: true, bootFsckAvailable: true,
  audioRealtimeAvailable: true,
  sessionBus: true, compositorRunning: true, runtimeRunning: true, audioUnitsAvailable: true, audioRunning: true, databaseHeader: true,
};
for (const architecture of ['x86_64', 'aarch64']) {
  const report = evaluateAppliance({ ...ready, architecture, emulators: [] });
  assert.equal(report.status, 'pending-functional-validation');
  assert.equal(report.applianceValidated, false);
  assert.equal(report.distributionValidated, false);
  assert.ok(report.checks.some(check => check.id === 'ui-ipc' && check.status === 'pending'));
}
for (const failure of [
  { architecture: 'unsupported' }, { distributionSupported: false }, { systemd: false },
  { uid: 0 }, { inspectorUid: 0 }, { binaryCompatible: false }, { runtimePersistence: false },
  { autologin: false }, { singleStartup: false }, { launcherInstalled: false },
  { audioUnitsAvailable: false },
  { ttyRecovery: false }, { bootPrivate: false }, { bootFsckAvailable: false },
  { directoryAccess: [{ path: '/var/lib/emubox', writable: false }] },
]) assert.equal(evaluateAppliance({ ...ready, ...failure }).status, 'failed');
const headless = evaluateAppliance({ ...ready, compositorRunning: false, audioRunning: false, runtimeRunning: false });
assert.equal(headless.status, 'pending-functional-validation');
assert.equal(headless.checks.find(check => check.id === 'audio-processes').status, 'pending');
const unknownBoot = evaluateAppliance({ ...ready, bootFilesystem: null });
assert.equal(unknownBoot.status, 'pending-functional-validation');
assert.equal(unknownBoot.checks.find(check => check.id === 'boot-inspection').status, 'pending');
assert.ok(!evaluateAppliance({ ...ready, bootFilesystem: 'ext4', bootFsckAvailable: false })
  .checks.some(check => check.id === 'boot-fsck-tool'));
assert.equal(evaluateAppliance({ ...ready, audioRealtimeAvailable: false })
  .checks.find(check => check.id === 'audio-realtime').status, 'pending');
console.log('Appliance x86_64/aarch64 prerequisites, no-emulator boot and pending acceptance: OK');
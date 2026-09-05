import assert from 'node:assert/strict';
import { MockBackendService } from '../solid/src/services/backend/mock-backend.service';
import { normalizeArchitecture } from '../solid/src/types/architecture.types';
import { emulatorBlockReason, gameBlockReason } from '../solid/src/services/compatibility/launch-capability';
import type { Game } from '../solid/src/types/game.types';

const game: Game = {
  id: 'architecture-fixture', title: 'Fixture', platform: 'ps2', platformName: 'PS2',
  releaseYear: 2000, genre: '', developer: '', publisher: '', rating: 0,
  playTimeMinutes: 0, favorite: false, coverImage: '', description: '', installed: true
};

for (const architecture of ['x86_64', 'aarch64', 'armv7l']) {
  const backend = new MockBackendService([game], architecture);
  const expected = normalizeArchitecture(architecture);
  assert.equal((await backend.getSystemInfo()).architecture, expected);
  assert.equal((await backend.getHardwareInfo()).cpuArchitecture, expected);
  assert.equal((await backend.getDiagnostics()).architecture, expected);
  const emulators = await backend.getEmulators();
  assert.ok(emulators.some(emulator => emulator.id === 'rpcs3'));
  const pcsx2 = emulators.find(emulator => emulator.id === 'pcsx2')!;
  assert.equal(pcsx2.compatibility?.status, architecture === 'x86_64' ? 'supported' : 'unsupported_architecture');
  assert.equal(emulatorBlockReason(pcsx2) === null, architecture === 'x86_64');
  assert.equal(gameBlockReason(game, emulators) === null, architecture === 'x86_64');
  assert.equal((await backend.launchGame(game.id, 'pcsx2')).success, architecture === 'x86_64');
  assert.deepEqual(await backend.getGames(), [game]);
  if (architecture === 'x86_64') {
    await backend.saveEmulator({ ...pcsx2, status: 'inactive', executable: '' });
    const missing = await backend.getEmulator('pcsx2');
    assert.equal(missing?.compatibility?.status, 'not_installed');
    assert.ok(emulatorBlockReason(missing));
    assert.equal((await backend.launchGame(game.id, 'pcsx2')).success, false);
  }
}
assert.equal(normalizeArchitecture('arm64'), 'aarch64');
assert.ok(emulatorBlockReason(undefined));
console.log('Mock architecture, launch capabilities and catalog invariance: OK');
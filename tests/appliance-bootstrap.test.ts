import assert from 'node:assert/strict';
import { createRoot } from 'solid-js';
import { MockBackendService } from '../solid/src/services/backend/mock-backend.service';
import { createSystemStore } from '../solid/src/stores/system.store';
import { createLibraryStore } from '../solid/src/stores/library.store';
import { gameBlockReason } from '../solid/src/services/compatibility/launch-capability';
import type { Game } from '../solid/src/types/game.types';

class EmptyEmulatorBackend extends MockBackendService {
  override async getEmulators() {
    return [];
  }
}

const game: Game = {
  id: 'appliance-catalog-fixture', title: 'Catalog fixture', platform: 'ps2', platformName: 'PS2',
  releaseYear: 2000, genre: '', developer: '', publisher: '', rating: 0,
  playTimeMinutes: 0, favorite: false, coverImage: '', description: '', installed: false,
};

for (const architecture of ['x86_64', 'aarch64']) {
  for (const state of ['empty', 'not_installed']) {
    const backend = state === 'empty'
      ? new EmptyEmulatorBackend([game], architecture)
      : new MockBackendService([game], architecture);
    if (state === 'not_installed') {
      for (const emulator of await backend.getEmulators()) {
        await backend.saveEmulator({ ...emulator, status: 'inactive', executable: '' });
      }
    }
    const stores = createRoot(dispose => ({
      system: createSystemStore(backend), library: createLibraryStore(backend), dispose,
    }));
    try {
      await stores.system.loadSystemData();
      await stores.library.loadGames();
      assert.equal(stores.system.isLoading(), false);
      assert.equal(stores.library.isLoading(), false);
      assert.ok(stores.system.settings());
      assert.ok(stores.system.platforms().length > 0);
      assert.deepEqual(stores.library.games(), [game]);
      assert.ok(gameBlockReason(game, stores.system.emulators()));
      assert.ok(stores.system.emulators().every(emulator => emulator.compatibility?.status !== 'supported'));
      if (state === 'empty') assert.equal(stores.system.emulators().length, 0);
      else assert.ok(stores.system.emulators().length > 0);
      await stores.library.toggleFavorite(game.id);
      assert.equal(stores.library.games()[0].favorite, true);
    } finally {
      stores.dispose();
    }
  }
}
console.log('Appliance bootstrap: settings, platforms and catalog load without installed emulators on both CPUs');
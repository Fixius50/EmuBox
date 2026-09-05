import type { Game, Emulator } from '@contracts/game.types';
import type { IEmuBoxBackend } from '@contracts/backend.types';
import type { ModalStore } from '@stores/modal.store';
import type { SystemStore } from '@stores/system.store';
import type { SoundFxService } from '@services/audio/sound-fx.service';
import { emulatorBlockReason } from '@services/compatibility/launch-capability';

interface UseGameLauncherOptions {
  backend: IEmuBoxBackend;
  systemStore: SystemStore;
  modalStore: ModalStore;
  soundFx: SoundFxService;
}

export function useGameLauncher(options: UseGameLauncherOptions) {
  const { backend, systemStore, modalStore, soundFx } = options;

  const launchWithEmulator = async (game: Game, emulator: Emulator) => {
    if (emulatorBlockReason(emulator)) return;
    soundFx.playSelect();
    try {
      const result = await backend.launchGame(game.id, emulator.id);
      console.log(`[GameLauncher] Juego iniciado: ${result.message} (Motor: ${emulator.name}, PID: ${result.pid})`);
    } catch (err: any) {
      console.error('[GameLauncher] Error al iniciar sesión:', err);
    } finally {
      modalStore.closeEmulatorSelector();
      modalStore.closeConfirmLaunch();
      modalStore.closeGameDetails();
    }
  };

  const launchGameDirect = async (game: Game) => {
    soundFx.playSelect();
    try {
      const availableEmulators = systemStore.emulators();
      const targetEmulator = availableEmulators.find(
        (e) => !emulatorBlockReason(e) && e.supportedPlatforms.includes(game.platform)
      );
      if (!targetEmulator) {
        return;
      }

      const emulatorId = targetEmulator ? targetEmulator.id : '';
      const result = await backend.launchGame(game.id, emulatorId);
      console.log(`[GameLauncher] Juego iniciado: ${result.message} (Motor: ${targetEmulator ? targetEmulator.name : 'Auto'}, PID: ${result.pid})`);
    } catch (err: any) {
      console.error('[GameLauncher] Error al lanzar juego:', err);
    } finally {
      modalStore.closeEmulatorSelector();
      modalStore.closeConfirmLaunch();
      modalStore.closeGameDetails();
    }
  };

  return {
    launchWithEmulator,
    launchGameDirect
  };
}

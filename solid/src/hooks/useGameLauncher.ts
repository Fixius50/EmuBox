import type { Game, Emulator } from '@contracts/game.types';
import type { IEmuBoxBackend } from '@contracts/backend.types';
import type { ModalStore } from '@stores/modal.store';
import type { SystemStore } from '@stores/system.store';
import type { SoundFxService } from '@services/audio/sound-fx.service';

interface UseGameLauncherOptions {
  backend: IEmuBoxBackend;
  systemStore: SystemStore;
  modalStore: ModalStore;
  soundFx: SoundFxService;
}

export function useGameLauncher(options: UseGameLauncherOptions) {
  const { backend, systemStore, modalStore, soundFx } = options;

  const launchWithEmulator = async (game: Game, emulator: Emulator) => {
    soundFx.playSelect();
    try {
      const result = await backend.launchGame(game.id, emulator.id);
      alert(`[EMUBOX LAUNCHER]\n\n${result.message}\nMotor: ${emulator.name}\nPID: ${result.pid}`);
    } catch (err: any) {
      console.error('[GameLauncher] Error al iniciar sesión:', err);
      alert(`[EMUBOX LAUNCHER - ERROR]\n\nNo se pudo lanzar el juego: ${err?.message || err}`);
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
      let targetEmulator = availableEmulators.find(
        (e) => e.status === 'active' && e.supportedPlatforms.includes(game.platform)
      );
      if (!targetEmulator) {
        targetEmulator = availableEmulators.find((e) => e.supportedPlatforms.includes(game.platform))
          || availableEmulators[0];
      }

      const emulatorId = targetEmulator ? targetEmulator.id : '';
      const result = await backend.launchGame(game.id, emulatorId);
      alert(`[EMUBOX LAUNCHER]\n\n${result.message}\nMotor: ${targetEmulator ? targetEmulator.name : 'Auto'}\nPID: ${result.pid}`);
    } catch (err: any) {
      console.error('[GameLauncher] Error al lanzar juego:', err);
      alert(`[EMUBOX LAUNCHER - ERROR]\n\nNo se pudo lanzar el juego: ${err?.message || err}`);
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

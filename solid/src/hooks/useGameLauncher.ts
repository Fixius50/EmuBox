import type { Game, Emulator } from '@contracts/game.types';
import type { IEmuBoxBackend } from '@contracts/backend.types';
import type { ModalStore } from '@stores/modal.store';
import type { SoundFxService } from '@services/audio/sound-fx.service';

interface UseGameLauncherOptions {
  backend: IEmuBoxBackend;
  modalStore: ModalStore;
  soundFx: SoundFxService;
}

export function useGameLauncher(options: UseGameLauncherOptions) {
  const { backend, modalStore, soundFx } = options;

  const launchWithEmulator = async (game: Game, emulator: Emulator) => {
    soundFx.playSelect();
    try {
      const result = await backend.launchGame(game.id, emulator.id);
      alert(`[EMUBOX LAUNCHER]\n\n${result.message}\nMotor: ${emulator.name}\nPID: ${result.pid}`);
    } catch (err) {
      console.error('[GameLauncher] Error al iniciar sesión:', err);
    } finally {
      modalStore.closeEmulatorSelector();
      modalStore.closeConfirmLaunch();
      modalStore.closeGameDetails();
    }
  };

  return {
    launchWithEmulator
  };
}

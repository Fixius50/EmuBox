import { onMount, onCleanup, createSignal } from 'solid-js';
import type { InputAction, InputDeviceStatus } from '@contracts/input.types';
import { InputManager } from '@services/input/input.manager';
import { KeyboardProvider } from '@services/input/keyboard.provider';
import { GamepadProvider } from '@services/input/gamepad.provider';
import { TauriIpcProvider } from '@services/input/tauri-ipc.provider';

interface UseConsoleInputOptions {
  onAction: (action: InputAction) => void;
}

const keyboardStatus: InputDeviceStatus = {
  isConnected: true,
  deviceName: 'Teclado USB Detectado',
  source: 'keyboard',
};

export function useConsoleInput(options: UseConsoleInputOptions) {
  const inputManager = new InputManager();
  const [inputStatus, setInputStatus] = createSignal<InputDeviceStatus>(keyboardStatus);

  onMount(() => {
    const keyboard = new KeyboardProvider();
    const gamepad = new GamepadProvider();
    const tauriInput = new TauriIpcProvider();

    inputManager.registerProvider(keyboard);
    inputManager.registerProvider(gamepad);
    inputManager.registerProvider(tauriInput);

    inputManager.onStatusChange((status) => {
      if (!status.isConnected && inputStatus().source === status.source)
        setInputStatus(keyboardStatus);
    });

    inputManager.onAction((action: InputAction, status?: InputDeviceStatus) => {
      if (status?.isConnected) setInputStatus(status);
      options.onAction(action);
    });

    onCleanup(() => {
      inputManager.destroy();
    });
  });

  return {
    inputStatus,
    inputManager
  };
}

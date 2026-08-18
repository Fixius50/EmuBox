import { onMount, onCleanup, createSignal } from 'solid-js';
import type { InputAction, InputDeviceStatus } from '@contracts/input.types';
import { InputManager } from '@services/input/input.manager';
import { KeyboardProvider } from '@services/input/keyboard.provider';
import { GamepadProvider } from '@services/input/gamepad.provider';
import { TauriIpcProvider } from '@services/input/tauri-ipc.provider';

interface UseConsoleInputOptions {
  onAction: (action: InputAction) => void;
}

export function useConsoleInput(options: UseConsoleInputOptions) {
  const inputManager = new InputManager();
  const [inputStatus, setInputStatus] = createSignal<InputDeviceStatus>({
    isConnected: true,
    deviceName: 'Teclado USB Detectado',
    source: 'keyboard'
  });

  onMount(() => {
    const keyboard = new KeyboardProvider();
    const gamepad = new GamepadProvider();
    const tauriInput = new TauriIpcProvider();

    inputManager.registerProvider(keyboard);
    inputManager.registerProvider(gamepad);
    inputManager.registerProvider(tauriInput);

    inputManager.onStatusChange((status) => {
      setInputStatus(status);
    });

    inputManager.onAction((action: InputAction) => {
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

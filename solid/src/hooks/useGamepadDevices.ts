import { createSignal, onMount, onCleanup, Accessor } from 'solid-js';
import type { GamepadDeviceInfo } from '@contracts/settings.types';

export interface UseGamepadDevicesReturn {
  gamepadsList: Accessor<GamepadDeviceInfo[]>;
  refreshGamepads: () => void;
}

export function useGamepadDevices(): UseGamepadDevicesReturn {
  const [gamepadsList, setGamepadsList] = createSignal<GamepadDeviceInfo[]>([]);
  let pollInterval: number | undefined;

  const refreshGamepads = () => {
    if (typeof navigator !== 'undefined' && navigator.getGamepads) {
      const raw = navigator.getGamepads();
      const list: GamepadDeviceInfo[] = [];
      for (let i = 0; i < raw.length; i++) {
        const pad = raw[i];
        if (pad) {
          list.push({
            index: pad.index,
            id: pad.id || `Mando #${pad.index + 1}`,
            buttonsCount: pad.buttons.length,
            axesCount: pad.axes.length,
            hasVibration: !!(pad as any).vibrationActuator
          });
        }
      }
      setGamepadsList(list);
    }
  };

  onMount(() => {
    refreshGamepads();
    if (typeof window !== 'undefined') {
      window.addEventListener('gamepadconnected', refreshGamepads);
      window.addEventListener('gamepaddisconnected', refreshGamepads);
      pollInterval = window.setInterval(refreshGamepads, 2000);
    }
  });

  onCleanup(() => {
    if (typeof window !== 'undefined') {
      window.removeEventListener('gamepadconnected', refreshGamepads);
      window.removeEventListener('gamepaddisconnected', refreshGamepads);
      if (pollInterval) clearInterval(pollInterval);
    }
  });

  return {
    gamepadsList,
    refreshGamepads
  };
}

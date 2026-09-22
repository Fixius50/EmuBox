import { createSignal, onCleanup, onMount } from 'solid-js';
import type { DisplayInfo } from '@contracts/system.types';
import type { IEmuBoxBackend } from '@contracts/backend.types';

export function useDisplayInfo(backend?: IEmuBoxBackend) {
  const [info, setInfo] = createSignal<DisplayInfo | null>(null);
  const [error, setError] = createSignal('');
  let disposed = false;

  const refresh = async () => {
    if (!backend) return;
    try {
      const display = await backend.getDisplayInfo();
      if (!disposed) {
        setInfo(display);
        setError('');
      }
    } catch (cause) {
      if (!disposed) {
        setError(cause instanceof Error ? cause.message : 'Salida actual no disponible');
      }
    }
  };

  onMount(() => { void refresh(); });
  onCleanup(() => { disposed = true; });

  return { info, error, refresh };
}
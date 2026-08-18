import type {
  InputAction,
  InputDeviceStatus,
  IInputProvider,
  InputActionListener,
  InputStatusListener
} from '@contracts/input.types';

interface TauriWindow {
  __TAURI__?: {
    event?: {
      listen: <T>(eventName: string, handler: (event: { payload: T }) => void) => Promise<() => void>;
    };
  };
}

export class TauriIpcProvider implements IInputProvider {
  public readonly id = 'tauri-ipc-provider';
  public readonly name = 'Tauri Native Gamepad Service (gilrs)';

  private actionListeners: Set<InputActionListener> = new Set();
  private statusListeners: Set<InputStatusListener> = new Set();
  private unlisten: (() => void) | null = null;
  private isConnected: boolean = false;

  public async init(): Promise<void> {
    if (typeof window !== 'undefined' && (window as unknown as TauriWindow).__TAURI__?.event) {
      try {
        const tauri = (window as unknown as TauriWindow).__TAURI__!.event!;
        this.unlisten = await tauri.listen<string>('gamepad-event', (event) => {
          const action = event.payload as InputAction;
          this.emitAction(action);
        });
        this.isConnected = true;
        this.broadcastStatus();
      } catch (err) {
        console.warn('[TauriIpcProvider] gilrs no disponible:', err);
      }
    }
  }

  public destroy(): void {
    if (this.unlisten) {
      this.unlisten();
      this.unlisten = null;
    }
    this.isConnected = false;
    this.actionListeners.clear();
    this.statusListeners.clear();
  }

  public onAction(listener: InputActionListener): () => void {
    this.actionListeners.add(listener);
    return () => this.actionListeners.delete(listener);
  }

  public onStatusChange(listener: InputStatusListener): () => void {
    this.statusListeners.add(listener);
    return () => this.statusListeners.delete(listener);
  }

  public getStatus(): InputDeviceStatus {
    return {
      isConnected: this.isConnected,
      deviceName: this.isConnected ? 'Mando Nativo Rust (gilrs)' : 'Tauri IPC Inactivo',
      source: 'tauri_gilrs'
    };
  }

  public simulateAction(action: InputAction): void {
    this.emitAction(action);
  }

  public injectTauriEvent(action: InputAction): void {
    this.emitAction(action);
  }

  private emitAction(action: InputAction): void {
    for (const listener of this.actionListeners) {
      listener(action);
    }
  }

  private broadcastStatus(): void {
    const status = this.getStatus();
    for (const listener of this.statusListeners) {
      listener(status);
    }
  }
}

export { TauriIpcProvider as MockTauriInputProvider };

import type {
  InputAction,
  InputDeviceStatus,
  IInputProvider,
  InputActionListener,
  InputStatusListener
} from '@contracts/input.types';

export class InputManager {
  private providers: Map<string, IInputProvider> = new Map();
  private actionListeners: Set<InputActionListener> = new Set();
  private statusListeners: Set<InputStatusListener> = new Set();
  private cleanups: Map<string, () => void> = new Map();

  public registerProvider(provider: IInputProvider): void {
    if (this.providers.has(provider.id)) return;

    this.providers.set(provider.id, provider);
    provider.init();

    const unhookAction = provider.onAction((action: InputAction) => {
      this.broadcastAction(action);
    });

    const unhookStatus = provider.onStatusChange((status: InputDeviceStatus) => {
      this.broadcastStatus(status);
    });

    this.cleanups.set(provider.id, () => {
      unhookAction();
      unhookStatus();
      provider.destroy();
    });
  }

  public unregisterProvider(providerId: string): void {
    const cleanup = this.cleanups.get(providerId);
    if (cleanup) {
      cleanup();
      this.cleanups.delete(providerId);
    }
    this.providers.delete(providerId);
  }

  public onAction(listener: InputActionListener): () => void {
    this.actionListeners.add(listener);
    return () => this.actionListeners.delete(listener);
  }

  public onStatusChange(listener: InputStatusListener): () => void {
    this.statusListeners.add(listener);
    return () => this.statusListeners.delete(listener);
  }

  private broadcastAction(action: InputAction): void {
    for (const listener of this.actionListeners) {
      listener(action);
    }
  }

  private broadcastStatus(status: InputDeviceStatus): void {
    for (const listener of this.statusListeners) {
      listener(status);
    }
  }

  public getActiveStatus(): InputDeviceStatus {
    for (const provider of this.providers.values()) {
      const status = provider.getStatus();
      if (status.isConnected && status.source !== 'keyboard') {
        return status;
      }
    }
    return {
      isConnected: true,
      deviceName: 'Teclado USB Estándar',
      source: 'keyboard'
    };
  }

  public destroy(): void {
    for (const cleanup of this.cleanups.values()) {
      cleanup();
    }
    this.cleanups.clear();
    this.providers.clear();
    this.actionListeners.clear();
    this.statusListeners.clear();
  }
}

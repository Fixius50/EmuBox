import type {
  InputAction,
  InputDeviceStatus,
  IInputProvider,
  InputActionListener,
  InputStatusListener
} from '@contracts/input.types';

export class KeyboardProvider implements IInputProvider {
  public readonly id = 'keyboard-provider';
  public readonly name = 'Teclado Estándar';

  private actionListeners: Set<InputActionListener> = new Set();
  private statusListeners: Set<InputStatusListener> = new Set();
  private boundKeyDown: (e: KeyboardEvent) => void;

  private keyMap: Record<string, InputAction> = {
    'ArrowUp': 'NAV_UP',
    'KeyW': 'NAV_UP',
    'w': 'NAV_UP',
    'W': 'NAV_UP',

    'ArrowDown': 'NAV_DOWN',
    'KeyS': 'NAV_DOWN',
    's': 'NAV_DOWN',
    'S': 'NAV_DOWN',

    'ArrowLeft': 'NAV_LEFT',
    'KeyA': 'NAV_LEFT',
    'a': 'NAV_LEFT',
    'A': 'NAV_LEFT',

    'ArrowRight': 'NAV_RIGHT',
    'KeyD': 'NAV_RIGHT',
    'd': 'NAV_RIGHT',
    'D': 'NAV_RIGHT',

    'Enter': 'BUTTON_A',
    'Space': 'BUTTON_A',
    ' ': 'BUTTON_A',

    'Escape': 'BUTTON_B',
    'Backspace': 'BUTTON_B',

    'KeyX': 'BUTTON_X',
    'x': 'BUTTON_X',
    'X': 'BUTTON_X',
    'KeyF': 'BUTTON_X',
    'f': 'BUTTON_X',
    'F': 'BUTTON_X',

    'KeyY': 'BUTTON_Y',
    'y': 'BUTTON_Y',
    'Y': 'BUTTON_Y',

    'KeyQ': 'BUTTON_LB',
    'q': 'BUTTON_LB',
    'Q': 'BUTTON_LB',
    'PageUp': 'BUTTON_LB',

    'KeyE': 'BUTTON_RB',
    'e': 'BUTTON_RB',
    'E': 'BUTTON_RB',
    'PageDown': 'BUTTON_RB',

    'Tab': 'BUTTON_START',
    'F1': 'BUTTON_START'
  };

  constructor() {
    this.boundKeyDown = this.handleKeyDown.bind(this);
  }

  public init(): void {
    if (typeof window !== 'undefined') {
      window.addEventListener('keydown', this.boundKeyDown);
    }
  }

  public destroy(): void {
    if (typeof window !== 'undefined') {
      window.removeEventListener('keydown', this.boundKeyDown);
    }
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
      isConnected: true,
      deviceName: 'Teclado USB Detectado',
      source: 'keyboard'
    };
  }

  private handleKeyDown(e: KeyboardEvent): void {
    const target = e.target as HTMLElement;
    if (target && (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') && e.key !== 'Escape') {
      return;
    }

    const action = this.keyMap[e.code] || this.keyMap[e.key];
    if (action) {
      e.preventDefault();
      for (const listener of this.actionListeners) {
        listener(action);
      }
    }
  }
}

export { KeyboardProvider as KeyboardInputProvider };

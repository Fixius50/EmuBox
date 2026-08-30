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
    // Navigation (Arrows & WASD)
    'ArrowUp': 'NAV_UP',
    'KeyW': 'NAV_UP',
    'w': 'NAV_UP',
    'W': 'NAV_UP',

    'ArrowDown': 'NAV_DOWN',
    'KeyS': 'NAV_DOWN',
    's': 'NAV_DOWN',
    'S': 'NAV_DOWN',

    'ArrowLeft': 'NAV_LEFT',

    'ArrowRight': 'NAV_RIGHT',
    'KeyD': 'NAV_RIGHT',
    'd': 'NAV_RIGHT',
    'D': 'NAV_RIGHT',

    // Primary Action [A] (Enter, Space, A, Z, J)
    'Enter': 'BUTTON_A',
    'Space': 'BUTTON_A',
    ' ': 'BUTTON_A',
    'KeyA': 'BUTTON_A',
    'a': 'BUTTON_A',
    'A': 'BUTTON_A',
    'KeyZ': 'BUTTON_A',
    'z': 'BUTTON_A',
    'Z': 'BUTTON_A',
    'KeyJ': 'BUTTON_A',
    'j': 'BUTTON_A',
    'J': 'BUTTON_A',

    // Secondary / Back [B] (Escape, Backspace, B, K)
    'Escape': 'BUTTON_B',
    'Backspace': 'BUTTON_B',
    'KeyB': 'BUTTON_B',
    'b': 'BUTTON_B',
    'B': 'BUTTON_B',
    'KeyK': 'BUTTON_B',
    'k': 'BUTTON_B',
    'K': 'BUTTON_B',

    // Action [X] (X, F, U)
    'KeyX': 'BUTTON_X',
    'x': 'BUTTON_X',
    'X': 'BUTTON_X',
    'KeyF': 'BUTTON_X',
    'f': 'BUTTON_X',
    'F': 'BUTTON_X',
    'KeyU': 'BUTTON_X',
    'u': 'BUTTON_X',
    'U': 'BUTTON_X',

    // Action [Y] (Y, C, I)
    'KeyY': 'BUTTON_Y',
    'y': 'BUTTON_Y',
    'Y': 'BUTTON_Y',
    'KeyC': 'BUTTON_Y',
    'c': 'BUTTON_Y',
    'C': 'BUTTON_Y',
    'KeyI': 'BUTTON_Y',
    'i': 'BUTTON_Y',
    'I': 'BUTTON_Y',

    // Shoulder Triggers (LB/RB)
    'KeyQ': 'BUTTON_LB',
    'q': 'BUTTON_LB',
    'Q': 'BUTTON_LB',
    'PageUp': 'BUTTON_LB',

    'KeyE': 'BUTTON_RB',
    'e': 'BUTTON_RB',
    'E': 'BUTTON_RB',
    'PageDown': 'BUTTON_RB',

    // Menu / Start / Select
    'Tab': 'BUTTON_START',
    'F1': 'BUTTON_START',
    'Control': 'BUTTON_SELECT',
    'Shift': 'BUTTON_SELECT'
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
      source: 'keyboard',
      batteryLevel: 100
    };
  }

  private handleKeyDown(e: KeyboardEvent): void {
    const action = this.keyMap[e.code] || this.keyMap[e.key];
    if (action) {
      // Prevent default browser scrolling on arrow keys and space
      if (['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'Space', ' '].includes(e.key)) {
        e.preventDefault();
      }
      this.emitAction(action);
    }
  }

  private emitAction(action: InputAction): void {
    for (const listener of this.actionListeners) {
      try {
        listener(action);
      } catch (err) {
        console.error('[KeyboardProvider] Error in action listener:', err);
      }
    }
  }
}

import type {
  InputAction,
  InputDeviceStatus,
  IInputProvider,
  InputActionListener,
  InputStatusListener
} from '@contracts/input.types';

export class GamepadProvider implements IInputProvider {
  public readonly id = 'gamepad-provider';
  public readonly name = 'W3C Gamepad API';

  private actionListeners: Set<InputActionListener> = new Set();
  private statusListeners: Set<InputStatusListener> = new Set();

  private isRunning: boolean = false;
  private animFrameId: number | null = null;

  private deadzone: number = 0.35;
  private buttonStates: boolean[] = [];
  private axisStates: { x: number; y: number } = { x: 0, y: 0 };
  private repeatTimer: number = 0;
  private connectedGamepadIndex: number | null = null;
  private connectedGamepadName: string = 'Mando Desconectado';

  private boundConnected: (e: GamepadEvent) => void;
  private boundDisconnected: (e: GamepadEvent) => void;

  constructor(deadzone: number = 0.35) {
    this.deadzone = deadzone;
    this.boundConnected = this.handleGamepadConnected.bind(this);
    this.boundDisconnected = this.handleGamepadDisconnected.bind(this);
  }

  public init(): void {
    if (typeof window !== 'undefined') {
      window.addEventListener('gamepadconnected', this.boundConnected);
      window.addEventListener('gamepaddisconnected', this.boundDisconnected);

      this.isRunning = true;
      this.pollLoop();
    }
  }

  public destroy(): void {
    this.isRunning = false;
    if (this.animFrameId !== null && typeof window !== 'undefined') {
      cancelAnimationFrame(this.animFrameId);
      this.animFrameId = null;
    }
    if (typeof window !== 'undefined') {
      window.removeEventListener('gamepadconnected', this.boundConnected);
      window.removeEventListener('gamepaddisconnected', this.boundDisconnected);
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
      isConnected: this.connectedGamepadIndex !== null,
      deviceName: this.connectedGamepadName,
      source: 'gamepad'
    };
  }

  private handleGamepadConnected(e: GamepadEvent): void {
    this.connectedGamepadIndex = e.gamepad.index;
    this.connectedGamepadName = e.gamepad.id || 'Mando Estándar Conectado';
    this.broadcastStatus();
  }

  private handleGamepadDisconnected(e: GamepadEvent): void {
    if (this.connectedGamepadIndex === e.gamepad.index) {
      this.connectedGamepadIndex = null;
      this.connectedGamepadName = 'Mando Desconectado';
      this.broadcastStatus();
    }
  }

  private broadcastStatus(): void {
    const status = this.getStatus();
    for (const listener of this.statusListeners) {
      listener(status);
    }
  }

  private emitAction(action: InputAction): void {
    for (const listener of this.actionListeners) {
      listener(action);
    }
  }

  private pollLoop(): void {
    if (!this.isRunning) return;

    if (typeof navigator !== 'undefined' && navigator.getGamepads) {
      const gamepads = navigator.getGamepads();
      let activePad: Gamepad | null = null;

      if (this.connectedGamepadIndex !== null && gamepads[this.connectedGamepadIndex]) {
        activePad = gamepads[this.connectedGamepadIndex];
      } else {
        for (let i = 0; i < gamepads.length; i++) {
          if (gamepads[i]) {
            activePad = gamepads[i];
            if (this.connectedGamepadIndex === null) {
              this.connectedGamepadIndex = i;
              this.connectedGamepadName = activePad!.id;
              this.broadcastStatus();
            }
            break;
          }
        }
      }

      if (activePad) {
        this.processGamepadInput(activePad);
      }
    }

    this.animFrameId = requestAnimationFrame(() => this.pollLoop());
  }

  private processGamepadInput(pad: Gamepad): void {
    const checkButton = (index: number, action: InputAction) => {
      const isPressed = pad.buttons[index]?.pressed || (pad.buttons[index]?.value || 0) > 0.5;
      const wasPressed = this.buttonStates[index] || false;

      if (isPressed && !wasPressed) {
        this.emitAction(action);
      }
      this.buttonStates[index] = isPressed;
    };

    checkButton(0, 'BUTTON_A');
    checkButton(1, 'BUTTON_B');
    checkButton(2, 'BUTTON_X');
    checkButton(3, 'BUTTON_Y');
    checkButton(4, 'BUTTON_LB');
    checkButton(5, 'BUTTON_RB');
    checkButton(6, 'BUTTON_LT');
    checkButton(7, 'BUTTON_RT');
    checkButton(8, 'BUTTON_SELECT');
    checkButton(9, 'BUTTON_START');
    checkButton(12, 'NAV_UP');
    checkButton(13, 'NAV_DOWN');
    checkButton(14, 'NAV_LEFT');
    checkButton(15, 'NAV_RIGHT');
    checkButton(16, 'HOME');

    const rawX = pad.axes[0] || 0;
    const rawY = pad.axes[1] || 0;

    const normX = Math.abs(rawX) > this.deadzone ? rawX : 0;
    const normY = Math.abs(rawY) > this.deadzone ? rawY : 0;

    const now = performance.now();

    if (normX !== 0 || normY !== 0) {
      if (this.axisStates.x === 0 && this.axisStates.y === 0) {
        if (normX > 0.5) this.emitAction('NAV_RIGHT');
        else if (normX < -0.5) this.emitAction('NAV_LEFT');
        else if (normY > 0.5) this.emitAction('NAV_DOWN');
        else if (normY < -0.5) this.emitAction('NAV_UP');
        this.repeatTimer = now + 350;
      } else if (now > this.repeatTimer) {
        if (normX > 0.5) this.emitAction('NAV_RIGHT');
        else if (normX < -0.5) this.emitAction('NAV_LEFT');
        else if (normY > 0.5) this.emitAction('NAV_DOWN');
        else if (normY < -0.5) this.emitAction('NAV_UP');
        this.repeatTimer = now + 120;
      }
    }

    this.axisStates = { x: normX, y: normY };
  }
}

export { GamepadProvider as GamepadInputProvider };

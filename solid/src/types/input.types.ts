export type InputAction =
  | 'NAV_UP'
  | 'NAV_DOWN'
  | 'NAV_LEFT'
  | 'NAV_RIGHT'
  | 'BUTTON_A'
  | 'BUTTON_B'
  | 'BUTTON_X'
  | 'BUTTON_Y'
  | 'BUTTON_LB'
  | 'BUTTON_RB'
  | 'BUTTON_LT'
  | 'BUTTON_RT'
  | 'BUTTON_START'
  | 'BUTTON_SELECT'
  | 'HOME'
  | 'MAINTENANCE_MENU';

export interface InputDeviceStatus {
  isConnected: boolean;
  deviceName: string;
  source: 'gamepad' | 'keyboard' | 'tauri_gilrs';
  batteryLevel?: number;
}

export interface GamepadDevice {
  index: number;
  id: string;
  name: string;
  connected: boolean;
  vendorId?: string;
  productId?: string;
  buttonsCount: number | null;
  axesCount: number | null;
  hasVibration: boolean | null;
  batteryPercent?: number;
  isPrimary: boolean;
}

export interface GamepadStatus {
  connectedCount: number;
  primaryDeviceIndex: number | null;
  devices: GamepadDevice[];
}

export type InputActionListener = (
  action: InputAction,
  status?: InputDeviceStatus,
) => void;
export type InputStatusListener = (status: InputDeviceStatus) => void;

export interface IInputProvider {
  readonly id: string;
  readonly name: string;
  init(): Promise<void> | void;
  destroy(): void;
  onAction(listener: InputActionListener): () => void;
  onStatusChange(listener: InputStatusListener): () => void;
  getStatus(): InputDeviceStatus;
}

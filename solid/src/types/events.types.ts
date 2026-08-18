import type { RunningGameInfo } from './process.types';
import type { GamepadDevice } from './input.types';
import type { ScanGamesResult } from './backend.types';

export interface EmuBoxEventsMap {
  'game-launched': RunningGameInfo;
  'game-exited': { pid: number; exitCode: number; durationSeconds: number };
  'game-crashed': { pid: number; errorMessage: string; logPath?: string };
  'emulator-status-changed': { emulatorId: string; newStatus: 'active' | 'inactive' | 'missing_bios' };
  'gamepad-connected': GamepadDevice;
  'gamepad-disconnected': { index: number; id: string };
  'scan-started': { targetDirectory: string };
  'scan-progress': { scanned: number; currentItem: string };
  'scan-completed': ScanGamesResult;
  'system-power-state-changed': { action: 'shutdown' | 'restart' | 'sleep' | 'logout' };
}

export type EmuBoxEventName = keyof EmuBoxEventsMap;
export type EmuBoxEventListener<K extends EmuBoxEventName> = (payload: EmuBoxEventsMap[K]) => void;

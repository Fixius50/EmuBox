export type ProcessExecutionStatus = 'starting' | 'running' | 'paused' | 'exited' | 'crashed';

export interface RunningGameInfo {
  pid: number;
  gameId: string;
  gameTitle: string;
  platformId: string;
  emulatorId: string;
  emulatorName: string;
  executable: string;
  arguments: string[];
  startTime: number;
  cpuPercent: number;
  memoryMb: number;
  status: ProcessExecutionStatus;
}

export interface ProcessStatus {
  hasActiveGame: boolean;
  runningGame: RunningGameInfo | null;
  activeChildPids: number[];
}

export interface ProcessLaunchConfig {
  gameId: string;
  emulatorId: string;
  romPath: string;
  saveSlot?: number;
  fullscreen?: boolean;
  useGamescope?: boolean;
  gamescopeArgs?: string[];
  extraArgs?: string[];
  environmentVars?: Record<string, string>;
}

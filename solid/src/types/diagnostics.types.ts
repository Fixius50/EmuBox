export type LogLevel = 'debug' | 'info' | 'warn' | 'error';

export interface LogEntry {
  timestamp: number;
  level: LogLevel;
  source: 'frontend' | 'tauri' | 'emulator' | 'system';
  category: string;
  message: string;
  data?: Record<string, unknown>;
}

export interface DiagnosticReport {
  generatedAt: number;
  osInfo: string;
  kernelVersion: string;
  architecture: string;
  gpuAdapter: string;
  vulkanReady: boolean;
  gamescopeReady: boolean;
  pipewireReady: boolean;
  storageMounted: boolean;
  emulatorsInstalledCount: number;
  emulatorsMissingCount: number;
  connectedGamepadsCount: number;
  recentErrors: LogEntry[];
  rawSummaryText: string;
}

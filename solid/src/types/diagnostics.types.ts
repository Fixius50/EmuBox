export type LogLevel = 'debug' | 'info' | 'warn' | 'error' | 'unknown';

export interface LogEntry {
  timestamp: number | null;
  level: LogLevel;
  source: string;
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
  recentErrors: LogEntry[] | null;
  rawSummaryText: string;
}

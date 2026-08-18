import type { DiagnosticReport, LogEntry, LogLevel } from '@contracts/diagnostics.types';
import type { IEmuBoxBackend } from '@contracts/backend.types';

export class DiagnosticsService {
  private logs: LogEntry[] = [];

  constructor(private backend?: IEmuBoxBackend) {}

  public log(level: LogLevel, category: string, message: string, data?: Record<string, unknown>): void {
    const entry: LogEntry = {
      timestamp: Date.now(),
      level,
      source: 'frontend',
      category,
      message,
      data
    };
    this.logs.push(entry);
    if (this.logs.length > 500) {
      this.logs.shift();
    }
  }

  public info(category: string, message: string, data?: Record<string, unknown>): void {
    this.log('info', category, message, data);
  }

  public warn(category: string, message: string, data?: Record<string, unknown>): void {
    this.log('warn', category, message, data);
  }

  public error(category: string, message: string, data?: Record<string, unknown>): void {
    this.log('error', category, message, data);
  }

  public async getDiagnostics(): Promise<DiagnosticReport> {
    if (this.backend) {
      return this.backend.getDiagnostics();
    }
    return {
      generatedAt: Date.now(),
      osInfo: 'EmuBox Dev Environment',
      kernelVersion: 'Development Kernel',
      architecture: 'x86_64',
      gpuAdapter: 'WebGL Accelerated Canvas',
      vulkanReady: true,
      gamescopeReady: true,
      pipewireReady: true,
      storageMounted: true,
      emulatorsInstalledCount: 8,
      emulatorsMissingCount: 0,
      connectedGamepadsCount: 1,
      recentErrors: this.logs.filter(l => l.level === 'error'),
      rawSummaryText: 'EmuBox Diagnostics Report: All systems nominal.'
    };
  }
}

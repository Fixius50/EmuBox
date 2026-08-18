import type { ProcessStatus, RunningGameInfo, LaunchGameRequest, LaunchResult } from '@contracts/backend.types';
import type { IEmuBoxBackend } from '@contracts/backend.types';

export class ProcessService {
  constructor(private backend: IEmuBoxBackend) {}

  public async launchGame(request: LaunchGameRequest): Promise<LaunchResult> {
    return this.backend.launchGame(request);
  }

  public async stopGame(): Promise<void> {
    return this.backend.stopGame();
  }

  public async isGameRunning(): Promise<boolean> {
    return this.backend.isGameRunning();
  }

  public async getRunningGame(): Promise<RunningGameInfo | null> {
    return this.backend.getRunningGame();
  }

  public async getProcessStatus(): Promise<ProcessStatus> {
    return this.backend.getProcessStatus();
  }

  public async killProcess(pid: number): Promise<boolean> {
    return this.backend.killProcess(pid);
  }
}

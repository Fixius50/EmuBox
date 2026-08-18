import type { UpdateInfo, UpdateCheckResult, UpdateProgress, RollbackResult, UpdateChannel } from '@contracts/update.types';
import type { IEmuBoxBackend } from '@contracts/backend.types';

export class UpdateService {
  constructor(private backend: IEmuBoxBackend) {}

  public async getUpdateInfo(): Promise<UpdateInfo> {
    return this.backend.getUpdateInfo();
  }

  public async checkForUpdates(channel?: UpdateChannel): Promise<UpdateCheckResult> {
    return this.backend.checkForUpdates(channel);
  }

  public async applyUpdate(targetVersion?: string): Promise<UpdateProgress> {
    return this.backend.applyUpdate(targetVersion);
  }

  public async rollbackToVersion(version: string): Promise<RollbackResult> {
    return this.backend.rollbackToVersion(version);
  }

  public async restartAppSession(): Promise<void> {
    return this.backend.restartAppSession();
  }
}

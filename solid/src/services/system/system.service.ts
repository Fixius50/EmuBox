import type { SystemInfo, HardwareInfo, DisplayInfo, AudioInfo, PowerAction } from '@contracts/system.types';
import type { IEmuBoxBackend } from '@contracts/backend.types';

export class SystemService {
  constructor(private backend: IEmuBoxBackend) {}

  public async getSystemInfo(): Promise<SystemInfo> {
    return this.backend.getSystemInfo();
  }

  public async getHardwareInfo(): Promise<HardwareInfo> {
    return this.backend.getHardwareInfo();
  }

  public async getDisplayInfo(): Promise<DisplayInfo> {
    return this.backend.getDisplayInfo();
  }

  public async getAudioInfo(): Promise<AudioInfo> {
    return this.backend.getAudioInfo();
  }

  public async performPowerAction(action: PowerAction): Promise<void> {
    switch (action) {
      case 'shutdown':
        return this.backend.shutdown();
      case 'restart':
        return this.backend.restart();
      case 'sleep':
        return this.backend.sleep();
      case 'logout':
        return this.backend.logout();
    }
  }
}

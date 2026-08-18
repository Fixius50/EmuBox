import type { Emulator, PlatformId } from '@contracts/game.types';
import type { IEmuBoxBackend } from '@contracts/backend.types';

export class EmulatorService {
  constructor(private backend: IEmuBoxBackend) {}

  public async getEmulators(): Promise<Emulator[]> {
    return this.backend.getEmulators();
  }

  public async getEmulatorById(id: string): Promise<Emulator | null> {
    return this.backend.getEmulator(id);
  }

  public async getEmulatorsForPlatform(platformId: PlatformId): Promise<Emulator[]> {
    const all = await this.backend.getEmulators();
    return all.filter(e => e.supportedPlatforms.includes(platformId));
  }

  public async saveEmulator(emulator: Emulator): Promise<void> {
    return this.backend.saveEmulator(emulator);
  }

  public async deleteEmulator(id: string): Promise<void> {
    return this.backend.deleteEmulator(id);
  }
}

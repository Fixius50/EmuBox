import type { StorageInfo, StorageLocation } from '@contracts/storage.types';
import type { IEmuBoxBackend } from '@contracts/backend.types';

export class StorageService {
  constructor(private backend: IEmuBoxBackend) {}

  public async getStorageInfo(): Promise<StorageInfo> {
    return this.backend.getStorageInfo();
  }

  public async getLocations(): Promise<Record<string, StorageLocation>> {
    return this.backend.getStorageLocations();
  }
}

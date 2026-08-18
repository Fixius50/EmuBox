export interface StorageDrive {
  id: string;
  name: string;
  mountPoint: string;
  filesystem: string;
  totalBytes: number;
  availableBytes: number;
  usedBytes: number;
  isRemovable: boolean;
  isSystemDrive: boolean;
}

export interface StorageLocation {
  id: 'roms' | 'saves' | 'states' | 'screenshots' | 'covers' | 'bios' | 'logs' | 'cache';
  label: string;
  path: string;
  totalFiles: number;
  totalBytes: number;
  accessible: boolean;
  isWritable: boolean;
}

export interface StorageInfo {
  drives: StorageDrive[];
  locations: Record<string, StorageLocation>;
  totalGamesStorageBytes: number;
  totalSavesStorageBytes: number;
}

export type UpdateChannel = 'stable' | 'beta' | 'nightly' | 'git-dev';

export type UpdateStage =
  | 'idle'
  | 'checking'
  | 'available'
  | 'up_to_date'
  | 'downloading'
  | 'verifying'
  | 'extracting'
  | 'symlinking'
  | 'ready_to_restart'
  | 'error';

export interface InstalledReleaseInfo {
  version: string;
  releaseDate: string;
  installedAt: number;
  commitHash?: string;
  isCurrent: boolean;
  installPath: string;
}

export interface UpdateInfo {
  currentVersion: string;
  channel: UpdateChannel;
  lastChecked: number;
  hasUpdate: boolean;
  latestVersion?: string;
  releaseNotes?: string[];
  releaseDate?: string;
  downloadSizeBytes?: number;
  installedReleases: InstalledReleaseInfo[];
}

export interface UpdateCheckResult {
  updateAvailable: boolean;
  currentVersion: string;
  targetVersion: string;
  releaseNotes: string[];
  releaseDate: string;
  downloadUrl: string;
  checksumSha256: string;
  downloadSizeBytes: number;
}

export interface UpdateProgress {
  stage: UpdateStage;
  percent: number;
  bytesDownloaded: number;
  totalBytes: number;
  message: string;
}

export interface RollbackResult {
  success: boolean;
  restoredVersion: string;
  message: string;
}

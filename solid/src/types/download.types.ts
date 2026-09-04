export type DownloadSourceType = 'http' | 'torrent' | 'magnet';

export type DownloadStatus =
  | 'queued'
  | 'downloading'
  | 'paused'
  | 'completed'
  | 'failed'
  | 'cancelled';

export interface DownloadSource {
  id: string;
  gameId: string;
  name: string;
  sourceType: DownloadSourceType;
  uri: string;
  sizeBytes?: number;
  checksum?: string;
  available: boolean;
}

export interface DownloadJob {
  id: string;
  gameId: string;
  sourceId: string;
  platform: string;
  destinationPath: string;
  status: DownloadStatus;
  progress: number;
  downloadedBytes: number;
  totalBytes?: number;
  speedBytesPerSecond: number;
  error?: string;
}

export interface CreateDownloadRequest {
  gameId: string;
  platform: string;
  source: DownloadSource;
}

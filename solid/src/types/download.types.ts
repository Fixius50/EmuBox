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

export interface DownloadSourceOption extends DownloadSource {
  access: 'http' | 'host_page' | 'unverified_http' | 'magnet' | 'torrent' | 'unsupported';
  downloadable: boolean;
  reason?: string | null;
}

export interface CreateDownloadRequest {
  gameId: string;
  platform: string;
  source: DownloadSource;
}

export interface HydraDownloadItem {
  title: string;
  uris: string[];
  fileSize?: string | number;
  uploadDate?: string;
  platform?: string;
  gameId?: string;
  sourceId?: string;
  checksum?: string;
  genre?: string;
  developer?: string;
  publisher?: string;
  rating?: number;
  coverImage?: string;
  backdropImage?: string;
  description?: string;
}

export interface HydraDownloadManifest {
  name?: string;
  downloads: HydraDownloadItem[];
  platform?: string;
}

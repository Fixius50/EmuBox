import type { CreateDownloadRequest, DownloadJob, DownloadSource } from '@contracts/download.types';
import type { IEmuBoxBackend } from '@contracts/backend.types';

export class DownloadService {
  constructor(private readonly backend: IEmuBoxBackend) {}

  public createSource(source: DownloadSource): Promise<DownloadSource> {
    return this.backend.createDownloadSource(source);
  }

  public createJob(request: CreateDownloadRequest): Promise<DownloadJob> {
    return this.backend.createDownloadJob(request);
  }

  public listJobs(): Promise<DownloadJob[]> {
    return this.backend.getDownloadJobs();
  }

  public start(jobId: string): Promise<DownloadJob> {
    return this.backend.startDownload(jobId);
  }

  public pause(jobId: string): Promise<DownloadJob> {
    return this.backend.pauseDownload(jobId);
  }

  public resume(jobId: string): Promise<DownloadJob> {
    return this.backend.resumeDownload(jobId);
  }

  public cancel(jobId: string): Promise<DownloadJob> {
    return this.backend.cancelDownload(jobId);
  }

  public downloadGame(gameId: string): Promise<DownloadJob> {
    return this.backend.downloadGame(gameId);
  }

  public importFromJson(jsonContent: string | object): Promise<DownloadJob[]> {
    const content = typeof jsonContent === 'string' ? jsonContent : JSON.stringify(jsonContent);
    return this.backend.importDownloadsFromJson(content);
  }

  public importFromUrl(url: string): Promise<DownloadJob[]> {
    return this.backend.importDownloadsFromUrl(url);
  }

  public importDownloadLinks(): Promise<DownloadJob[]> {
    return this.backend.importDownloadLinks();
  }
}

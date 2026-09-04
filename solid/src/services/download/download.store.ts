import { createSignal, onCleanup } from 'solid-js';
import type { DownloadJob } from '@contracts/download.types';
import type { DownloadService } from './download.service';

export function createDownloadStore(service: DownloadService) {
  const [jobs, setJobs] = createSignal<DownloadJob[]>([]);
  let refreshTimer: ReturnType<typeof setInterval> | undefined;

  const refresh = async () => {
    setJobs(await service.listJobs());
  };

  const startPolling = (intervalMs = 1000) => {
    void refresh();
    refreshTimer = setInterval(() => void refresh(), intervalMs);
  };

  const stopPolling = () => {
    if (refreshTimer) clearInterval(refreshTimer);
    refreshTimer = undefined;
  };

  onCleanup(stopPolling);

  return { jobs, refresh, startPolling, stopPolling };
}

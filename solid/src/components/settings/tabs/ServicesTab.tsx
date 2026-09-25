import { createResource, createSignal, For, onCleanup, onMount, Show } from 'solid-js';
import type { Component } from 'solid-js';
import { ExternalLink, Pause, Play, RefreshCw, RotateCw, Square, Trash2 } from 'lucide-solid';
import type { DownloadJob } from '@contracts/download.types';
import type { ServicesTabProps } from '@contracts/settings.types';
import { SettingSwitch } from '../SettingSwitch';

const torrentStatus: Record<DownloadJob['status'], string> = {
  queued: 'En cola', downloading: 'Descargando', paused: 'Pausado',
  completed: 'Completado', downloaded: 'Descargado', failed: 'Error', cancelled: 'Cancelado',
};

export const ServicesTab: Component<ServicesTabProps> = (props) => {
  const [jackettBusy, setJackettBusy] = createSignal(false);
  const [jackettError, setJackettError] = createSignal('');
  const [jobs, { refetch }] = createResource(async () => props.backend?.getDownloadJobs() ?? []);
  const [jobBusy, setJobBusy] = createSignal<string | null>(null);
  const [jobError, setJobError] = createSignal('');
  const torrents = () => (jobs() ?? []).filter(job => job.provider === 'bittorrent');
  onMount(() => {
    const timer = setInterval(() => { if (props.backend) void refetch(); }, 3000);
    onCleanup(() => clearInterval(timer));
  });
  const controlJob = async (job: DownloadJob, action: 'pause' | 'resume' | 'cancel') => {
    if (!props.backend || jobBusy()) return;
    setJobBusy(job.id);
    setJobError('');
    try {
      if (action === 'pause') await props.backend.pauseDownload(job.id);
      else if (action === 'resume' && props.libraryStore) {
        await props.libraryStore.controlDownload(job, 'resume');
        const error = props.libraryStore.downloadError();
        if (error?.gameId === job.gameId) throw new Error(error.message);
      }
      else if (action === 'resume') await props.backend.resumeDownload(job.id);
      else await props.backend.cancelDownload(job.id);
      await refetch();
      await props.libraryStore?.refreshJobs();
    } catch (error) {
      setJobError(typeof error === 'string' ? error : error instanceof Error ? error.message : 'No se pudo modificar el torrent.');
    } finally {
      setJobBusy(null);
    }
  };
  const cancelledJob = async (job: DownloadJob, action: 'retry' | 'delete') => {
    if (!props.libraryStore || jobBusy() || job.status !== 'cancelled') return;
    if (action === 'delete' && !window.confirm(`¿Borrar el trabajo cancelado de ${job.gameId} y sus archivos temporales?`)) return;
    setJobBusy(job.id);
    setJobError('');
    try {
      if (action === 'retry') {
        await props.libraryStore.retryCancelled(job);
        const error = props.libraryStore.downloadError();
        if (error?.gameId === job.gameId) throw new Error(error.message);
      } else await props.libraryStore.deleteCancelled(job);
      await refetch();
    } catch (error) {
      setJobError(typeof error === 'string' ? error : error instanceof Error ? error.message : 'No se pudo modificar el trabajo.');
    } finally {
      setJobBusy(null);
    }
  };
  const openJackett = async () => {
    if (!props.backend || jackettBusy()) return;
    setJackettError('');
    setJackettBusy(true);
    try {
      await props.backend.openJackett();
    } catch (error) {
      setJackettError(typeof error === 'string' ? error : error instanceof Error ? error.message : 'No se pudo abrir Jackett local.');
    } finally {
      setJackettBusy(false);
    }
  };
  return (
    <div class="settings-tab-panel">
      <div class="panel-header-block">
        <div class="panel-header-titles">
          <h3 class="panel-section-title">Servicios</h3>
        </div>
      </div>
      <div class="settings-form-stack">
        <SettingSwitch
          title="Compartir torrents completados"
          description="Continua subiendo hasta alcanzar ratio 1:1, con limite de 64 KiB/s. Se aplica a descargas nuevas."
          checked={props.settings?.system?.seedCompletedTorrents ?? false}
          isFocused={props.isRowFocused(0)}
          onChange={(val) => {
            props.onSelectContentArea?.();
            props.onUpdateSettings((settings) => {
              settings.system = { ...settings.system, seedCompletedTorrents: val };
            });
          }}
        />
        <SettingSwitch
          title="Instalar descargas automaticamente"
          description="Selecciona el archivo de lanzamiento solo cuando el paquete tiene un candidato inequívoco."
          checked={props.settings?.system?.autoInstallDownloads ?? true}
          isFocused={props.isRowFocused(1)}
          onChange={(value) => {
            props.onSelectContentArea?.();
            props.onUpdateSettings((settings) => {
              settings.system = { ...settings.system, autoInstallDownloads: value };
            });
          }}
        />
      </div>
      <section class="settings-information" aria-label="Servicios de descarga locales">
        <h4>Servicios de descarga</h4>
        <button type="button" class="settings-local-web-button" disabled={!props.backend || jackettBusy()} onClick={() => void openJackett()}>
          <ExternalLink size={18} aria-hidden="true" /> Abrir Jackett
        </button>
        <Show when={jackettError()}><p role="alert">{jackettError()}</p></Show>
        <div class="settings-torrent-heading">
          <h4>Torrents</h4>
          <button type="button" class="settings-local-web-button" title="Actualizar torrents" aria-label="Actualizar torrents" onClick={() => void refetch()}>
            <RefreshCw size={18} aria-hidden="true" />
          </button>
        </div>
        <Show when={jobs.error}><p role="alert">No se pudo consultar el estado de los torrents.</p></Show>
        <Show when={jobError()}><p role="alert">{jobError()}</p></Show>
        <Show when={jobs.loading}><p role="status">Cargando torrents...</p></Show>
        <Show when={!jobs.loading && !jobs.error && !torrents().length}><p>No hay torrents registrados.</p></Show>
        <div class="settings-torrent-list">
          <For each={torrents()}>{job => (
            <div class="settings-torrent-row">
              <div><strong>{job.gameId}</strong><span>{torrentStatus[job.status]} · {Math.round(Math.max(0, Math.min(1, job.progress)) * 100)}%</span></div>
              <div class="settings-torrent-actions">
                <Show when={job.status === 'downloading' || job.status === 'queued'}>
                  <button title="Pausar torrent" aria-label={`Pausar ${job.gameId}`} disabled={Boolean(jobBusy())} onClick={() => void controlJob(job, 'pause')}><Pause size={18} /></button>
                </Show>
                <Show when={job.status === 'paused' || job.status === 'failed'}>
                  <button title="Reanudar torrent" aria-label={`Reanudar ${job.gameId}`} disabled={Boolean(jobBusy())} onClick={() => void controlJob(job, 'resume')}><Play size={18} /></button>
                </Show>
                <Show when={!['completed', 'downloaded', 'cancelled'].includes(job.status)}>
                  <button title="Cancelar torrent" aria-label={`Cancelar ${job.gameId}`} disabled={Boolean(jobBusy())} onClick={() => void controlJob(job, 'cancel')}><Square size={18} /></button>
                </Show>
                <Show when={job.status === 'cancelled' && props.libraryStore}>
                  <button title="Reintentar descarga" aria-label={`Reintentar ${job.gameId}`} disabled={Boolean(jobBusy())} onClick={() => void cancelledJob(job, 'retry')}><RotateCw size={18} /></button>
                  <button title="Borrar trabajo cancelado" aria-label={`Borrar ${job.gameId}`} disabled={Boolean(jobBusy())} onClick={() => void cancelledJob(job, 'delete')}><Trash2 size={18} /></button>
                </Show>
              </div>
            </div>
          )}</For>
        </div>
      </section>
    </div>
  );
};
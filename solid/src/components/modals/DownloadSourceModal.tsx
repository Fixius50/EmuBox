import { For, Show, createEffect } from 'solid-js';
import { Dialog } from '@kobalte/core/dialog';
import type { LibraryStore } from '@stores/library.store';

const labels = {
  http: 'HTTP candidato a archivo', unverified_http: 'HTTP sin verificar', host_page: 'Pagina de alojamiento',
  magnet: 'Magnet', torrent: 'BitTorrent', unsupported: 'No compatible',
};

export function DownloadSourceModal(props: { store: LibraryStore; onConfirm: () => void }) {
  const selected = () => props.store.sourceOptions()[props.store.sourceIndex()];
  let list: HTMLDivElement | undefined;
  createEffect(() => {
    const index = props.store.sourceIndex();
    list?.querySelector(`[data-source-index="${index}"]`)?.scrollIntoView({ block: 'nearest' });
  });
  const move = (step: number) => props.store.setSourceIndex(Math.max(0,
    Math.min(props.store.sourceOptions().length - 1, props.store.sourceIndex() + step)));
  return (
    <Dialog open={Boolean(props.store.sourceGame())} onOpenChange={open => { if (!open) props.store.closeSources(); }}>
      <Dialog.Portal>
        <Dialog.Overlay class="console-modal-backdrop" />
        <div class="console-modal-center-container">
          <Dialog.Content class="download-source-dialog" onKeyDown={event => {
            event.stopPropagation();
            if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
              event.preventDefault();
              move(event.key === 'ArrowDown' ? 1 : -1);
            }
          }}>
            <Dialog.Title>Fuentes de descarga</Dialog.Title>
            <Dialog.Description>{props.store.sourceGame()?.title}</Dialog.Description>
            <Show when={props.store.sourcesLoading()}><p role="status">Consultando fuentes...</p></Show>
            <Show when={props.store.sourcesError()}><p role="alert">{props.store.sourcesError()}</p></Show>
            <Show when={!props.store.sourcesLoading() && !props.store.sourcesError() && props.store.sourceOptions().length === 0}>
              <p role="status">No hay fuentes registradas.</p>
            </Show>
            <div class="download-source-list" role="radiogroup" aria-label="Fuentes disponibles" ref={list}>
              <For each={props.store.sourceOptions()}>{(source, index) => (
                <label class={`download-source-option ${props.store.sourceIndex() === index() ? 'selected' : ''}`} data-source-index={index()}>
                  <input type="radio" name="download-source" checked={props.store.sourceIndex() === index()}
                    onChange={() => props.store.setSourceIndex(index())} />
                  <span>
                    <strong>{labels[source.access]}</strong>
                    <span class="download-source-uri">{source.uri}</span>
                    <Show when={source.sizeBytes}><span>{((source.sizeBytes ?? 0) / 1024 / 1024).toFixed(1)} MiB</span></Show>
                    <Show when={source.reason}><span class="download-source-reason">{source.reason}</span></Show>
                  </span>
                </label>
              )}</For>
            </div>
            <div class="download-source-actions">
              <button class="console-btn primary-glow-btn" disabled={!selected()?.downloadable || props.store.sourcesLoading()}
                onClick={props.onConfirm}>Descargar seleccionada</button>
              <button class="console-btn ghost-btn" onClick={props.store.closeSources}>Volver</button>
            </div>
          </Dialog.Content>
        </div>
      </Dialog.Portal>
    </Dialog>
  );
}
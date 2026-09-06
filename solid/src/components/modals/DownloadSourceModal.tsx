import { For, Show, createEffect, createMemo, createSignal, on, onMount, onCleanup } from 'solid-js';
import { Dialog } from '@kobalte/core/dialog';
import { Download, Heart, Play, X } from 'lucide-solid';
import type { LibraryStore } from '@stores/library.store';
import type { Game } from '@contracts/game.types';
import type { InputAction } from '@contracts/input.types';
import { ConsoleHardwareVisual } from '@components/common/ConsoleHardwareVisual';

const labels = {
  http: 'HTTP candidato a archivo', unverified_http: 'HTTP sin verificar', host_page: 'Pagina de alojamiento',
  magnet: 'Magnet', torrent: 'BitTorrent', unsupported: 'No compatible',
};

interface DownloadSourceModalProps {
  store: LibraryStore;
  onConfirm: () => void;
  onPlay?: (game: Game) => void;
  playBlockReason?: string | null;
  onControllerReady?: (handler: ((action: InputAction) => void) | null) => void;
}

export function DownloadSourceModal(props: DownloadSourceModalProps) {
  const selected = () => props.store.sourceOptions()[props.store.sourceIndex()];
  const game = () => props.store.sourceGame();
  const variant = createMemo(() => props.store.games().find(entry => entry.id === selected()?.gameId));
  const detailsGame = () => variant() || game();
  const [panel, setPanel] = createSignal<'sources' | 'details'>('sources');
  const [coverFailed, setCoverFailed] = createSignal(false);
  const [favoriteError, setFavoriteError] = createSignal('');
  const favorite = () => props.store.catalogGames().find(entry => entry.variants.some(item => item.id === game()?.id))?.favorite ?? game()?.favorite;
  let list: HTMLDivElement | undefined;
  let details: HTMLDivElement | undefined;
  let returnGameId: string | undefined;
  const busy = () => Boolean(game() && props.store.catalogDownloadingIds().has(game()!.id));
  const toggleFavorite = async () => {
    const current = game();
    if (!current) return;
    setFavoriteError('');
    try { await props.store.toggleFavorite(current.id); }
    catch { setFavoriteError('No se pudo actualizar el favorito.'); }
  };
  const host = (uri: string) => {
    try { const url = new URL(uri); return url.hostname.replace(/^www\./, '') || 'Red BitTorrent'; }
    catch { return 'Fuente sin identificar'; }
  };
  createEffect(on(() => game()?.id, () => {
    if (game()) returnGameId = game()!.id;
    setPanel('sources');
    setFavoriteError('');
    setCoverFailed(false);
    if (details) details.scrollTop = 0;
  }));
  createEffect(() => {
    const index = props.store.sourceIndex();
    if (panel() === 'sources') list?.querySelector(`[data-source-index="${index}"]`)?.scrollIntoView({ block: 'nearest' });
  });
  const move = (step: number) => {
    if (panel() === 'details') { details?.scrollBy({ top: step * 160, behavior: 'smooth' }); return; }
    props.store.setSourceIndex(Math.max(0, Math.min(props.store.sourceOptions().length - 1, props.store.sourceIndex() + step)));
    list?.querySelector<HTMLInputElement>(`[data-source-index="${props.store.sourceIndex()}"] input`)?.focus({ preventScroll: true });
  };
  const focusPanel = (target: 'sources' | 'details') => {
    setPanel(target);
    if (target === 'details') {
      details?.focus({ preventScroll: true });
      details?.scrollIntoView({ block: 'nearest' });
    }
    else list?.querySelector<HTMLInputElement>(`[data-source-index="${props.store.sourceIndex()}"] input`)?.focus({ preventScroll: true });
  };
  const controller = (action: InputAction) => {
    if (!game()) return;
    if (action === 'BUTTON_B') props.store.closeSources();
    else if (action === 'NAV_RIGHT') focusPanel('details');
    else if (action === 'NAV_LEFT') focusPanel('sources');
    else if (action === 'NAV_UP' || action === 'NAV_DOWN') move(action === 'NAV_DOWN' ? 1 : -1);
    else if (action === 'BUTTON_X') void toggleFavorite();
    else if (action === 'BUTTON_A' && panel() === 'sources' && selected()?.downloadable && !busy()) props.onConfirm();
  };
  onMount(() => props.onControllerReady?.(controller));
  onCleanup(() => props.onControllerReady?.(null));
  return (
    <Dialog open={Boolean(props.store.sourceGame())} onOpenChange={open => { if (!open) props.store.closeSources(); }}>
      <Dialog.Portal>
        <Dialog.Overlay class="console-modal-backdrop game-case-overlay" />
        <div class="console-modal-center-container">
          <Dialog.Content class="download-source-dialog game-case" onCloseAutoFocus={event => {
            event.preventDefault();
            const id = returnGameId;
            requestAnimationFrame(() => {
              if (id && !document.querySelector('[role="dialog"]')) document.getElementById(`shelf-card-${id}`)?.focus({ preventScroll: true });
            });
          }} onKeyDown={event => {
            event.stopPropagation();
            if (event.key === 'Escape') { event.preventDefault(); props.store.closeSources(); }
            else if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
              event.preventDefault();
              move(event.key === 'ArrowDown' ? 1 : -1);
            } else if (event.key === 'ArrowRight' || event.key === 'ArrowLeft') {
              event.preventDefault();
              focusPanel(event.key === 'ArrowRight' ? 'details' : 'sources');
            } else if (event.key === 'Enter' && event.target instanceof HTMLInputElement) {
              event.preventDefault();
              controller('BUTTON_A');
            }
          }}>
            <Dialog.CloseButton class="game-case-close" aria-label="Cerrar ficha" title="Cerrar ficha"><X size={20} /></Dialog.CloseButton>
            <div class="game-case-left" onFocusIn={() => setPanel('sources')}>
              <header class="game-case-summary">
                <div class="game-case-cover">
                  <Show when={game()?.coverImage && !coverFailed()} fallback={<ConsoleHardwareVisual platformId={game()?.platform || 'all'} size="lg" />}>
                    <img src={game()?.coverImage} alt={game()?.title} onError={() => setCoverFailed(true)} />
                  </Show>
                </div>
                <div class="game-case-heading">
                  <span class="game-case-platform">{game()?.platformName}</span>
                  <Dialog.Title>{game()?.title}</Dialog.Title>
                  <Dialog.Description>{game()?.installed ? 'Instalado' : 'No instalado'}</Dialog.Description>
                  <button class="game-case-favorite" aria-pressed={Boolean(favorite())} aria-label="Alternar favorito" title="Alternar favorito" onClick={() => { void toggleFavorite(); }}>
                    <Heart size={18} fill={favorite() ? 'currentColor' : 'none'} />
                  </button>
                </div>
              </header>
              <div class="game-case-section-label"><h3>Distribuidores y paquetes</h3><span>{props.store.sourceOptions().length} fuentes</span></div>
              <Show when={props.store.sourcesLoading()}><p class="game-case-state" role="status">Consultando distribuidores...</p></Show>
              <Show when={props.store.sourcesError()}><p class="game-case-state" role="alert">{props.store.sourcesError()}</p></Show>
              <Show when={!props.store.sourcesLoading() && !props.store.sourcesError() && props.store.sourceOptions().length === 0}>
                <p class="game-case-state" role="status">No hay fuentes registradas.</p>
              </Show>
              <div class="download-source-list" role="radiogroup" aria-label="Distribuidores y paquetes" ref={list}>
                <For each={props.store.sourceOptions()}>{(source, index) => (
                  <label class={`download-source-option ${props.store.sourceIndex() === index() ? 'selected' : ''}`} data-source-index={index()}>
                    <input type="radio" name="download-source" checked={props.store.sourceIndex() === index()}
                      onChange={() => props.store.setSourceIndex(index())} />
                    <span>
                      <strong>{source.name}</strong>
                      <span class="game-case-source-meta">{host(source.uri)} · {labels[source.access]}</span>
                      <Show when={source.sizeBytes}><span class="game-case-source-meta">{((source.sizeBytes ?? 0) / 1024 / 1024).toFixed(1)} MiB</span></Show>
                      <Show when={source.reason}><span class="download-source-reason">{source.reason}</span></Show>
                    </span>
                  </label>
                )}</For>
              </div>
              <Show when={favoriteError()}><p class="game-case-state" role="alert">{favoriteError()}</p></Show>
              <footer class="download-source-actions">
                <button class="game-case-download" disabled={!selected()?.downloadable || props.store.sourcesLoading() || busy()}
                  onClick={props.onConfirm}><Download size={18} />{busy() ? 'Descargando...' : 'Descargar seleccionada'}</button>
                <Show when={game()?.installed && props.onPlay}>
                  <button class="game-case-play" disabled={Boolean(props.playBlockReason)} title={props.playBlockReason || 'Jugar'}
                    onClick={() => { const current = game(); if (current && !props.playBlockReason) props.onPlay?.(current); }}><Play size={18} />Jugar</button>
                </Show>
              </footer>
              <Show when={game()?.installed && props.playBlockReason}><p class="game-case-state" role="status">{props.playBlockReason}</p></Show>
            </div>
            <div class="game-case-right" data-active={panel() === 'details'} tabIndex={0} role="region" aria-label="Descripcion y datos del juego" ref={details} onFocusIn={() => setPanel('details')}>
              <Show when={game()?.backdropImage}><img class="game-case-backdrop" src={game()?.backdropImage} alt="" onError={event => { event.currentTarget.hidden = true; }} /></Show>
              <div class="game-case-details">
                <span class="game-case-platform">Ficha del juego</span>
                <h3>{game()?.title}</h3>
                <p class="game-case-description">{detailsGame()?.description || game()?.description || 'Descripcion no disponible.'}</p>
                <dl class="game-case-facts">
                  <div><dt>Plataforma</dt><dd>{game()?.platformName}</dd></div>
                  <div><dt>Lanzamiento</dt><dd>{detailsGame()?.releaseYear || game()?.releaseYear || 'No disponible'}</dd></div>
                  <div><dt>Desarrollador</dt><dd>{detailsGame()?.developer || game()?.developer || 'No disponible'}</dd></div>
                  <div><dt>Editor</dt><dd>{detailsGame()?.publisher || game()?.publisher || 'No disponible'}</dd></div>
                  <div><dt>Genero</dt><dd>{detailsGame()?.genre || game()?.genre || 'No disponible'}</dd></div>
                  <div><dt>Valoracion</dt><dd>{detailsGame()?.rating ? `${detailsGame()!.rating.toFixed(1)} / 5` : 'No disponible'}</dd></div>
                  <div><dt>Tiempo jugado</dt><dd>{game()?.playTimeMinutes || 0} min</dd></div>
                </dl>
                <Show when={selected()}>{source => (
                  <section class="game-case-package">
                    <h4>Paquete seleccionado</h4>
                    <p>{source().name}</p>
                    <p>{labels[source().access]}</p>
                    <span class="download-source-uri">{source().uri}</span>
                    <Show when={source().reason}><p class="download-source-reason">{source().reason}</p></Show>
                  </section>
                )}</Show>
              </div>
            </div>
          </Dialog.Content>
        </div>
      </Dialog.Portal>
    </Dialog>
  );
}
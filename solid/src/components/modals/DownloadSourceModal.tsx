import {
  For,
  Show,
  createEffect,
  createMemo,
  createSignal,
  on,
  onMount,
  onCleanup,
} from "solid-js";
import { Dialog } from "@kobalte/core/dialog";
import { Download, Heart, Play, Pause, X, Square, RotateCw, Check } from "lucide-solid";
import type { LibraryStore } from "@stores/library.store";
import type { Game } from "@contracts/game.types";
import type { InputAction } from "@contracts/input.types";
import { ConsoleHardwareVisual } from "@components/common/ConsoleHardwareVisual";

const labels = {
  http: "HTTP candidato a archivo",
  unverified_http: "HTTP sin verificar",
  host_page: "Pagina de alojamiento",
  magnet: "Magnet",
  torrent: "BitTorrent",
  unsupported: "No compatible",
};

const phases: Record<string, string> = {
  queued: 'En cola', transferring: 'Descargando', verifying: 'Verificando', preparing: 'Preparando',
  ready: 'Preparado', preparation_required: 'Requiere preparacion', paused: 'Pausado', failed: 'Error', cancelled: 'Cancelado',
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
  const selectedRelease = () => props.store.releaseOptions()[props.store.releaseIndex()];
  const game = () => props.store.sourceGame();
  const currentJob = () => props.store.downloadJobs().find(job => job.sourceId === selected()?.id);
  const variant = createMemo(() =>
    props.store.games().find((entry) => entry.id === (selected()?.gameId || selectedRelease()?.catalogGameId)),
  );
  const detailsGame = () => variant() || game();
  const [panel, setPanel] = createSignal<"sources" | "details" | "files">("sources");
  const [localFiles, setLocalFiles] = createSignal<string[]>([]);
  const [localIndex, setLocalIndex] = createSignal(0);
  const [localLoading, setLocalLoading] = createSignal(false);
  const [localSaving, setLocalSaving] = createSignal(false);
  const [localError, setLocalError] = createSignal("");
  let localSelect: HTMLSelectElement | undefined;
  let localRequest = 0;
  createEffect(on(() => `${game()?.id || ''}:${currentJob()?.id || ''}:${currentJob()?.status || ''}`, () => {
    const request = ++localRequest;
    const job = currentJob();
    setLocalFiles([]);
    setLocalIndex(0);
    setLocalError("");
    setLocalLoading(false);
    if (!game() || !job || !['downloaded', 'completed'].includes(job.status)) return;
    setLocalLoading(true);
    void props.store.getDownloadCandidates(job.id).then(files => {
      if (request === localRequest) setLocalFiles(files);
    }).catch(() => {
      if (request === localRequest) setLocalError("No se pudieron consultar los archivos locales.");
    }).finally(() => { if (request === localRequest) setLocalLoading(false); });
  }));
  const chooseLocal = async () => {
    const job = currentJob();
    const path = localFiles()[localIndex()];
    if (!job || !path || localSaving() || localLoading()) return;
    const request = localRequest;
    setLocalSaving(true);
    setLocalError("");
    try {
      await props.store.selectDownloadCandidate(job.id, path);
    } catch {
      if (request === localRequest) setLocalError("No se pudo seleccionar el archivo local. Comprueba que el paquete siga disponible.");
    } finally { setLocalSaving(false); }
  };
  const [coverFailed, setCoverFailed] = createSignal(false);
  const [favoriteError, setFavoriteError] = createSignal("");
  const favorite = () =>
    props.store
      .catalogGames()
      .find((entry) => entry.variants.some((item) => item.id === game()?.id))
      ?.favorite ?? game()?.favorite;
  let list: HTMLDivElement | undefined;
  let details: HTMLDivElement | undefined;
  let dialogContent: HTMLDivElement | undefined;
  let closeButton: HTMLButtonElement | undefined;
  let returnGameId: string | undefined;
  const busy = () =>
    Boolean(game() && props.store.catalogDownloadingIds().has(game()!.id)) || currentJob()?.status === 'downloading' || currentJob()?.status === 'queued';
  const toggleFavorite = async () => {
    const current = game();
    if (!current) return;
    setFavoriteError("");
    try {
      await props.store.toggleFavorite(current.id);
    } catch {
      setFavoriteError("No se pudo actualizar el favorito.");
    }
  };
  const host = (uri: string) => {
    try {
      const url = new URL(uri);
      return url.hostname.replace(/^www\./, "") || "Red BitTorrent";
    } catch {
      return "Fuente sin identificar";
    }
  };
  createEffect(
    on(
      () => game()?.id,
      () => {
        if (game()) returnGameId = game()!.id;
        setPanel("sources");
        setFavoriteError("");
        setCoverFailed(false);
        if (details) details.scrollTop = 0;
      },
    ),
  );
  createEffect(() => {
    const index = props.store.sourceIndex();
    if (panel() === "sources")
      list
        ?.querySelector(`[data-source-index="${index}"]`)
        ?.scrollIntoView({ block: "nearest" });
  });
  const move = (step: number) => {
    if (panel() === "files") {
      setLocalIndex(index => Math.max(0, Math.min(localFiles().length - 1, index + step)));
      return;
    }
    if (panel() === "details") {
      details?.scrollBy({ top: step * 160, behavior: "smooth" });
      return;
    }
    props.store.setSourceIndex(
      Math.max(
        0,
        Math.min(
          props.store.sourceOptions().length - 1,
          props.store.sourceIndex() + step,
        ),
      ),
    );
    list
      ?.querySelector<HTMLInputElement>(
        `[data-source-index="${props.store.sourceIndex()}"] input`,
      )
      ?.focus({ preventScroll: true });
  };
  const moveRelease = (step: number) => {
    props.store.selectRelease(props.store.releaseIndex() + step);
  };
  const focusPanel = (target: "sources" | "details" | "files") => {
    setPanel(target);
    if (target === "files") {
      localSelect?.focus();
      localSelect?.scrollIntoView({ block: "nearest" });
    } else if (target === "details") {
      details?.focus({ preventScroll: true });
      details?.scrollIntoView({ block: "nearest" });
    } else
      list
        ?.querySelector<HTMLInputElement>(
          `[data-source-index="${props.store.sourceIndex()}"] input`,
        )
        ?.focus({ preventScroll: true });
  };
  const controller = (action: InputAction) => {
    if (!game()) return;
    if (action === "BUTTON_B") props.store.closeSources();
    else if (action === "NAV_RIGHT") focusPanel("details");
    else if (action === "NAV_LEFT") focusPanel("sources");
    else if (action === "NAV_UP" || action === "NAV_DOWN")
      move(action === "NAV_DOWN" ? 1 : -1);
    else if (action === "BUTTON_X") void toggleFavorite();
    else if (action === 'BUTTON_LB' && panel() === 'sources' && !currentJob() && props.store.releaseOptions().length > 1) moveRelease(-1);
    else if (action === 'BUTTON_RB' && panel() === 'sources' && !currentJob() && props.store.releaseOptions().length > 1) moveRelease(1);
    else if (action === 'BUTTON_LB' && localFiles().length) focusPanel('files');
    else if (action === 'BUTTON_LB' && currentJob()) void props.store.controlDownload(currentJob()!, 'pause');
    else if (action === 'BUTTON_A' && panel() === 'files') void chooseLocal();
    else if (action === 'BUTTON_RB' && ['paused', 'failed', 'downloaded'].includes(currentJob()?.status || '')) void props.store.controlDownload(currentJob()!, 'resume');
    else if (
      action === "BUTTON_Y" &&
      detailsGame()?.installed &&
      !props.playBlockReason
    )
      props.onPlay?.(detailsGame()!);
    else if (
      action === "BUTTON_A" &&
      panel() === "sources" &&
      selected()?.downloadable &&
      !busy()
    )
      props.onConfirm();
  };
  onMount(() => props.onControllerReady?.(controller));
  onCleanup(() => { localRequest++; props.onControllerReady?.(null); });
  return (
    <Dialog
      open={Boolean(props.store.sourceGame())}
      onOpenChange={(open) => {
        if (!open) props.store.closeSources();
      }}
    >
      <Dialog.Portal>
        <Dialog.Overlay class="console-modal-backdrop game-case-overlay" />
        <div class="console-modal-center-container">
          <Dialog.Content
            class="download-source-dialog game-case"
            ref={dialogContent}
            onOpenAutoFocus={(event) => {
              event.preventDefault();
              closeButton?.focus({ preventScroll: true });
              if (dialogContent) dialogContent.scrollTop = 0;
            }}
            onCloseAutoFocus={(event) => {
              event.preventDefault();
              const id = returnGameId;
              requestAnimationFrame(() => {
                if (id && !document.querySelector('[role="dialog"]'))
                  document
                    .getElementById(`shelf-card-${id}`)
                    ?.focus({ preventScroll: true });
              });
            }}
            onKeyDown={(event) => {
              event.stopPropagation();
              if (event.target instanceof HTMLSelectElement && event.key !== "Escape") return;
              if (event.key === "Escape") {
                event.preventDefault();
                props.store.closeSources();
              } else if (event.key === "ArrowDown" || event.key === "ArrowUp") {
                event.preventDefault();
                move(event.key === "ArrowDown" ? 1 : -1);
              } else if (
                event.key === "ArrowRight" ||
                event.key === "ArrowLeft"
              ) {
                event.preventDefault();
                focusPanel(event.key === "ArrowRight" ? "details" : "sources");
              } else if (event.key.toLowerCase() === "y") {
                event.preventDefault();
                controller("BUTTON_Y");
              } else if (
                event.key === "Enter" &&
                event.target instanceof HTMLInputElement
              ) {
                event.preventDefault();
                controller("BUTTON_A");
              }
            }}
          >
            <Dialog.CloseButton
              ref={closeButton}
              class="game-case-close"
              aria-label="Cerrar ficha"
              title="Cerrar ficha"
            >
              <X size={20} />
            </Dialog.CloseButton>
            <div class="game-case-left" onFocusIn={event => { if (!(event.target as HTMLElement).closest('.download-local-files')) setPanel("sources"); }}>
              <header class="game-case-summary">
                <div class="game-case-cover">
                  <Show
                    when={game()?.coverImage && !coverFailed()}
                    fallback={
                      <ConsoleHardwareVisual
                        platformId={game()?.platform || "all"}
                        size="lg"
                      />
                    }
                  >
                    <img
                      src={game()?.coverImage}
                      alt={game()?.title}
                      onError={() => setCoverFailed(true)}
                    />
                  </Show>
                </div>
                <div class="game-case-heading">
                  <span class="game-case-platform">{game()?.platformName}</span>
                  <Dialog.Title>{game()?.title}</Dialog.Title>
                  <Dialog.Description>
                    {detailsGame()?.installed ? "Instalado" : "No instalado"}
                  </Dialog.Description>
                  <button
                    class="game-case-favorite"
                    aria-pressed={Boolean(favorite())}
                    aria-label="Alternar favorito"
                    title="Alternar favorito"
                    onClick={() => {
                      void toggleFavorite();
                    }}
                  >
                    <Heart
                      size={18}
                      fill={favorite() ? "currentColor" : "none"}
                    />
                  </button>
                </div>
              </header>
              <Show when={props.store.releaseOptions().length > 0}>
                <div class="game-release-selector" aria-label="Versiones disponibles">
                  <div class="game-case-section-label">
                    <h3>Versiones</h3>
                    <span>{props.store.releaseOptions().length}</span>
                  </div>
                  <div class="game-release-track" role="tablist" aria-label="Versiones del juego">
                    <For each={props.store.releaseOptions()}>
                      {(release, index) => (
                        <button
                          role="tab"
                          aria-selected={props.store.releaseIndex() === index()}
                          classList={{ selected: props.store.releaseIndex() === index() }}
                          onClick={() => props.store.selectRelease(index())}
                          onKeyDown={(event) => {
                            if (event.key === "ArrowLeft" || event.key === "ArrowRight") {
                              event.preventDefault();
                              moveRelease(event.key === "ArrowRight" ? 1 : -1);
                            }
                          }}
                        >
                          <strong>{release.title}</strong>
                          <small>{release.region || "Region no indicada"}</small>
                          <small>{release.downloadableSourceCount}/{release.sourceCount} fuentes compatibles</small>
                          <Show when={release.installed}><Check size={15} aria-label="Instalada" /></Show>
                        </button>
                      )}
                    </For>
                  </div>
                </div>
              </Show>
              <div class="game-case-section-label">
                <h3>Distribuidores y paquetes</h3>
                <span>{props.store.sourceOptions().length} fuentes</span>
              </div>
              <Show when={props.store.sourcesLoading()}>
                <p class="game-case-state" role="status">
                  Consultando distribuidores...
                </p>
              </Show>
              <Show when={props.store.sourcesError()}>
                <p class="game-case-state" role="alert">
                  {props.store.sourcesError()}
                </p>
              </Show>
              <Show
                when={
                  !props.store.sourcesLoading() &&
                  !props.store.sourcesError() &&
                  props.store.sourceOptions().length === 0
                }
              >
                <p class="game-case-state" role="status">
                  No hay fuentes registradas.
                </p>
              </Show>
              <div
                class="download-source-list"
                role="radiogroup"
                aria-label="Distribuidores y paquetes"
                ref={list}
              >
                <For each={props.store.sourceOptions()}>
                  {(source, index) => (
                    <label
                      class={`download-source-option ${props.store.sourceIndex() === index() ? "selected" : ""}`}
                      data-source-index={index()}
                    >
                      <input
                        type="radio"
                        name="download-source"
                        checked={props.store.sourceIndex() === index()}
                        onChange={() => props.store.setSourceIndex(index())}
                      />
                      <span>
                        <strong>{source.name}</strong>
                        <span class="game-case-source-meta">
                          {host(source.uri)} · {labels[source.access]}
                        </span>
                        <Show when={source.sizeBytes}>
                          <span class="game-case-source-meta">
                            {((source.sizeBytes ?? 0) / 1024 / 1024).toFixed(1)}{" "}
                            MiB
                          </span>
                        </Show>
                        <Show when={source.reason}>
                          <span class="download-source-reason">
                            {source.reason}
                          </span>
                        </Show>
                      </span>
                    </label>
                  )}
                </For>
              </div>
              <Show when={favoriteError()}>
                <p class="game-case-state" role="alert">
                  {favoriteError()}
                </p>
              </Show>
              <footer class="download-source-actions">
                <Show when={currentJob()}>{job => <>
                  <Show when={job().status === 'downloading' || job().status === 'queued'}>
                    <button title="Pausar descarga" aria-label="Pausar descarga" onClick={() => void props.store.controlDownload(job(), 'pause')}><Pause size={18} /></button>
                    <button title="Cancelar descarga" aria-label="Cancelar descarga" onClick={() => void props.store.controlDownload(job(), 'cancel')}><Square size={18} /></button>
                  </Show>
                  <Show when={job().status === 'paused' || job().status === 'failed'}>
                    <button title="Reanudar descarga" aria-label="Reanudar descarga" onClick={() => void props.store.controlDownload(job(), 'resume')}><Play size={18} /></button>
                  </Show>
                  <Show when={job().status === 'downloaded'}>
                    <button title="Reintentar preparacion local" aria-label="Reintentar preparacion local" onClick={() => void props.store.controlDownload(job(), 'resume')}><RotateCw size={18} /></button>
                  </Show>
                </>}</Show>
                <button
                  class="game-case-download"
                  disabled={
                    !selected()?.downloadable ||
                    props.store.sourcesLoading() ||
                    busy()
                  }
                  onClick={props.onConfirm}
                >
                  <Download size={18} />
                  {busy() ? (currentJob()?.phase === 'preparing' ? "Preparando..." : "Descargando...") : "Descargar seleccionada"}
                </button>
                <Show when={detailsGame()?.installed && props.onPlay}>
                  <button
                    class="game-case-play"
                    disabled={Boolean(props.playBlockReason)}
                    title={props.playBlockReason || "Jugar"}
                    onClick={() => {
                      const current = detailsGame();
                      if (current && !props.playBlockReason)
                        props.onPlay?.(current);
                    }}
                  >
                    <Play size={18} />
                    Jugar
                  </button>
                </Show>
              </footer>
              <Show when={localLoading()}><p class="game-case-state" role="status">Consultando archivos locales...</p></Show>
              <Show when={localFiles().length}>
                <form class="download-local-files download-source-actions" onFocusIn={() => setPanel('files')} onSubmit={event => { event.preventDefault(); void chooseLocal(); }}>
                  <label for="download-launch-file">Archivo de lanzamiento</label>
                  <select id="download-launch-file" ref={localSelect} value={localFiles()[localIndex()] || ''} disabled={localSaving()} onChange={event => setLocalIndex(localFiles().indexOf(event.currentTarget.value))}>
                    <For each={localFiles()}>{path => <option value={path}>{path}</option>}</For>
                  </select>
                  <button type="submit" title="Usar archivo seleccionado" aria-label="Usar archivo seleccionado" disabled={localSaving() || localLoading()}><Check size={18} /></button>
                </form>
              </Show>
              <Show when={localError()}><p class="game-case-state" role="alert">{localError()}</p></Show>
              <Show when={currentJob()}>{job => <p class="game-case-state" role="status">{job().provider || 'Proveedor'} · {phases[job().phase || ''] || job().status} · {Math.round(job().progress * 100)}%<Show when={job().status === 'downloaded'}> · {job().error}</Show></p>}</Show>
              <Show when={currentJob()?.status === 'downloaded'}><span class="download-source-uri">{currentJob()?.destinationPath}</span></Show>
              <Show when={props.store.downloadError()?.gameId === game()?.id}><p class="game-case-state" role="alert">{props.store.downloadError()?.message}</p></Show>
              <Show when={detailsGame()?.installed && props.playBlockReason}>
                <p class="game-case-state" role="status">
                  {props.playBlockReason}
                </p>
              </Show>
            </div>
            <div
              class="game-case-right"
              data-active={panel() === "details"}
              tabIndex={0}
              role="region"
              aria-label="Descripcion y datos del juego"
              ref={details}
              onFocusIn={() => setPanel("details")}
            >
              <Show when={game()?.backdropImage}>
                <img
                  class="game-case-backdrop"
                  src={game()?.backdropImage}
                  alt=""
                  onError={(event) => {
                    event.currentTarget.hidden = true;
                  }}
                />
              </Show>
              <div class="game-case-details">
                <span class="game-case-platform">Ficha del juego</span>
                <h3>{game()?.title}</h3>
                <p class="game-case-description">
                  {detailsGame()?.description ||
                    game()?.description ||
                    "Descripcion no disponible."}
                </p>
                <dl class="game-case-facts">
                  <div>
                    <dt>Plataforma</dt>
                    <dd>{game()?.platformName}</dd>
                  </div>
                  <div>
                    <dt>Lanzamiento</dt>
                    <dd>
                      {detailsGame()?.releaseYear ||
                        game()?.releaseYear ||
                        "No disponible"}
                    </dd>
                  </div>
                  <div>
                    <dt>Desarrollador</dt>
                    <dd>
                      {detailsGame()?.developer ||
                        game()?.developer ||
                        "No disponible"}
                    </dd>
                  </div>
                  <div>
                    <dt>Editor</dt>
                    <dd>
                      {detailsGame()?.publisher ||
                        game()?.publisher ||
                        "No disponible"}
                    </dd>
                  </div>
                  <div>
                    <dt>Genero</dt>
                    <dd>
                      {detailsGame()?.genre || game()?.genre || "No disponible"}
                    </dd>
                  </div>
                  <div>
                    <dt>Valoracion</dt>
                    <dd>
                      {detailsGame()?.rating
                        ? `${detailsGame()!.rating.toFixed(1)} / 5`
                        : "No disponible"}
                    </dd>
                  </div>
                  <div>
                    <dt>Tiempo jugado</dt>
                    <dd>{game()?.playTimeMinutes || 0} min</dd>
                  </div>
                </dl>
                <Show when={selected()}>
                  {(source) => (
                    <section class="game-case-package">
                      <h4>Paquete seleccionado</h4>
                      <p>{source().name}</p>
                      <p>{labels[source().access]}</p>
                      <span class="download-source-uri">{source().uri}</span>
                      <Show when={source().reason}>
                        <p class="download-source-reason">{source().reason}</p>
                      </Show>
                    </section>
                  )}
                </Show>
              </div>
            </div>
          </Dialog.Content>
        </div>
      </Dialog.Portal>
    </Dialog>
  );
}

import { createSignal, For, Show } from "solid-js";
import {
  ArrowLeft,
  ChevronDown,
  ChevronLeft,
  ChevronRight,
  ChevronUp,
  Download,
  Folder,
  Gamepad2,
  Heart,
  Keyboard,
  Layers,
  LoaderCircle,
  Play,
  Search,
  Settings2,
  Trash2,
} from "lucide-solid";
import type { XmbLibraryProps } from "@contracts/xmb.types";
import type { Game } from "@contracts/game.types";
import { useXmbLibrary } from "@hooks/useXmbLibrary";
import { ConsoleHardwareVisual } from "@components/common/ConsoleHardwareVisual";
import { catalogTitle, cleanGameDescription } from "@services/library/catalog-groups";
import { xmbDownloadStatus } from "@services/library/xmb-navigation";

export function XmbLibrary(props: XmbLibraryProps) {
  const downloadStatus = (entry: Game) =>
    xmbDownloadStatus(entry, props.downloadJobs, props.downloadingIds);
  const isInstalled = (entry: Game) =>
    Boolean(
      entry.installed ||
        entry.romPath ||
        downloadStatus(entry) === "Instalada" ||
        downloadStatus(entry) === "Instalado",
    );
  const {
    position,
    query,
    clock,
    coverFailed,
    virtualKeyboardOpen,
    virtualKeyboardIndex,
    virtualKeyboardKeys,
    categories,
    category,
    folders,
    rows,
    folder,
    selectedGame,
    rowWindow,
    gameWindow,
    navigate,
    chooseCategory,
    chooseRow,
    chooseGame,
    scrollWheel,
    search,
    openSearch,
    closeVirtualKeyboard,
    applyVirtualKey,
    onSearchKeyDown,
    bindSearchInput,
    bindDetailPanel,
    onCoverError,
  } = useXmbLibrary(props);

  return (
    <main
      class="xmb"
      classList={{
        "xmb-expanded": position().expanded,
        "xmb-in-list": position().row > 0 || category().kind === 'settings',
        "xmb-settings": category().kind === 'settings',
      }}
      data-category={category().id}
      onKeyDown={(event) => {
        if (
          event.key === "Tab" ||
          (event.key === "Enter" && event.target instanceof HTMLButtonElement)
        )
          event.stopPropagation();
        if (
          !virtualKeyboardOpen() &&
          (event.key === "/" || (event.ctrlKey && (event.key === "f" || event.key === "F")))
        ) {
          event.preventDefault();
          openSearch();
          return;
        }
        if (position().expanded && selectedGame()) {
          if (event.key === "Delete") {
            props.onUninstall?.(selectedGame()!.id);
          } else if (event.key === "f" || event.key === "F") {
            props.onFavorite(selectedGame()!.id);
          }
        }
      }}
      onWheel={(event) => {
        const target = event.target;
        if (event.ctrlKey || !(target instanceof Element) || target.closest(".xmb-detail-scroll, .xmb-settings-panel, .xmb-virtual-keyboard, input, textarea, select")) return;
        const horizontal = event.shiftKey || Math.abs(event.deltaX) > Math.abs(event.deltaY);
        const surface = target.closest(".xmb-category-rail") ? "categories"
          : target.closest(".xmb-game-shelf") ? "versions"
          : horizontal ? position().expanded ? "versions" : "categories" : "folders";
        const delta = horizontal && event.deltaX ? event.deltaX : event.deltaY;
        scrollWheel(surface, delta, event.deltaMode);
      }}
    >
      <div class="xmb-wave" />
      <header class="xmb-topbar">
        <div class="xmb-brand">
          <strong>EMUBOX</strong>
          <span>/</span>
          <span>{category().title}</span>
          <Show when={folder()}>
            {(f) => (
              <>
                <span>/</span>
                <span>{f().title}</span>
              </>
            )}
          </Show>
        </div>
        <div class="xmb-top-actions">
          <Show
            when={props.inputStatus.source === "keyboard"}
            fallback={
              <button
                class="xmb-search"
                title="Buscar juegos"
                aria-label="Buscar juegos"
                onClick={openSearch}
              >
                <Keyboard size={17} />
                <span>{query() || "Buscar"}</span>
                <span class="xmb-key-badge gamepad gold">(Y)</span>
              </button>
            }
          >
            <label class="xmb-search">
              <Search size={17} />
              <input
                ref={bindSearchInput}
                type="search"
                value={query()}
                placeholder="Buscar"
                aria-label="Buscar juegos"
                onInput={(event) => search(event.currentTarget.value)}
                onKeyDown={onSearchKeyDown}
              />
              <span class="xmb-key-badge kbd">/</span>
            </label>
          </Show>
          <time>{clock()}</time>
        </div>
      </header>

      <Show when={props.loading}>
        <div class="xmb-load-status" role="status">
          <LoaderCircle size={16} class="spin" />
          <span>{props.loadingMessage || "Cargando..."}</span>
        </div>
      </Show>

      <Show when={virtualKeyboardOpen()}>
        <section class="xmb-virtual-keyboard" aria-label="Teclado en pantalla">
          <header>
            <Keyboard size={16} />
            <strong>Buscar en catalogo</strong>
            <button
              class="xmb-icon-button"
              title="Cerrar teclado"
              aria-label="Cerrar teclado"
              onClick={closeVirtualKeyboard}
            >
              Cerrar
            </button>
          </header>
          <div class="xmb-search">
            <Search size={16} />
            <input
              ref={bindSearchInput}
              value={query()}
              placeholder="Escribe el nombre del juego..."
              onInput={(event) => search(event.currentTarget.value)}
              onKeyDown={onSearchKeyDown}
            />
          </div>
          <div class="xmb-virtual-keyboard-grid">
            <For each={virtualKeyboardKeys}>
              {(key, index) => (
                <button
                  type="button"
                  class="xmb-virtual-key"
                  classList={{ active: index() === virtualKeyboardIndex() }}
                  onClick={() => applyVirtualKey(key)}
                >
                  {key.label}
                </button>
              )}
            </For>
          </div>
        </section>
      </Show>

      <nav class="xmb-category-rail" aria-label="Categorias principales">
        <div
          class="xmb-category-track"
          style={{ "--category-index": position().category }}
        >
          <For each={categories()}>
            {(cat, index) => (
              <button
                class="xmb-category"
                classList={{
                  active: index() === position().category,
                  selected: index() === position().category,
                }}
                title={cat.title}
                aria-label={cat.title}
                onClick={() => chooseCategory(index())}
              >
                <span class="xmb-category-icon">
                  {cat.id === "favorites" ? (
                    <Heart size={28} />
                  ) : cat.id === "installed" ? (
                    <Download size={28} />
                  ) : cat.id === "all" ? (
                    <Layers size={28} />
                  ) : cat.kind === "settings" ? (
                    <Settings2 size={28} />
                  ) : (
                    <ConsoleHardwareVisual platformId={cat.id} size="md" />
                  )}
                </span>
                <span>{cat.title}</span>
              </button>
            )}
          </For>
        </div>
      </nav>

      <section class="xmb-category-intro" aria-label="Informacion de categoria">
        <h1>{category().title}</h1>
        <p>
          {category().kind === "settings"
            ? "Ajustes del sistema y configuracion."
            : `${folders().length.toLocaleString("es-ES")} juegos disponibles`}
        </p>
      </section>

      <Show
        when={category().kind === "settings"}
        fallback={
          <section
            class="xmb-folder-viewport"
            aria-label="Lista vertical de juegos"
          >
            <div class="xmb-folder-rail">
              <For each={rowWindow()}>
                {(rowNumber) => {
                  const entry = () => folders()[rowNumber - 1];
                  const offset = () => rowNumber - position().row;
                  const isActive = () => offset() === 0;
                  return (
                    <div
                      class="xmb-folder-row"
                      classList={{ active: isActive() }}
                      style={{ "--row-offset": offset() }}
                    >
                      <button
                        class="xmb-folder-button xmb-folder-node"
                        title={entry()?.title || "Elemento"}
                        aria-label={entry()?.title || "Elemento"}
                        onClick={() => chooseRow(rowNumber)}
                      >
                        <Show
                          when={entry()?.games[0]?.coverImage && !coverFailed()}
                          fallback={
                            entry()?.games[0]?.platform ? (
                              <ConsoleHardwareVisual
                                platformId={entry()!.games[0].platform}
                                size="md"
                              />
                            ) : (
                              <Folder size={32} />
                            )
                          }
                        >
                          <img
                            src={entry()!.games[0].coverImage}
                            alt=""
                            loading="lazy"
                            decoding="async"
                            onError={onCoverError}
                          />
                        </Show>
                      </button>
                      <Show when={isActive() && !position().expanded}>
                        <div class="xmb-folder-label">
                          <h2>{entry()?.title}</h2>
                          <p>
                            {entry()?.games.length || 0} versiones disponibles
                          </p>
                        </div>
                      </Show>
                    </div>
                  );
                }}
              </For>
            </div>
          </section>
        }
      >
        <section class="xmb-settings-panel" aria-label="Panel de configuracion">
          {props.settingsPanel}
        </section>
      </Show>

      <Show when={position().expanded && selectedGame()}>
        {(game) => (
          <>
            <section class="xmb-game-shelf" aria-label="Versiones del juego">
              <div class="xmb-tile-row">
                <For each={gameWindow()}>
                  {(entry) => {
                    const offset = () => entry.index - position().game;
                    const isSelected = () => entry.index === position().game;
                    const [imageFailed, setImageFailed] = createSignal(false);
                    return (
                      <div
                        class="xmb-game-tile"
                        classList={{ selected: isSelected() }}
                        style={{ "--game-offset": offset() }}
                        onClick={() => chooseGame(entry.index)}
                        role="button"
                        tabIndex={0}
                        aria-label={`${entry.game.title} (${entry.game.platformName})`}
                        onKeyDown={(event) => {
                          if (event.key === "Enter" || event.key === " ") {
                            event.preventDefault();
                            props.onOpenGame(entry.game);
                          }
                        }}
                      >
                        <Show when={entry.index === position().game && position().game > 0}>
                          <span class="xmb-more-left" aria-hidden="true">
                            <ChevronLeft size={20} />
                          </span>
                        </Show>
                        <Show when={entry.index === position().game && position().game + 1 < (folder()?.games.length || 0)}>
                          <span class="xmb-more-right" aria-hidden="true">
                            <ChevronRight size={20} />
                          </span>
                        </Show>
                        <div class="xmb-tile-art">
                          <Show
                            when={entry.game.coverImage && !imageFailed()}
                            fallback={
                              <ConsoleHardwareVisual
                                platformId={entry.game.platform}
                                size="md"
                              />
                            }
                          >
                            <img
                              src={entry.game.coverImage}
                              alt=""
                              loading="lazy"
                              decoding="async"
                              onError={() => setImageFailed(true)}
                            />
                          </Show>
                        </div>
                        <div class="xmb-tile-overlay">
                          <strong>{entry.game.title}</strong>
                          <div class="xmb-tile-sub">
                            <span>{entry.game.platformName}</span>
                            <span>{downloadStatus(entry.game)}</span>
                          </div>
                        </div>
                        <Show when={entry.index === position().game}>
                          <div class="xmb-tile-halo-actions">
                            <button
                              type="button"
                              class="xmb-tile-action-open"
                              title={isInstalled(entry.game) ? "Jugar juego" : "Instalar juego"}
                              onClick={(e) => {
                                e.stopPropagation();
                                props.onOpenGame(entry.game);
                              }}
                            >
                              <span class="xmb-tile-pad-pill">A</span>
                              <span>{isInstalled(entry.game) ? "Jugar" : "Instalar"}</span>
                            </button>
                            <button
                              type="button"
                              class="xmb-tile-action-fav"
                              title="Alternar favorito"
                              aria-pressed={entry.game.favorite}
                              onClick={(e) => {
                                e.stopPropagation();
                                props.onFavorite(entry.game.id);
                              }}
                            >
                              <span class="xmb-tile-pad-pill">Y</span>
                              <Heart
                                size={14}
                                fill={entry.game.favorite ? "currentColor" : "none"}
                              />
                            </button>
                          </div>
                        </Show>
                      </div>
                    );
                  }}
                </For>
              </div>
            </section>

            <aside class="xmb-detail-drawer" aria-label="Interior de la carátula">
              <div class="xmb-drawer-ambient-art" aria-hidden="true">
                <Show when={game().backdropImage || game().coverImage}>
                  <img
                    src={game().backdropImage || game().coverImage}
                    alt=""
                    loading="lazy"
                    decoding="async"
                  />
                </Show>
                <div class="xmb-drawer-ambient-overlay" />
              </div>

              <header class="xmb-case-topbar">
                <h2 class="xmb-case-title">
                  {catalogTitle(game().canonicalTitle || game().title)}
                </h2>
                <button
                  class="xmb-drawer-close-btn"
                  title="Cerrar ficha (B / Esc)"
                  aria-label="Cerrar ficha"
                  onClick={() => navigate("back")}
                >
                  <span class="xmb-key-badge gamepad">(B)</span>
                  <span class="xmb-key-badge kbd">Esc</span>
                  <ArrowLeft size={16} />
                </button>
              </header>

              <div
                class="xmb-detail-scroll"
                ref={bindDetailPanel}
                tabIndex={0}
                onKeyDown={(event) => {
                  if (
                    [
                      "ArrowUp",
                      "ArrowDown",
                      "PageUp",
                      "PageDown",
                      "Home",
                      "End",
                      " ",
                    ].includes(event.key)
                  )
                    event.stopPropagation();
                }}
              >
                <div class="xmb-case-description-box">
                  <p class="xmb-case-description">
                    {cleanGameDescription(game().description)}
                  </p>
                </div>

                <div class="xmb-case-info-section">
                  <span class="xmb-case-section-label">Datos de la versión</span>
                  <dl class="xmb-case-facts">
                    <div>
                      <dt>PLATAFORMA</dt>
                      <dd>{game().platformName || game().platform}</dd>
                    </div>
                    <div>
                      <dt>LANZAMIENTO</dt>
                      <dd>{game().releaseYear || "No disponible"}</dd>
                    </div>
                    <div>
                      <dt>DESARROLLADOR</dt>
                      <dd>{game().developer || "No disponible"}</dd>
                    </div>
                    <div>
                      <dt>EDITOR</dt>
                      <dd>{game().publisher || "No disponible"}</dd>
                    </div>
                    <div>
                      <dt>GÉNERO</dt>
                      <dd>{game().genre || "No disponible"}</dd>
                    </div>
                    <div>
                      <dt>VALORACIÓN</dt>
                      <dd>
                        {game().rating
                          ? `${game().rating.toFixed(1)} / 5`
                          : "No disponible"}
                      </dd>
                    </div>
                    <div>
                      <dt>TIEMPO JUGADO</dt>
                      <dd>{game().playTimeMinutes || 0} min</dd>
                    </div>
                    <div>
                      <dt>ESTADO</dt>
                      <dd>{downloadStatus(game())}</dd>
                    </div>
                  </dl>
                </div>
              </div>

              <footer class="xmb-case-actions">
                <button
                  class="xmb-case-btn-primary"
                  title={
                    isInstalled(game())
                      ? "Jugar juego (A / Enter)"
                      : "Instalar juego (A / Enter)"
                  }
                  onClick={() => props.onOpenGame(game())}
                >
                  <span class="xmb-key-badge gamepad">(A)</span>
                  <span class="xmb-key-badge kbd">Enter</span>
                  <Show
                    when={isInstalled(game())}
                    fallback={<Download size={18} />}
                  >
                    <Play size={18} fill="currentColor" />
                  </Show>
                  <span>{isInstalled(game()) ? "Jugar" : "Instalar"}</span>
                </button>

                <Show when={isInstalled(game())}>
                  <button
                    class="xmb-case-btn-uninstall"
                    title="Desinstalar juego (Supr / Del)"
                    onClick={() => props.onUninstall?.(game().id)}
                  >
                    <span class="xmb-key-badge gamepad danger">(X)</span>
                    <span class="xmb-key-badge kbd">Supr</span>
                    <Trash2 size={16} />
                    <span>Desinstalar</span>
                  </button>
                </Show>

                <button
                  class="xmb-case-btn-fav"
                  title="Alternar favorito (Y / F)"
                  aria-label="Alternar favorito"
                  aria-pressed={game().favorite}
                  onClick={() => props.onFavorite(game().id)}
                >
                  <span class="xmb-key-badge gamepad gold">(Y)</span>
                  <span class="xmb-key-badge kbd">F</span>
                  <Heart
                    size={18}
                    fill={game().favorite ? "currentColor" : "none"}
                  />
                </button>
              </footer>
            </aside>
          </>
        )}
      </Show>
      <Show when={props.error}>
        <div class="xmb-error" role="alert">
          {props.error?.message}
        </div>
      </Show>
      <footer class="xmb-bottom">
        <div class="xmb-step-controls">
          <button
            title={
              position().expanded ? "Version anterior" : "Categoria anterior"
            }
            aria-label={
              position().expanded ? "Version anterior" : "Categoria anterior"
            }
            onClick={() => navigate("left")}
          >
            <ChevronLeft size={19} />
          </button>
          <button
            title="Anterior"
            aria-label="Anterior"
            onClick={() => navigate("up")}
          >
            <ChevronUp size={19} />
          </button>
          <span>
            {position().row} / {rows().toLocaleString("es-ES")}
          </span>
          <button
            title="Siguiente"
            aria-label="Siguiente"
            onClick={() => navigate("down")}
          >
            <ChevronDown size={19} />
          </button>
          <button
            title={
              position().expanded ? "Version siguiente" : "Categoria siguiente"
            }
            aria-label={
              position().expanded ? "Version siguiente" : "Categoria siguiente"
            }
            onClick={() => navigate("right")}
          >
            <ChevronRight size={19} />
          </button>
        </div>
        <span class="xmb-input-status">
          <Gamepad2 size={16} />
          {props.inputStatus.deviceName}
        </span>
      </footer>
    </main>
  );
}

import { For, Show } from "solid-js";
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
  Layers,
  LoaderCircle,
  Monitor,
  Search,
  Settings2,
} from "lucide-solid";
import type { XmbLibraryProps } from "@contracts/xmb.types";
import { useXmbLibrary } from "@hooks/useXmbLibrary";
import { ConsoleHardwareVisual } from "@components/common/ConsoleHardwareVisual";

export function XmbLibrary(props: XmbLibraryProps) {
  const {
    position,
    query,
    clock,
    coverFailed,
    categories,
    category,
    folders,
    settingItems,
    rows,
    folder,
    selectedGame,
    rowWindow,
    gameWindow,
    packages,
    navigate,
    chooseCategory,
    chooseRow,
    chooseGame,
    search,
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
      }}
    >
      <div class="xmb-atmosphere" aria-hidden="true">
        <div class="xmb-wave" />
      </div>
      <header class="xmb-topbar">
        <div class="xmb-brand">
          <strong>EMUBOX</strong>
          <span>/</span>
          <span>{category().title}</span>
          <Show when={position().expanded && folder()}>
            <span>/</span>
            <span>{folder()?.title}</span>
          </Show>
        </div>
        <div class="xmb-load-status" role="status" aria-live="polite" aria-atomic="true">
          <Show when={props.loading} fallback={props.loadError}>
            <LoaderCircle class="xmb-loading-spinner" size={16} aria-hidden="true" />
            <span>{props.loadingMessage || 'Cargando biblioteca...'}</span>
          </Show>
        </div>
        <div class="xmb-top-actions">
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
          </label>
          <time>{clock()}</time>
        </div>
      </header>
      <nav
        class="xmb-category-rail"
        aria-label="Categorias"
        style={{ "--category-index": position().category }}
      >
        <div class="xmb-category-track">
          <For each={categories()}>
            {(item, index) => (
              <button
                class="xmb-category"
                classList={{ selected: index() === position().category }}
                title={item.title}
                aria-label={item.title}
                tabIndex={index() === position().category ? 0 : -1}
                aria-current={
                  index() === position().category ? "page" : undefined
                }
                onClick={() => chooseCategory(index())}
              >
                <Show
                  when={item.kind === "platform"}
                  fallback={
                    item.kind === "settings" ? (
                      <Settings2 />
                    ) : item.kind === "favorites" ? (
                      <Heart />
                    ) : item.kind === "installed" ? (
                      <Download />
                    ) : (
                      <Layers />
                    )
                  }
                >
                  <Show
                    when={item.id !== "pc" && item.id !== "linux"}
                    fallback={<Monitor />}
                  >
                    <ConsoleHardwareVisual platformId={item.id} size="sm" />
                  </Show>
                </Show>
                <span>{item.title}</span>
              </button>
            )}
          </For>
        </div>
      </nav>
      <Show when={position().row === 0 && category().kind !== 'settings'}>
        <section class="xmb-category-intro">
          <span class="xmb-eyebrow">
            {category().kind === "settings" ? "Consola" : "Biblioteca"}
          </span>
          <h1>{category().title}</h1>
          <p>
            {category().kind === "settings"
              ? `${rows()} secciones`
              : `${folders().length.toLocaleString("es-ES")} juegos · ${packages().toLocaleString("es-ES")} versiones`}
          </p>
          <Show when={rows() === 0}>
            <p role="status">
              {props.loading
                ? "Cargando biblioteca..."
                : "Sin juegos en esta categoria"}
            </p>
          </Show>
        </section>
      </Show>
      <Show when={category().kind === 'settings'}>
        <section class="xmb-settings-panel" aria-label="Panel de ajustes" classList={{ 'is-active': props.settingsActive }}>
          <Show when={props.settingsActive} fallback={
            <button class="xmb-settings-entry" onClick={() => props.onOpenSettings('system')}>
              <Settings2 size={32} aria-hidden="true" />
              <strong>Ajustes</strong>
              <span>Sistema y pantalla · Audio general · Controles · Tiendas</span>
              <ChevronDown size={22} aria-hidden="true" />
            </button>
          }>{props.settingsPanel}</Show>
        </section>
      </Show>
      <Show when={category().kind !== 'settings'}>
      <section
        class="xmb-folder-viewport"
        aria-label={
          category().kind === "settings" ? "Secciones de ajustes" : "Titulos"
        }
      >
        <For each={rowWindow()}>
          {(row) => {
            const current = () =>
              category().kind === "settings"
                ? settingItems[row - 1]
                : folders()[row - 1];
            const active = () => position().row === row;
            return (
              <div
                class="xmb-folder-row"
                classList={{
                  active: active(),
                  expanded: active() && position().expanded,
                }}
                style={{ "--row-offset": row - position().row }}
              >
                <button
                  class="xmb-folder-button"
                  aria-label={current()?.title}
                  aria-expanded={active() && position().expanded}
                  tabIndex={
                    active() || (position().row === 0 && row === 1) ? 0 : -1
                  }
                  onClick={() => chooseRow(row)}
                >
                  <ChevronUp
                    class="xmb-above"
                    size={12}
                    style={{ visibility: row > 1 ? "visible" : "hidden" }}
                  />
                  <Folder size={26} fill="currentColor" strokeWidth={1.4} />
                  <ChevronDown
                    class="xmb-below"
                    size={12}
                    style={{ visibility: row < rows() ? "visible" : "hidden" }}
                  />
                </button>
                <Show when={active() && !position().expanded}>
                  <div class="xmb-folder-label">
                    <h2>{current()?.title}</h2>
                    <p>
                      {category().kind === "settings"
                        ? settingItems[row - 1]?.detail
                        : `${folders()[row - 1]?.games.length || 0} versiones disponibles`}
                    </p>
                  </div>
                </Show>
              </div>
            );
          }}
        </For>
      </section>
      </Show>
      <Show when={position().expanded && selectedGame()}>
        {(game) => (
          <>
            <section
              class="xmb-game-shelf"
              aria-label="Versiones del juego"
            >
              <header class="xmb-version-heading">
                <strong>Versiones</strong>
                <span>{position().game + 1} / {folder()?.games.length || 0}</span>
              </header>
              <div class="xmb-version-track">
              <For each={gameWindow()}>
                {(entry) => (
                  <button
                    id={`shelf-card-${entry.game.id}`}
                    class="xmb-game-tile"
                    tabIndex={entry.index === position().game ? 0 : -1}
                    classList={{ selected: entry.index === position().game }}
                    style={{ "--game-offset": entry.index - position().game }}
                    title={entry.game.title}
                    onClick={() => chooseGame(entry.index)}
                  >
                    <Show
                      when={entry.game.coverImage}
                      fallback={
                        <ConsoleHardwareVisual
                          platformId={entry.game.platform}
                          size="sm"
                        />
                      }
                    >
                      <img
                        src={entry.game.coverImage}
                        alt=""
                        loading="lazy"
                        onError={(event) => {
                          event.currentTarget.style.visibility = "hidden";
                        }}
                      />
                    </Show>
                    <span>
                      <strong>{entry.game.title}</strong>
                      <small>{entry.game.platformName}</small>
                      <small>{props.downloadingIds.has(entry.game.id) ? 'Descargando' : entry.game.installed ? 'Instalada' : 'Sin instalar'}</small>
                    </span>
                  </button>
                )}
              </For>
              </div>
            </section>
            <aside class="xmb-detail-drawer" aria-label="Ficha del juego">
              <header>
                <span class="xmb-eyebrow">{game().platformName}</span>
                <button
                  class="xmb-icon-button"
                  title="Cerrar ficha"
                  aria-label="Cerrar ficha"
                  onClick={() => navigate("back")}
                >
                  <ArrowLeft size={18} />
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
                <div class="xmb-boxart">
                  <Show
                    when={game().coverImage && !coverFailed()}
                    fallback={
                      <ConsoleHardwareVisual
                        platformId={game().platform}
                        size="lg"
                      />
                    }
                  >
                    <img
                      src={game().coverImage}
                      alt={game().title}
                      onError={onCoverError}
                    />
                  </Show>
                </div>
                <div class="xmb-title-line">
                  <h2>{game().canonicalTitle || game().title}</h2>
                  <button
                    class="xmb-icon-button"
                    title="Alternar favorito"
                    aria-label="Alternar favorito"
                    aria-pressed={game().favorite}
                    onClick={() => props.onFavorite(game().id)}
                  >
                    <Heart
                      size={19}
                      fill={game().favorite ? "currentColor" : "none"}
                    />
                  </button>
                </div>
                <p class="xmb-description">
                  {game().description || "Descripcion no disponible."}
                </p>
                <dl class="xmb-facts">
                  <div>
                    <dt>Genero</dt>
                    <dd>{game().genre || "No disponible"}</dd>
                  </div>
                  <div>
                    <dt>Lanzamiento</dt>
                    <dd>{game().releaseYear || "No disponible"}</dd>
                  </div>
                  <div>
                    <dt>Desarrollador</dt>
                    <dd>{game().developer || "No disponible"}</dd>
                  </div>
                  <div>
                    <dt>Version seleccionada</dt>
                    <dd>{game().releaseTitle || game().title}</dd>
                  </div>
                  <div>
                    <dt>Identificacion</dt>
                    <dd>{game().matchMethod?.startsWith('libretro-') ? 'Libretro Database' : 'Catalogo local'}</dd>
                  </div>
                  <div>
                    <dt>Estado</dt>
                    <dd>
                      {props.downloadingIds.has(game().id)
                        ? "Descargando"
                        : game().installed
                          ? "Instalado"
                          : "No instalado"}
                    </dd>
                  </div>
                  <div>
                    <dt>Tiempo jugado</dt>
                    <dd>{game().playTimeMinutes || 0} min</dd>
                  </div>
                </dl>
              </div>
              <footer>
                <button
                  class="xmb-primary"
                  onClick={() => props.onOpenGame(game())}
                >
                  <Download size={18} />
                  Fuentes de esta version
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

import { For, Show, createEffect, createMemo, createSignal, on, onCleanup, onMount } from 'solid-js';
import { ArrowLeft, ChevronDown, ChevronLeft, ChevronRight, ChevronUp, Download, Folder, Gamepad2, Heart, Layers, Monitor, Search, Settings2, SlidersHorizontal, Wrench } from 'lucide-solid';
import type { CatalogGroup } from '@services/library/catalog-groups';
import { moveXmb, xmbFolders, type XmbCommand, type XmbPosition } from '@services/library/xmb-navigation';
import type { Game, Platform } from '@contracts/game.types';
import type { InputAction, InputDeviceStatus } from '@contracts/input.types';
import { SETTINGS_TABS } from '@contracts/settings.types';
import { ConsoleHardwareVisual } from '@components/common/ConsoleHardwareVisual';

interface XmbLibraryProps {
  games: CatalogGroup[];
  platforms: Platform[];
  loading: boolean;
  downloadingIds: Set<string>;
  inputStatus: InputDeviceStatus;
  error?: { gameId: string; message: string } | null;
  onOpenGame: (game: Game) => void;
  onOpenSettings: (tab: string) => void;
  onMaintenance: () => void;
  onFavorite: (id: string) => void;
  onMove: () => void;
  onControllerReady: (handler: ((action: InputAction) => void) | null) => void;
}

export function XmbLibrary(props: XmbLibraryProps) {
  const [position, setPosition] = createSignal<XmbPosition>({ category: 1, row: 0, expanded: false, game: 0 });
  const [query, setQuery] = createSignal('');
  const [clock, setClock] = createSignal('');
  const [coverFailed, setCoverFailed] = createSignal(false);
  let searchInput: HTMLInputElement | undefined;
  let detailPanel: HTMLDivElement | undefined;
  const categories = createMemo(() => {
    const platforms = new Map(props.platforms.filter(platform => platform.id !== 'all').map(platform => [platform.id, platform.name]));
    for (const game of props.games) if (!platforms.has(game.platform)) platforms.set(game.platform, game.platformName || game.platform);
    return [{ id: 'settings', title: 'Ajustes', kind: 'settings' }, { id: 'all', title: 'Todos los juegos', kind: 'all' },
      { id: 'favorites', title: 'Favoritos', kind: 'favorites' }, { id: 'installed', title: 'Instalados', kind: 'installed' },
      ...[...platforms].map(([id, title]) => ({ id, title, kind: 'platform' }))];
  });
  const categoryIndex = createMemo(() => position().category);
  const category = createMemo(() => categories()[categoryIndex()] || categories()[1]);
  const filtered = createMemo(() => {
    const current = category();
    const search = query().trim().toLocaleLowerCase();
    if (current.kind === 'settings') return [];
    return props.games.filter(game => (current.kind !== 'platform' || game.platform === current.id)
      && (current.kind !== 'favorites' || game.favorite) && (current.kind !== 'installed' || game.installed)
      && (!search || game.title.toLocaleLowerCase().includes(search)));
  });
  const folders = createMemo(() => xmbFolders(filtered()));
  const settingItems = [...SETTINGS_TABS.map(tab => ({ id: tab.id, title: tab.name, detail: tab.desc })),
    { id: 'maintenance', title: 'Mantenimiento', detail: 'Diagnostico, red y consola' }];
  const rows = () => category().kind === 'settings' ? settingItems.length : folders().length;
  const folder = () => folders()[position().row - 1];
  const selectedGame = () => folder()?.games[position().game];
  const rowWindow = createMemo(() => {
    const current = position().row;
    const start = Math.max(1, current - 3);
    const end = Math.min(rows(), Math.max(6, current + 3));
    return Array.from({ length: Math.max(0, end - start + 1) }, (_, index) => start + index);
  });
  const gameWindow = createMemo(() => {
    const list = folder()?.games || [];
    const start = Math.max(0, position().game - 1);
    return list.slice(start, position().game + 4).map((game, offset) => ({ game, index: start + offset }));
  });
  const packages = createMemo(() => filtered().reduce((total, game) => total + game.variants.length, 0));
  createEffect(() => {
    const maxRows = rows();
    const current = position();
    if (current.row > maxRows) setPosition({ ...current, row: maxRows, expanded: false, game: 0 });
    else if (current.expanded && !selectedGame()) setPosition({ ...current, expanded: false, game: 0 });
  });
  createEffect(on(() => selectedGame()?.id, () => {
    setCoverFailed(false);
    if (detailPanel) detailPanel.scrollTop = 0;
  }));
  const navigate = (command: XmbCommand) => {
    if (command === 'enter' && category().kind === 'settings' && position().row > 0) {
      const item = settingItems[position().row - 1];
      if (item.id === 'maintenance') props.onMaintenance();
      else props.onOpenSettings(item.id);
      return;
    }
    if (command === 'enter' && position().expanded && selectedGame()) { props.onOpenGame(selectedGame()!); return; }
    setPosition(previous => moveXmb(previous, command, { categories: categories().length, rows: rows(), games: folder()?.games.length || 0 }));
    props.onMove();
  };
  const chooseCategory = (index: number) => {
    setPosition({ category: index, row: 0, expanded: false, game: 0 });
    props.onMove();
  };
  const controller = (action: InputAction) => {
    const commands: Partial<Record<InputAction, XmbCommand>> = { NAV_LEFT: 'left', NAV_RIGHT: 'right', NAV_UP: 'up', NAV_DOWN: 'down', BUTTON_A: 'enter', BUTTON_B: 'back' };
    if (commands[action]) navigate(commands[action]!);
    else if (action === 'BUTTON_LB' || action === 'BUTTON_RB') chooseCategory(Math.max(0, Math.min(categories().length - 1, position().category + (action === 'BUTTON_RB' ? 1 : -1))));
    else if (action === 'BUTTON_X' && selectedGame()) props.onFavorite(selectedGame()!.id);
    else if ((action === 'BUTTON_LT' || action === 'BUTTON_RT') && position().expanded) detailPanel?.scrollBy({ top: action === 'BUTTON_RT' ? 160 : -160 });
    else if (action === 'BUTTON_Y') searchInput?.focus();
    else if (action === 'BUTTON_START' || action === 'HOME') chooseCategory(0);
  };
  onMount(() => {
    props.onControllerReady(controller);
    const updateClock = () => setClock(new Date().toLocaleTimeString('es-ES', { hour: '2-digit', minute: '2-digit' }));
    updateClock();
    const timer = setInterval(updateClock, 10000);
    onCleanup(() => { clearInterval(timer); props.onControllerReady(null); });
  });

  return (
    <main class="xmb" classList={{ 'xmb-expanded': position().expanded, 'xmb-in-list': position().row > 0 }} data-category={category().id}
      onKeyDown={event => { if (event.key === 'Tab' || (event.key === 'Enter' && event.target instanceof HTMLButtonElement)) event.stopPropagation(); }}>
      <div class="xmb-atmosphere" aria-hidden="true"><div class="xmb-wave" /></div>
      <header class="xmb-topbar">
        <div class="xmb-brand"><strong>EMUBOX</strong><span>/</span><span>{category().title}</span></div>
        <div class="xmb-top-actions">
          <label class="xmb-search"><Search size={17} /><input ref={searchInput} type="search" value={query()} placeholder="Buscar" aria-label="Buscar juegos"
            onInput={event => { setQuery(event.currentTarget.value); setPosition(previous => ({ ...previous, row: 0, expanded: false, game: 0 })); }}
            onKeyDown={event => { event.stopPropagation(); if (event.key === 'Escape') { event.currentTarget.blur(); setQuery(''); } else if (event.key === 'Enter') { event.currentTarget.blur(); navigate('enter'); } }} /></label>
          <time>{clock()}</time>
        </div>
      </header>
      <nav class="xmb-category-rail" aria-label="Categorias" style={{ '--category-index': position().category }}>
        <div class="xmb-category-track">
          <For each={categories()}>{(item, index) => (
            <button class="xmb-category" classList={{ selected: index() === position().category }} title={item.title} aria-label={item.title}
              tabIndex={index() === position().category ? 0 : -1}
              aria-current={index() === position().category ? 'page' : undefined} onClick={() => chooseCategory(index())}>
              <Show when={item.kind === 'platform'} fallback={item.kind === 'settings' ? <Settings2 /> : item.kind === 'favorites' ? <Heart /> : item.kind === 'installed' ? <Download /> : <Layers />}>
                <Show when={item.id !== 'pc' && item.id !== 'linux'} fallback={<Monitor />}><ConsoleHardwareVisual platformId={item.id} size="sm" /></Show>
              </Show>
              <span>{item.title}</span>
            </button>
          )}</For>
        </div>
      </nav>
      <Show when={position().row === 0}>
        <section class="xmb-category-intro"><span class="xmb-eyebrow">{category().kind === 'settings' ? 'Consola' : 'Biblioteca'}</span><h1>{category().title}</h1>
          <p>{category().kind === 'settings' ? `${rows()} secciones` : `${folders().length.toLocaleString('es-ES')} titulos · ${packages().toLocaleString('es-ES')} paquetes`}</p>
          <Show when={rows() === 0}><p role="status">{props.loading ? 'Cargando biblioteca...' : 'Sin juegos en esta categoria'}</p></Show>
        </section>
      </Show>
      <section class="xmb-folder-viewport" aria-label={category().kind === 'settings' ? 'Secciones de ajustes' : 'Titulos'}>
        <For each={rowWindow()}>{row => {
          const current = () => category().kind === 'settings' ? settingItems[row - 1] : folders()[row - 1];
          const active = () => position().row === row;
          return <div class="xmb-folder-row" classList={{ active: active(), expanded: active() && position().expanded }}
            style={{ '--row-offset': row - position().row }}>
            <button class="xmb-folder-button" aria-label={current()?.title} aria-expanded={active() && position().expanded}
              tabIndex={active() || (position().row === 0 && row === 1) ? 0 : -1}
              onClick={() => { if (active()) navigate('enter'); else { setPosition(previous => ({ ...previous, row, expanded: false, game: 0 })); props.onMove(); } }}>
              <ChevronUp class="xmb-above" size={12} style={{ visibility: row > 1 ? 'visible' : 'hidden' }} />
              {category().kind === 'settings' ? <SlidersHorizontal size={24} /> : <Folder size={26} fill="currentColor" strokeWidth={1.4} />}
              <ChevronDown class="xmb-below" size={12} style={{ visibility: row < rows() ? 'visible' : 'hidden' }} />
            </button>
            <Show when={active() && !position().expanded}>
              <div class="xmb-folder-label"><h2>{current()?.title}</h2><p>{category().kind === 'settings' ? settingItems[row - 1]?.detail : `${folders()[row - 1]?.games.length || 0} versiones de plataforma`}</p></div>
            </Show>
          </div>;
        }}</For>
      </section>
      <Show when={position().expanded && selectedGame()}>{game => (
        <>
          <section class="xmb-game-shelf" aria-label="Versiones de plataforma">
            <For each={gameWindow()}>{entry => <button id={`shelf-card-${entry.game.id}`} class="xmb-game-tile"
              tabIndex={entry.index === position().game ? 0 : -1}
              classList={{ selected: entry.index === position().game }} style={{ '--game-offset': entry.index - position().game }}
              title={entry.game.title} onClick={() => { if (entry.index === position().game) props.onOpenGame(entry.game); else setPosition(previous => ({ ...previous, game: entry.index })); }}>
              <Show when={entry.game.coverImage} fallback={<ConsoleHardwareVisual platformId={entry.game.platform} size="sm" />}>
                <img src={entry.game.coverImage} alt="" loading="lazy" onError={event => { event.currentTarget.style.visibility = 'hidden'; }} />
              </Show>
              <span><strong>{entry.game.title}</strong><small>{entry.game.platformName}</small><small>{entry.game.variants.length} paquetes</small></span>
            </button>}</For>
          </section>
          <aside class="xmb-detail-drawer" aria-label="Ficha del juego">
            <header><span class="xmb-eyebrow">{game().platformName}</span><button class="xmb-icon-button" title="Cerrar ficha" aria-label="Cerrar ficha" onClick={() => navigate('back')}><ArrowLeft size={18} /></button></header>
            <div class="xmb-detail-scroll" ref={detailPanel} tabIndex={0} onKeyDown={event => {
              if (['ArrowUp', 'ArrowDown', 'PageUp', 'PageDown', 'Home', 'End', ' '].includes(event.key)) event.stopPropagation();
            }}>
              <div class="xmb-boxart"><Show when={game().coverImage && !coverFailed()} fallback={<ConsoleHardwareVisual platformId={game().platform} size="lg" />}>
                <img src={game().coverImage} alt={game().title} onError={() => setCoverFailed(true)} />
              </Show></div>
              <div class="xmb-title-line"><h2>{game().title}</h2><button class="xmb-icon-button" title="Alternar favorito" aria-label="Alternar favorito" aria-pressed={game().favorite} onClick={() => props.onFavorite(game().id)}><Heart size={19} fill={game().favorite ? 'currentColor' : 'none'} /></button></div>
              <p class="xmb-description">{game().description || 'Descripcion no disponible.'}</p>
              <dl class="xmb-facts"><div><dt>Genero</dt><dd>{game().genre || 'No disponible'}</dd></div><div><dt>Lanzamiento</dt><dd>{game().releaseYear || 'No disponible'}</dd></div>
                <div><dt>Desarrollador</dt><dd>{game().developer || 'No disponible'}</dd></div><div><dt>Paquetes</dt><dd>{game().variants.length}</dd></div>
                <div><dt>Estado</dt><dd>{props.downloadingIds.has(game().id) ? 'Descargando' : game().installed ? 'Instalado' : 'No instalado'}</dd></div><div><dt>Tiempo jugado</dt><dd>{game().playTimeMinutes || 0} min</dd></div></dl>
            </div>
            <footer><button class="xmb-primary" onClick={() => props.onOpenGame(game())}><Download size={18} />Fuentes y paquetes</button></footer>
          </aside>
        </>
      )}</Show>
      <Show when={props.error}><div class="xmb-error" role="alert">{props.error?.message}</div></Show>
      <footer class="xmb-bottom"><div class="xmb-step-controls">
        <button title={position().expanded ? 'Version anterior' : 'Categoria anterior'} aria-label={position().expanded ? 'Version anterior' : 'Categoria anterior'} onClick={() => navigate('left')}><ChevronLeft size={19} /></button>
        <button title="Anterior" aria-label="Anterior" onClick={() => navigate('up')}><ChevronUp size={19} /></button>
        <span>{position().row} / {rows().toLocaleString('es-ES')}</span>
        <button title="Siguiente" aria-label="Siguiente" onClick={() => navigate('down')}><ChevronDown size={19} /></button>
        <button title={position().expanded ? 'Version siguiente' : 'Categoria siguiente'} aria-label={position().expanded ? 'Version siguiente' : 'Categoria siguiente'} onClick={() => navigate('right')}><ChevronRight size={19} /></button>
      </div><span class="xmb-input-status"><Gamepad2 size={16} />{props.inputStatus.deviceName}</span><button class="xmb-icon-button" title="Mantenimiento" aria-label="Mantenimiento" onClick={props.onMaintenance}><Wrench size={17} /></button></footer>
    </main>
  );
}
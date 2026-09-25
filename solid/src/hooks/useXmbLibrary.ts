import {
  createEffect,
  createMemo,
  createSignal,
  on,
  onCleanup,
  onMount,
} from "solid-js";
import type { InputAction } from "@contracts/input.types";
import type {
  VirtualKeyboardKey,
  XmbCommand,
  XmbLibraryProps,
  XmbPosition,
} from "@contracts/xmb.types";
import { moveXmb, xmbFolders } from "@services/library/xmb-navigation";

const commands: Partial<Record<InputAction, XmbCommand>> = {
  NAV_LEFT: "left",
  NAV_RIGHT: "right",
  NAV_UP: "up",
  NAV_DOWN: "down",
  BUTTON_A: "enter",
  BUTTON_B: "back",
};
const settingItems = [
  {
    id: "system",
    title: "Ajustes",
    detail: "Sistema, audio, controles y tiendas",
  },
];
const VIRTUAL_KEYBOARD_COLUMNS = 10;
const VIRTUAL_KEYBOARD_KEYS: readonly VirtualKeyboardKey[] = [
  ..."ABCDEFGHIJ".split("").map((value) => ({ label: value, value })),
  ..."KLMNOPQRST".split("").map((value) => ({ label: value, value })),
  ..."UVWXYZ".split("").map((value) => ({ label: value, value })),
  { label: "ESP", value: " " },
  { label: "BORRAR", value: "backspace" },
  { label: "LIMPIAR", value: "clear" },
  { label: "LISTO", value: "done" },
];

export function useXmbLibrary(props: XmbLibraryProps) {
  const [position, setPosition] = createSignal<XmbPosition>({
    category: 1,
    row: 0,
    expanded: false,
    game: 0,
  });
  const [query, setQuery] = createSignal("");
  const [clock, setClock] = createSignal("");
  const [coverFailed, setCoverFailed] = createSignal(false);
  const [virtualKeyboardOpen, setVirtualKeyboardOpen] = createSignal(false);
  const [virtualKeyboardIndex, setVirtualKeyboardIndex] = createSignal(0);
  let searchInput: HTMLInputElement | undefined;
  let detailPanel: HTMLDivElement | undefined;
  let wheelSurface: "categories" | "folders" | "versions" | undefined;
  let wheelDistance = 0;

  const categories = createMemo(() => {
    const platforms = new Map(
      props.platforms
        .filter((platform) => platform.id !== "all")
        .map((platform) => [platform.id, platform.name]),
    );
    for (const game of props.games) {
      if (!platforms.has(game.platform))
        platforms.set(game.platform, game.platformName || game.platform);
    }
    return [
      { id: "settings", title: "Ajustes", kind: "settings" },
      { id: "all", title: "Todos los juegos", kind: "all" },
      { id: "favorites", title: "Favoritos", kind: "favorites" },
      { id: "installed", title: "Instalados", kind: "installed" },
      ...[...platforms]
        .sort(([, left], [, right]) =>
          left.localeCompare(right, "es", { sensitivity: "base" }),
        )
        .map(([id, title]) => ({ id, title, kind: "platform" })),
    ];
  });
  const categoryIndex = createMemo(() => position().category);
  const category = createMemo(
    () => categories()[categoryIndex()] || categories()[1],
  );
  const filtered = createMemo(() => {
    const current = category();
    const search = query().trim().toLocaleLowerCase();
    if (current.kind === "settings") return [];
    return props.games.filter(
      (game) =>
        (current.kind !== "platform" || game.platform === current.id) &&
        (current.kind !== "favorites" || game.favorite) &&
        (current.kind !== "installed" || game.installed) &&
        (!search || game.title.toLocaleLowerCase().includes(search)),
    );
  });
  const folders = createMemo(() => xmbFolders(filtered()));
  const rows = () =>
    category().kind === "settings" ? settingItems.length : folders().length;
  const folder = () => folders()[position().row - 1];
  const selectedGame = () => folder()?.games[position().game];
  const rowWindow = createMemo(() => {
    const start = Math.max(1, position().row - 6);
    const end = Math.min(rows(), Math.max(8, position().row + 6));
    return Array.from(
      { length: Math.max(0, end - start + 1) },
      (_, index) => start + index,
    );
  });
  const gameWindow = createMemo(() => {
    const start = Math.max(0, position().game - 1);
    return (folder()?.games || [])
      .slice(start, position().game + 4)
      .map((game, offset) => ({ game, index: start + offset }));
  });
  const packages = createMemo(() =>
    filtered().reduce((total, game) => total + game.variants.length, 0),
  );

  createEffect(() => {
    const maxRows = rows();
    const current = position();
    if (current.row > maxRows)
      setPosition({ ...current, row: maxRows, expanded: false, game: 0 });
    else if (current.expanded && !selectedGame())
      setPosition({ ...current, expanded: false, game: 0 });
  });
  createEffect(
    on(
      () => selectedGame()?.id,
      () => {
        setCoverFailed(false);
        if (detailPanel) detailPanel.scrollTop = 0;
      },
    ),
  );

  const navigate = (command: XmbCommand) => {
    if (
      (command === "enter" || command === "down") &&
      category().kind === "settings"
    ) {
      props.onOpenSettings("system");
      return;
    }
    if (command === "enter" && position().expanded && selectedGame()) {
      props.onOpenGame(selectedGame()!);
      return;
    }
    setPosition((previous) =>
      moveXmb(previous, command, {
        categories: categories().length,
        rows: rows(),
        games: folder()?.games.length || 0,
      }),
    );
    props.onMove();
  };
  const chooseCategory = (index: number) => {
    setPosition({ category: index, row: 0, expanded: false, game: 0 });
    if (index === 0) props.onOpenSettings("system");
    else props.onCloseSettings?.();
    props.onMove();
  };
  const chooseRow = (row: number) => {
    if (position().row === row) navigate("enter");
    else {
      setPosition((previous) => ({
        ...previous,
        row,
        expanded: false,
        game: 0,
      }));
      props.onMove();
    }
  };
  const chooseGame = (index: number) => {
    const game = folder()?.games[index];
    if (!game) return;
    if (index === position().game) props.onOpenGame(game);
    else setPosition((previous) => ({ ...previous, game: index }));
  };
  const scrollWheel = (surface: "categories" | "folders" | "versions", delta: number, mode: number) => {
    if (!Number.isFinite(delta) || delta === 0 || virtualKeyboardOpen()) return false;
    const distance = delta * (mode === 1 ? 16 : mode === 2 ? 300 : 1);
    if (wheelSurface !== surface || Math.sign(wheelDistance) !== Math.sign(distance)) wheelDistance = 0;
    wheelSurface = surface;
    wheelDistance += distance;
    if (Math.abs(wheelDistance) < 50) return true;
    const forward = wheelDistance > 0;
    wheelDistance = 0;
    if (surface === "categories") {
      const next = Math.max(0, Math.min(categories().length - 1, position().category + (forward ? 1 : -1)));
      if (next !== position().category) chooseCategory(next);
    } else if (surface === "versions") {
      if (position().expanded) navigate(forward ? "right" : "left");
    } else {
      navigate(forward ? "down" : "up");
    }
    return true;
  };
  const search = (value: string) => {
    setQuery(value);
    setPosition((previous) => ({
      ...previous,
      row: 0,
      expanded: false,
      game: 0,
    }));
  };
  const keyboardAvailable = () => props.inputStatus.source === "keyboard";
  const openSearch = () => {
    if (keyboardAvailable()) searchInput?.focus();
    else {
      setVirtualKeyboardIndex(0);
      setVirtualKeyboardOpen(true);
    }
  };
  const closeVirtualKeyboard = () => setVirtualKeyboardOpen(false);
  const applyVirtualKey = (key: VirtualKeyboardKey) => {
    switch (key.value) {
      case "backspace":
        search(query().slice(0, -1));
        break;
      case "clear":
        search("");
        break;
      case "done":
        closeVirtualKeyboard();
        break;
      default:
        search(`${query()}${key.value}`);
        break;
    }
  };
  const navigateVirtualKeyboard = (action: InputAction) => {
    switch (action) {
      case "NAV_LEFT":
        setVirtualKeyboardIndex((index) =>
          Math.max(0, index - 1),
        );
        break;
      case "NAV_RIGHT":
        setVirtualKeyboardIndex((index) =>
          Math.min(VIRTUAL_KEYBOARD_KEYS.length - 1, index + 1),
        );
        break;
      case "NAV_UP":
        setVirtualKeyboardIndex((index) =>
          Math.max(0, index - VIRTUAL_KEYBOARD_COLUMNS),
        );
        break;
      case "NAV_DOWN":
        setVirtualKeyboardIndex((index) =>
          Math.min(VIRTUAL_KEYBOARD_KEYS.length - 1, index + VIRTUAL_KEYBOARD_COLUMNS),
        );
        break;
      case "BUTTON_A":
        applyVirtualKey(VIRTUAL_KEYBOARD_KEYS[virtualKeyboardIndex()]);
        break;
      case "BUTTON_B":
        closeVirtualKeyboard();
        break;
      case "BUTTON_X":
        applyVirtualKey({ label: "BORRAR", value: "backspace" });
        break;
      case "BUTTON_Y":
        applyVirtualKey({ label: "ESP", value: " " });
        break;
      default:
        break;
    }
  };
  const controller = (action: InputAction) => {
    if (virtualKeyboardOpen()) {
      navigateVirtualKeyboard(action);
      return;
    }
    const command = commands[action];
    if (command) {
      navigate(command);
      return;
    }
    switch (action) {
      case "BUTTON_LB":
      case "BUTTON_RB":
        chooseCategory(
          Math.max(
            0,
            Math.min(
              categories().length - 1,
              position().category + (action === "BUTTON_RB" ? 1 : -1),
            ),
          ),
        );
        break;
      case "BUTTON_X":
        if (selectedGame()) props.onFavorite(selectedGame()!.id);
        break;
      case "BUTTON_LT":
      case "BUTTON_RT":
        if (position().expanded)
          detailPanel?.scrollBy({ top: action === "BUTTON_RT" ? 160 : -160 });
        break;
      case "BUTTON_Y":
        openSearch();
        break;
      case "BUTTON_START":
      case "HOME":
        chooseCategory(0);
        break;
      default:
        break;
    }
  };
  const onSearchKeyDown = (event: KeyboardEvent) => {
    event.stopPropagation();
    if (event.key === "Escape") {
      searchInput?.blur();
      search("");
    } else if (event.key === "Enter") {
      searchInput?.blur();
      navigate("enter");
    }
  };
  onMount(() => {
    props.onControllerReady(controller);
    const updateClock = () =>
      setClock(
        new Date().toLocaleTimeString("es-ES", {
          hour: "2-digit",
          minute: "2-digit",
        }),
      );
    updateClock();
    const timer = setInterval(updateClock, 10000);
    onCleanup(() => {
      clearInterval(timer);
      props.onControllerReady(null);
    });
  });

  return {
    position,
    query,
    clock,
    coverFailed,
    virtualKeyboardOpen,
    virtualKeyboardIndex,
    virtualKeyboardKeys: VIRTUAL_KEYBOARD_KEYS,
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
    scrollWheel,
    search,
    openSearch,
    closeVirtualKeyboard,
    applyVirtualKey,
    onSearchKeyDown,
    bindSearchInput: (element: HTMLInputElement) => {
      searchInput = element;
    },
    bindDetailPanel: (element: HTMLDivElement) => {
      detailPanel = element;
    },
    onCoverError: () => setCoverFailed(true),
  };
}

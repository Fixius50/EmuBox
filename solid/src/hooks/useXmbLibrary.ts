import {
  createEffect,
  createMemo,
  createSignal,
  on,
  onCleanup,
  onMount,
} from "solid-js";
import { SETTINGS_TABS } from "@contracts/settings.types";
import type { InputAction } from "@contracts/input.types";
import type {
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
  ...SETTINGS_TABS.map((tab) => ({
    id: tab.id,
    title: tab.name,
    detail: tab.desc,
  })),
  {
    id: "maintenance",
    title: "Mantenimiento",
    detail: "Sistema y recuperacion",
  },
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
  let searchInput: HTMLInputElement | undefined;
  let detailPanel: HTMLDivElement | undefined;

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
      ...[...platforms].map(([id, title]) => ({ id, title, kind: "platform" })),
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
    const start = Math.max(1, position().row - 3);
    const end = Math.min(rows(), Math.max(6, position().row + 3));
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
      command === "enter" &&
      category().kind === "settings" &&
      position().row > 0
    ) {
      const item = settingItems[position().row - 1];
      if (item.id === "maintenance") props.onMaintenance();
      else props.onOpenSettings(item.id);
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
  const search = (value: string) => {
    setQuery(value);
    setPosition((previous) => ({
      ...previous,
      row: 0,
      expanded: false,
      game: 0,
    }));
  };
  const controller = (action: InputAction) => {
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
        searchInput?.focus();
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
    bindSearchInput: (element: HTMLInputElement) => {
      searchInput = element;
    },
    bindDetailPanel: (element: HTMLDivElement) => {
      detailPanel = element;
    },
    onCoverError: () => setCoverFailed(true),
  };
}

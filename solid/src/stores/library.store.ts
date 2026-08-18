import { createSignal, createMemo } from 'solid-js';
import type { Game, PlatformId } from '@contracts/game.types';
import type { IEmuBoxBackend } from '@contracts/backend.types';

export function createLibraryStore(backend: IEmuBoxBackend) {
  const [games, setGames] = createSignal<Game[]>([]);
  const [selectedPlatform, setSelectedPlatform] = createSignal<PlatformId>('all');
  const [searchQuery, setSearchQuery] = createSignal<string>('');
  const [favoritesOnly, setFavoritesOnly] = createSignal<boolean>(false);
  const [datasetLimit, setDatasetLimit] = createSignal<number>(10000);
  const [isLoading, setIsLoading] = createSignal<boolean>(false);

  const loadGames = async (preloadedGames?: Game[]) => {
    setIsLoading(true);
    try {
      if (preloadedGames && preloadedGames.length > 0) {
        setGames(preloadedGames);
      } else {
        const fetched = await backend.getGames();
        setGames(fetched);
      }
    } finally {
      setIsLoading(false);
    }
  };

  const filteredGames = createMemo(() => {
    let list = games();
    const plat = selectedPlatform();
    const query = searchQuery().toLowerCase().trim();
    const favOnly = favoritesOnly();
    const limit = datasetLimit();

    if (plat && plat !== 'all') {
      list = list.filter(g => g.platform === plat);
    }

    if (favOnly) {
      list = list.filter(g => g.favorite);
    }

    if (query) {
      list = list.filter(g =>
        g.title.toLowerCase().includes(query) ||
        g.genre.toLowerCase().includes(query) ||
        g.developer.toLowerCase().includes(query)
      );
    }

    if (limit > 0 && list.length > limit) {
      list = list.slice(0, limit);
    }

    return list;
  });

  const toggleFavorite = async (gameId: string) => {
    const newStatus = await backend.toggleFavorite(gameId);
    setGames(prev =>
      prev.map(g => (g.id === gameId ? { ...g, favorite: newStatus } : g))
    );
  };

  return {
    games,
    setGames,
    selectedPlatform,
    setSelectedPlatform,
    searchQuery,
    setSearchQuery,
    favoritesOnly,
    setFavoritesOnly,
    datasetLimit,
    setDatasetLimit,
    isLoading,
    loadGames,
    filteredGames,
    toggleFavorite
  };
}

export type LibraryStore = ReturnType<typeof createLibraryStore>;

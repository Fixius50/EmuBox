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
  const [downloadingIds, setDownloadingIds] = createSignal<Set<string>>(new Set());

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

  const downloadGame = async (gameId: string) => {
    setDownloadingIds(prev => new Set(prev).add(gameId));
    const clear = () => setDownloadingIds(prev => {
      const next = new Set(prev);
      next.delete(gameId);
      return next;
    });
    try {
      const job = await backend.downloadGame(gameId);
      const poll = async () => {
        const jobs = await backend.getDownloadJobs();
        const current = jobs.find(j => j.id === job.id);
        if (!current || current.status === 'completed' || current.status === 'failed' || current.status === 'cancelled') {
          clear();
          return;
        }
        setTimeout(poll, 2000);
      };
      setTimeout(poll, 2000);
    } catch {
      clear();
    }
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
    toggleFavorite,
    downloadingIds,
    downloadGame
  };
}

export type LibraryStore = ReturnType<typeof createLibraryStore>;

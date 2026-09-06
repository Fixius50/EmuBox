import { createSignal, createMemo } from 'solid-js';
import type { Game, PlatformId } from '@contracts/game.types';
import type { IEmuBoxBackend } from '@contracts/backend.types';
import type { DownloadJob, DownloadSourceOption } from '@contracts/download.types';
import { groupCatalog } from '@services/library/catalog-groups';

export function createLibraryStore(backend: IEmuBoxBackend) {
  const [games, setGames] = createSignal<Game[]>([]);
  const [selectedPlatform, setSelectedPlatform] = createSignal<PlatformId>('all');
  const [searchQuery, setSearchQuery] = createSignal<string>('');
  const [favoritesOnly, setFavoritesOnly] = createSignal<boolean>(false);
  const [datasetLimit, setDatasetLimit] = createSignal<number>(0);
  const [isLoading, setIsLoading] = createSignal<boolean>(false);
  const [downloadingIds, setDownloadingIds] = createSignal<Set<string>>(new Set());
  const [downloadError, setDownloadError] = createSignal<{ gameId: string; message: string } | null>(null);
  const [sourceGame, setSourceGame] = createSignal<Game | null>(null);
  const [sourceOptions, setSourceOptions] = createSignal<DownloadSourceOption[]>([]);
  const [sourcesLoading, setSourcesLoading] = createSignal(false);
  const [sourcesError, setSourcesError] = createSignal('');
  const [sourceIndex, setSourceIndex] = createSignal(0);
  const catalogGames = createMemo(() => groupCatalog(games()));
  const groupByVariant = createMemo(() => new Map(catalogGames().flatMap(group => group.variants.map(variant => [variant.id, group] as const))));
  const catalogDownloadingIds = createMemo(() => new Set([...downloadingIds()].map(id => groupByVariant().get(id)?.id || id)));
  let sourceRequest = 0;
  const closeSources = () => {
    sourceRequest++;
    setSourceGame(null);
    setSourcesLoading(false);
  };
  const openSources = async (game: Game) => {
    const request = ++sourceRequest;
    setSourceGame(game);
    setSourceOptions([]);
    setSourcesError('');
    setSourceIndex(0);
    setSourcesLoading(true);
    try {
      const variants = groupByVariant().get(game.id)?.variants || [game];
      const sources: DownloadSourceOption[] = [];
      for (let offset = 0; offset < variants.length && request === sourceRequest; offset += 4) {
        const batch = await Promise.all(variants.slice(offset, offset + 4).map(async variant =>
          (await backend.getDownloadSources(variant.id)).map(source => ({ ...source, name: variant.title }))));
        sources.push(...batch.flat());
      }
      if (request === sourceRequest) setSourceOptions(sources);
    } catch (error) {
      if (request === sourceRequest) setSourcesError(error instanceof Error ? error.message : 'No se pudieron consultar las fuentes');
    } finally {
      if (request === sourceRequest) setSourcesLoading(false);
    }
  };

  const loadGames = async (preloadedGames?: Game[]) => {
    setIsLoading(true);
    try {
      const fetched = preloadedGames ?? await backend.getGames();
      const merged = new Map(fetched.map(game => [game.id, game]));
      setGames([...merged.values()]);
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
    const group = groupByVariant().get(gameId);
    if (group?.favorite) {
      for (const variant of group.variants.filter(entry => entry.favorite)) {
        const favorite = await backend.toggleFavorite(variant.id);
        setGames(previous => previous.map(game => game.id === variant.id ? { ...game, favorite } : game));
      }
      return;
    }
    const newStatus = await backend.toggleFavorite(gameId);
    setGames(prev =>
      prev.map(g => (g.id === gameId ? { ...g, favorite: newStatus } : g))
    );
  };

  const downloadGame = async (gameId: string, sourceId?: string) => {
    if (downloadingIds().has(gameId)) return;
    setDownloadError(null);
    setDownloadingIds(prev => new Set(prev).add(gameId));
    const clear = () => setDownloadingIds(prev => {
      const next = new Set(prev);
      next.delete(gameId);
      return next;
    });
    const fail = (error: unknown) => {
      const message = typeof error === 'string' ? error
        : error && typeof error === 'object' && 'message' in error && typeof error.message === 'string'
          ? error.message
          : error && typeof error === 'object' && 'details' in error && typeof error.details === 'string'
            ? error.details : 'No se pudo descargar el juego. Comprueba la fuente y la conexion.';
      setDownloadError({ gameId, message });
      clear();
    };
    const finish = async (job?: DownloadJob) => {
      if (!job) {
        fail('La descarga ya no aparece en el servidor. Vuelve a intentarlo.');
        return true;
      }
      if (job.status === 'failed' || job.status === 'cancelled') {
        fail(job.error || (job.status === 'failed' ? 'La descarga ha fallado.' : 'Descarga cancelada.'));
        return true;
      }
      if (job.status === 'completed') {
        await loadGames();
        clear();
        return true;
      }
      return false;
    };
    try {
      const job = await backend.downloadGame(gameId, sourceId);
      if (await finish(job)) return;
      const poll = async () => {
        try {
          const jobs = await backend.getDownloadJobs();
          if (!await finish(jobs.find(current => current.id === job.id))) setTimeout(poll, 2000);
        } catch (error) {
          fail(error);
        }
      };
      setTimeout(poll, 2000);
    } catch (error) {
      fail(error);
    }
  };

  return {
    catalogGames, catalogDownloadingIds,
    confirmSource: async () => {
      const source = sourceOptions()[sourceIndex()];
      if (!sourceGame() || sourcesLoading() || !source?.downloadable) return;
      closeSources();
      await downloadGame(source.gameId, source.id);
    },
    sourceGame, sourceOptions, sourcesLoading, sourcesError, sourceIndex, setSourceIndex, openSources, closeSources,
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
    downloadError,
    downloadGame
  };
}

export type LibraryStore = ReturnType<typeof createLibraryStore>;

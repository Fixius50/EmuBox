import { createSignal, createMemo } from 'solid-js';
import type { Game, GameReleaseOption, PlatformId } from '@contracts/game.types';
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
  const [allSourceOptions, setAllSourceOptions] = createSignal<DownloadSourceOption[]>([]);
  const [releaseOptions, setReleaseOptions] = createSignal<GameReleaseOption[]>([]);
  const [releaseIndex, setReleaseIndex] = createSignal(0);
  const sourceOptions = createMemo(() => {
    const release = releaseOptions()[releaseIndex()];
    return release ? allSourceOptions().filter(source => source.gameId === release.catalogGameId) : allSourceOptions();
  });
  const selectRelease = (index: number) => {
    setReleaseIndex(Math.max(0, Math.min(releaseOptions().length - 1, index)));
    setSourceIndex(0);
  };
  const [sourcesLoading, setSourcesLoading] = createSignal(false);
  const [sourcesError, setSourcesError] = createSignal('');
  const [sourceIndex, setSourceIndex] = createSignal(0);
  const [downloadJobs, setDownloadJobs] = createSignal<DownloadJob[]>([]);
  const refreshJobs = async () => {
    const jobs = await backend.getDownloadJobs();
    setDownloadJobs(jobs);
    return jobs;
  };
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
    setAllSourceOptions([]);
    setReleaseOptions([]);
    setSourcesError('');
    setSourceIndex(0);
    setReleaseIndex(0);
    setSourcesLoading(true);
    try {
      await refreshJobs();
      const group = groupByVariant().get(game.id);
      const canonicalId = game.canonicalId || group?.canonicalId || group?.variants.find(variant => variant.canonicalId)?.canonicalId;
      if (canonicalId) {
        const options = await backend.getCanonicalGameOptions(canonicalId);
        if (request === sourceRequest) {
          setReleaseOptions(options.releases);
          setAllSourceOptions(options.sources.map(source => ({ ...source,
            name: options.releases.find(release => release.catalogGameId === source.gameId)?.title || source.name })));
        }
      } else {
        const variants = groupByVariant().get(game.id)?.variants || [game];
        const sources: DownloadSourceOption[] = [];
        for (let offset = 0; offset < variants.length && request === sourceRequest; offset += 4) {
          const batch = await Promise.all(variants.slice(offset, offset + 4).map(async variant =>
            (await backend.getDownloadSources(variant.id)).map(source => ({ ...source, name: variant.title }))));
          sources.push(...batch.flat());
        }
        if (request === sourceRequest) {
          setReleaseOptions(variants.map(variant => ({ id: variant.releaseId || variant.id,
            catalogGameId: variant.id, title: variant.releaseTitle || variant.title, installed: variant.installed,
            sourceCount: sources.filter(source => source.gameId === variant.id).length,
            downloadableSourceCount: sources.filter(source => source.gameId === variant.id && source.downloadable).length })));
          setAllSourceOptions(sources);
        }
      }
    } catch (error) {
      if (request === sourceRequest) setSourcesError(error instanceof Error ? error.message : 'No se pudieron consultar las fuentes');
    } finally {
      if (request === sourceRequest) setSourcesLoading(false);
    }
  };

  let gamesRequest = 0;
  let pendingGames: Promise<void> | undefined;
  const [loadError, setLoadError] = createSignal('');
  const loadGames = (preloadedGames?: Game[]): Promise<void> => {
    if (preloadedGames === undefined && pendingGames) return pendingGames;
    const request = ++gamesRequest;
    setIsLoading(true);
    setLoadError('');
    const operation = Promise.resolve().then(() => preloadedGames ?? backend.getGames())
      .then(fetched => {
        if (request !== gamesRequest) return;
        const previous = games();
        const byId = new Map(previous.map(game => [game.id, game]));
        const merged = new Map<string, Game>();
        for (const game of fetched) {
          const existing = byId.get(game.id);
          const keys = Object.keys(game) as (keyof Game)[];
          const unchanged = existing && Object.keys(existing).length === keys.length
            && keys.every(key => Object.is(existing[key], game[key]));
          merged.set(game.id, unchanged ? existing : game);
        }
        const next = [...merged.values()];
        if (next.length !== previous.length || next.some((game, index) => game !== previous[index])) {
          setGames(next);
        }
      }).catch(error => {
        if (request === gamesRequest) setLoadError(error instanceof Error ? error.message : 'No se pudo actualizar la biblioteca');
        throw error;
      }).finally(() => {
        if (request === gamesRequest) {
          pendingGames = undefined;
          setIsLoading(false);
        }
      });
    pendingGames = operation;
    return operation;
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
    const newStatus = await backend.toggleFavorite(gameId);
    const variantIds = new Set(group?.variants.map(variant => variant.id) || [gameId]);
    setGames(previous => previous.map(game => variantIds.has(game.id) ? { ...game, favorite: newStatus } : game));
  };

  const downloadGame = async (gameId: string, sourceId?: string, resumeId?: string) => {
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
      if (job.status === 'completed' || job.status === 'downloaded') {
        await loadGames();
        await refreshJobs();
        if (job.status === 'downloaded') setDownloadError({ gameId, message: job.error || 'Contenido descargado; requiere preparacion.' });
        clear();
        return true;
      }
      if (job.status === 'paused') { clear(); return true; }
      return false;
    };
    try {
      const job = resumeId ? await backend.resumeDownload(resumeId) : await backend.downloadGame(gameId, sourceId);
      await refreshJobs();
      if (await finish(job)) return;
      const poll = async () => {
        try {
          const jobs = await refreshJobs();
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
    downloadJobs, refreshJobs,
    getDownloadCandidates: (id: string) => backend.getDownloadCandidates(id),
    selectDownloadCandidate: async (id: string, path: string) => {
      await backend.selectDownloadCandidate(id, path);
      await refreshJobs();
      await loadGames();
    },
    controlDownload: async (job: DownloadJob, action: 'pause' | 'resume' | 'cancel') => {
      try {
        if (action === 'pause') await backend.pauseDownload(job.id);
        else if (action === 'cancel') await backend.cancelDownload(job.id);
        else {
          setDownloadingIds(previous => { const next = new Set(previous); next.delete(job.gameId); return next; });
          await downloadGame(job.gameId, job.sourceId, job.id);
        }
        await refreshJobs();
      } catch (error) { setDownloadError({ gameId: job.gameId, message: error instanceof Error ? error.message : 'No se pudo cambiar el estado de la descarga' }); }
    },
    catalogGames, catalogDownloadingIds,
    confirmSource: async () => {
      const source = sourceOptions()[sourceIndex()];
      if (!sourceGame() || sourcesLoading() || !source?.downloadable) return;
      closeSources();
      await downloadGame(source.gameId, source.id);
    },
    sourceGame, sourceOptions, releaseOptions, releaseIndex, selectRelease,
    sourcesLoading, sourcesError, sourceIndex, setSourceIndex, openSources, closeSources,
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
    loadError,
    loadGames,
    filteredGames,
    toggleFavorite,
    downloadingIds,
    downloadError,
    downloadGame
  };
}

export type LibraryStore = ReturnType<typeof createLibraryStore>;

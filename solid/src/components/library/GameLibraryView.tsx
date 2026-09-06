import { Component, For, Show, createMemo } from 'solid-js';
import type { Game, Platform, Emulator } from '@contracts/game.types';
import type { CatalogGroup } from '@services/library/catalog-groups';
import { ConsoleHardwareVisual } from '@components/common/ConsoleHardwareVisual';
import { ConsoleShelfGrid } from './ConsoleShelfGrid';

interface GameLibraryViewProps {
  games: CatalogGroup[];
  platforms: Platform[];
  emulators: Emulator[];
  selectedPlatform: string;
  focusedIndex: number;
  downloadingIds: Set<string>;
  downloadError?: { gameId: string; message: string } | null;
  onSelectPlatform: (platformId: string) => void;
  onFocusIndex: (index: number) => void;
  onSelectGame: (game: Game) => void;
  onDownloadGame: (game: Game) => void;
  onToggleFavorite: (gameId: string) => void;
}

export const GameLibraryView: Component<GameLibraryViewProps> = (props) => {
  const counts = createMemo(() => {
    const result = new Map<string, number>();
    for (const game of props.games) result.set(game.platform, (result.get(game.platform) ?? 0) + 1);
    return result;
  });
  const availablePlatforms = createMemo(() => {
    const result = new Map(props.platforms.filter(platform => platform.id !== 'all')
      .map(platform => [platform.id, { id: platform.id, name: platform.name, shortName: platform.shortName }]));
    for (const game of props.games) {
      if (!result.has(game.platform)) result.set(game.platform, {
        id: game.platform, name: game.platformName || game.platform, shortName: game.platform.toUpperCase(),
      });
    }
    return [...result.values()];
  });
  const visibleGames = createMemo(() => props.selectedPlatform === 'all'
    ? props.games
    : props.games.filter((game) => game.platform === props.selectedPlatform));
  const selectedName = () => props.selectedPlatform === 'all'
    ? 'Todos los juegos'
    : availablePlatforms().find((platform) => platform.id === props.selectedPlatform)?.name || 'Juegos';

  return (
    <main class="game-library-layout">
      <aside class="game-library-sidebar" aria-label="Filtrar juegos por consola">
        <div class="library-sidebar-heading">BIBLIOTECA</div>
        <button
          class={`library-filter-item ${props.selectedPlatform === 'all' ? 'active' : ''}`}
          onClick={() => props.onSelectPlatform('all')}
        >
          <span>Todos</span>
          <span>{props.games.length}</span>
        </button>
        <div class="library-sidebar-divider" />
        <For each={availablePlatforms()}>
          {(platform) => {
            const count = () => counts().get(platform.id) ?? 0;
            return (
              <button
                class={`library-filter-item ${props.selectedPlatform === platform.id ? 'active' : ''}`}
                onClick={() => props.onSelectPlatform(platform.id)}
              >
                <span class="library-filter-platform">
                  <span aria-hidden="true"><ConsoleHardwareVisual platformId={platform.id} size="sm" /></span>
                  <span>{platform.shortName}</span>
                </span>
                <span>{count()}</span>
              </button>
            );
          }}
        </For>
      </aside>

      <section class="game-library-content" aria-label={selectedName()}>
        <Show when={props.downloadError}>
          {(error) => (
            <p role="alert" class="library-download-error">
              {props.games.find(game => game.id === error().gameId)?.title || 'Descarga'}: {error().message}
            </p>
          )}
        </Show>
        <Show when={visibleGames().length === 0}>
          <p class="library-empty-state" role="status">No hay juegos en esta biblioteca.</p>
        </Show>
        <ConsoleShelfGrid
          games={visibleGames()}
          focusedIndex={props.focusedIndex}
          downloadingIds={props.downloadingIds}
          onSelectGame={props.onSelectGame}
          onDownloadGame={props.onDownloadGame}
          onFocusIndex={props.onFocusIndex}
          onToggleFavorite={props.onToggleFavorite}
        />
      </section>
    </main>
  );
};
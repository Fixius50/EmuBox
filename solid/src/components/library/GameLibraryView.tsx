import { Component, For } from 'solid-js';
import type { Game, Platform } from '@contracts/game.types';
import { HeroSection } from './HeroSection';
import { ConsoleShelfGrid } from './ConsoleShelfGrid';

interface GameLibraryViewProps {
  games: Game[];
  platforms: Platform[];
  selectedPlatform: string;
  focusedIndex: number;
  onSelectPlatform: (platformId: string) => void;
  onFocusIndex: (index: number) => void;
  onSelectGame: (game: Game) => void;
  onToggleFavorite: (gameId: string) => void;
}

export const GameLibraryView: Component<GameLibraryViewProps> = (props) => {
  const visibleGames = () => props.selectedPlatform === 'all'
    ? props.games
    : props.games.filter((game) => game.platform === props.selectedPlatform);
  const selectedName = () => props.selectedPlatform === 'all'
    ? 'Todos los juegos'
    : props.platforms.find((platform) => platform.id === props.selectedPlatform)?.name || 'Juegos';
  const focusedGame = () => visibleGames()[props.focusedIndex] || visibleGames()[0] || null;

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
        <For each={props.platforms}>
          {(platform) => {
            const count = () => props.games.filter((game) => game.platform === platform.id).length;
            return (
              <button
                class={`library-filter-item ${props.selectedPlatform === platform.id ? 'active' : ''}`}
                onClick={() => props.onSelectPlatform(platform.id)}
              >
                <span>{platform.shortName}</span>
                <span>{count()}</span>
              </button>
            );
          }}
        </For>
      </aside>

      <section class="game-library-content">
        <div class="game-library-heading">
          <div>
            <span class="game-library-kicker">JUEGOS</span>
            <h1>{selectedName()}</h1>
          </div>
          <span class="game-library-count">{visibleGames().length} títulos</span>
        </div>
        <HeroSection
          focusedGame={focusedGame()}
          onPlayGame={props.onSelectGame}
          onOpenDetails={props.onSelectGame}
          onToggleFavorite={props.onToggleFavorite}
        />
        <ConsoleShelfGrid
          games={visibleGames()}
          focusedIndex={props.focusedIndex}
          onSelectGame={props.onSelectGame}
          onFocusIndex={props.onFocusIndex}
          onToggleFavorite={props.onToggleFavorite}
        />
      </section>
    </main>
  );
};
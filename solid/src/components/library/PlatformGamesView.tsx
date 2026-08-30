import { Component, onMount } from 'solid-js';
import { HeroSection } from './HeroSection';
import { ConsoleShelfGrid } from './ConsoleShelfGrid';
import { animateScreenEnter, animateGamesToWheel } from '@animations/screen-transitions';
import type { Game, Platform } from '@contracts/game.types';

interface PlatformGamesViewProps {
  platform: Platform;
  games: Game[];
  focusedIndex: number;
  onFocusIndex: (idx: number) => void;
  onSelectGame: (game: Game) => void;
  onBackToPlatforms: () => void;
  onToggleFavorite: (gameId: string) => void;
}

export const PlatformGamesView: Component<PlatformGamesViewProps> = (props) => {
  let viewContainerRef!: HTMLDivElement;

  const focusedGame = () => {
    const list = props.games;
    const idx = props.focusedIndex;
    if (list.length === 0) return null;
    if (idx >= list.length) return list[list.length - 1];
    return list[idx] || list[0] || null;
  };

  onMount(() => {
    if (viewContainerRef) {
      animateScreenEnter(viewContainerRef);
    }
  });

  const handleBack = () => {
    if (viewContainerRef) {
      animateGamesToWheel(viewContainerRef, () => {
        props.onBackToPlatforms();
      });
    } else {
      props.onBackToPlatforms();
    }
  };

  return (
    <div class="console-platform-games-view" ref={viewContainerRef}>
      <div class="system-view-subbar">
        <button
          class="back-to-wheel-btn"
          onClick={(e) => {
            e.stopPropagation();
            handleBack();
          }}
        >
          <span class="back-icon-arrow">←</span>
          <span>CONSOLAS</span>
        </button>

        <div class="subbar-platform-badge">
          <span class="subbar-plat-name">{props.platform.name.toUpperCase()}</span>
          <span class="subbar-plat-count">{props.games.length.toLocaleString()} JUEGOS</span>
        </div>
      </div>

      <HeroSection
        focusedGame={focusedGame()}
        onPlayGame={(game) => props.onSelectGame(game)}
        onOpenDetails={(game) => props.onSelectGame(game)}
        onToggleFavorite={(id) => props.onToggleFavorite(id)}
      />

      <ConsoleShelfGrid
        games={props.games}
        focusedIndex={props.focusedIndex}
        onSelectGame={props.onSelectGame}
        onFocusIndex={props.onFocusIndex}
        onToggleFavorite={props.onToggleFavorite}
      />
    </div>
  );
};

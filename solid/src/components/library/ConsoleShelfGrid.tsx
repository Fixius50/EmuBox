import { Component, For, createEffect, onMount, createSignal, Show } from 'solid-js';
import { createVirtualizer } from '@tanstack/solid-virtual';
import type { Game } from '@contracts/game.types';
import { ViewportService } from '@services/system/viewport.service';
import { ConsoleHardwareVisual } from '@components/common/ConsoleHardwareVisual';
import { animateStaggerShelf } from '@animations/shelf-animations';

interface ConsoleShelfGridProps {
  games: Game[];
  focusedIndex: number;
  onSelectGame: (game: Game) => void;
  onFocusIndex: (index: number) => void;
  onToggleFavorite: (id: string) => void;
}

export const ITEMS_PER_ROW = 5;
export const ROW_HEIGHT = 300;

export const ConsoleShelfGrid: Component<ConsoleShelfGridProps> = (props) => {
  let scrollContainerRef!: HTMLDivElement;
  const viewport = ViewportService.getInstance();

  const virtualizer = createVirtualizer({
    get count() {
      return Math.ceil((props.games?.length || 0) / ITEMS_PER_ROW);
    },
    getScrollElement: () => scrollContainerRef,
    estimateSize: () => ROW_HEIGHT,
    overscan: 4
  });

  onMount(() => {
    setTimeout(() => {
      animateStaggerShelf('.console-shelf-card');
    }, 40);
  });

  // Re-measure virtual rows dynamically when viewport dimensions change in hot runtime
  createEffect(() => {
    viewport.width();
    viewport.height();
    virtualizer.measure();
  });

  // Automatically scroll viewport and virtualizer whenever focusedIndex changes
  createEffect(() => {
    const idx = props.focusedIndex;
    if (idx >= 0 && props.games && props.games.length > 0) {
      const rowIndex = Math.floor(idx / ITEMS_PER_ROW);
      virtualizer.scrollToIndex(rowIndex, { align: 'auto' });

      setTimeout(() => {
        const targetCard = scrollContainerRef?.querySelector(`#shelf-card-${props.games[idx]?.id}`) as HTMLElement;
        if (targetCard) {
          targetCard.scrollIntoView({ behavior: 'smooth', block: 'nearest', inline: 'nearest' });
        }
      }, 20);
    }
  });

  return (
    <div class="console-shelf-viewport" ref={scrollContainerRef} id="shelf-viewport">
      <div
        style={{
          height: `${virtualizer.getTotalSize()}px`,
          width: '100%',
          position: 'relative'
        }}
      >
        <For each={virtualizer.getVirtualItems()}>
          {(virtualRow) => {
            const startIndex = virtualRow.index * ITEMS_PER_ROW;
            const rowGames = () => props.games.slice(startIndex, startIndex + ITEMS_PER_ROW);

            return (
              <div
                class="console-shelf-row"
                style={{
                  position: 'absolute',
                  top: 0,
                  left: 0,
                  width: '100%',
                  height: `${virtualRow.size}px`,
                  transform: `translateY(${virtualRow.start}px)`,
                  display: 'grid',
                  "grid-template-columns": `repeat(${ITEMS_PER_ROW}, 1fr)`,
                  gap: '0.875rem'
                }}
              >
                <For each={rowGames()}>
                  {(game, colIdx) => {
                    const globalIdx = () => startIndex + colIdx();
                    const isCardFocused = () => props.focusedIndex === globalIdx();
                    const [hasImageError, setHasImageError] = createSignal(false);

                    const hasValidCover = () => {
                      return Boolean(
                        game.coverImage &&
                        !game.coverImage.includes('placeholder') &&
                        !game.coverImage.includes('data:image/svg+xml;utf8,<svg') &&
                        !hasImageError()
                      );
                    };

                    return (
                      <div
                        class={`console-shelf-card ${isCardFocused() ? 'focused spatial-focus' : ''}`}
                        id={`shelf-card-${game.id}`}
                        onClick={() => {
                          props.onFocusIndex(globalIdx());
                          props.onSelectGame(game);
                        }}
                        onMouseEnter={() => props.onFocusIndex(globalIdx())}
                      >
                        <div class="case-spine-mark">{game.platform}</div>
                        <div class="card-cover-media">
                          <Show
                            when={hasValidCover()}
                            fallback={
                              <div class="card-default-cover-fallback">
                                <div class="fallback-backdrop-glow" />
                                <ConsoleHardwareVisual
                                  platformId={game.platform}
                                  size="sm"
                                  class="fallback-svg-icon"
                                />
                                <div class="fallback-title-overlay">{game.title}</div>
                              </div>
                            }
                          >
                            <img
                              src={game.coverImage}
                              alt={game.title}
                              class="card-img"
                              loading="lazy"
                              onError={() => setHasImageError(true)}
                            />
                          </Show>

                          <div class="card-case-highlight"></div>

                          {game.favorite && (
                            <div class="card-fav-badge" title="Favorito">
                              ★
                            </div>
                          )}
                        </div>

                        <div class="card-meta-dock">
                          <div class="card-title" title={game.title}>
                            {game.title}
                          </div>
                          <div class="card-subline">
                            <span>{game.releaseYear}</span>
                            <span class="card-rating">★ {game.rating.toFixed(1)}</span>
                          </div>
                        </div>
                      </div>
                    );
                  }}
                </For>
              </div>
            );
          }}
        </For>
      </div>
    </div>
  );
};

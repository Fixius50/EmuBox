import { Component, For, createEffect } from 'solid-js';
import { createVirtualizer } from '@tanstack/solid-virtual';
import type { Game } from '@contracts/game.types';

interface ConsoleShelfGridProps {
  games: Game[];
  focusedIndex: number;
  onSelectGame: (game: Game) => void;
  onFocusIndex: (index: number) => void;
  onToggleFavorite: (id: string) => void;
}

export const ITEMS_PER_ROW = 6;
export const ROW_HEIGHT = 205; // compact height in px

export const ConsoleShelfGrid: Component<ConsoleShelfGridProps> = (props) => {
  let scrollContainerRef!: HTMLDivElement;

  const virtualizer = createVirtualizer({
    get count() {
      return Math.ceil((props.games?.length || 0) / ITEMS_PER_ROW);
    },
    getScrollElement: () => scrollContainerRef,
    estimateSize: () => ROW_HEIGHT,
    overscan: 4
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
                        <div class="card-cover-media">
                          <img
                            src={game.coverImage}
                            alt={game.title}
                            class="card-img"
                            loading="lazy"
                          />
                          <div class="card-glass-gloss"></div>

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

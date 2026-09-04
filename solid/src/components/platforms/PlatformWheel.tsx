import { Component, For, onMount, Show, createSignal } from 'solid-js';
import { animate } from 'animejs';
import type { PlatformWheelProps, WheelItem } from '@contracts/wheel.types';
import { usePlatformWheelLayout } from './usePlatformWheelLayout';
import { ConsoleHardwareVisual } from '@components/common/ConsoleHardwareVisual';

export type { PlatformWheelHandle } from '@contracts/wheel.types';

export const PlatformWheel: Component<PlatformWheelProps> = (props) => {
  let containerRef!: HTMLDivElement;
  let trackRef!: HTMLDivElement;
  let isTransitioning = false;

  const {
    currentItem,
    currentPlatform,
    gamesCount,
    previewGames,
    allItemSlots
  } = usePlatformWheelLayout({
    platforms: () => props.platforms,
    selectedIndex: () => props.selectedIndex,
    getGamesCountForPlatform: (id) => props.getGamesCountForPlatform(id),
    getPreviewGamesForPlatform: (id) => props.getPreviewGamesForPlatform(id)
  });

  const handleEnterItem = (item?: WheelItem) => {
    if (isTransitioning) return;
    const target = item || currentItem();
    if (!target) return;

    if (target.type === 'section' && target.section) {
      props.onSelectSection(target.section);
      return;
    }

    if (target.type === 'platform' && target.platform) {
      isTransitioning = true;
      if (containerRef) {
        animate(containerRef, {
          opacity: [1, 0],
          scale: [1, 1.05],
          translateY: ['0rem', '-1.5rem'],
          duration: 260,
          ease: 'inCubic',
          onComplete: () => {
            props.onSelectPlatform(target.platform!);
          }
        });
      } else {
        props.onSelectPlatform(target.platform);
      }
    }
  };

  onMount(() => {
    if (props.ref) {
      props.ref({
        triggerEnter: () => handleEnterItem()
      });
    }

    if (containerRef) {
      animate(containerRef, {
        opacity: [0, 1],
        translateY: ['1.5rem', '0rem'],
        duration: 320,
        ease: 'outCubic'
      });
    }
  });

  return (
    <div class="xmb-console-dashboard" ref={containerRef}>
      {/* 1. Dynamic Ambient Aura Behind the Carousel */}
      <div
        class="xmb-ambient-aura"
        style={{
          "background": `radial-gradient(ellipse at 50% 35%, ${currentItem()?.glow || 'rgba(0, 55, 145, 0.45)'} 0%, transparent 65%)`
        }}
      />
      <div class="xmb-wave-strip" />

      {/* 2. Authentic PS3 / Xbox Horizontal Icon Carousel (Icon Above, Info Below) */}
      <div class="xmb-carousel-viewport">
        <div
          class="xmb-carousel-track"
          ref={trackRef}
          style={{
            "transform": `translateX(calc(50vw - ${(props.selectedIndex * 14) + 7}rem))`
          }}
        >
          <For each={allItemSlots()}>
            {(slot) => {
              const item = slot.item;
              const isSelected = slot.offset === 0;

              return (
                <div
                  class={`xmb-carousel-node ${isSelected ? 'selected' : ''}`}
                  onClick={(e) => {
                    e.stopPropagation();
                    if (isSelected) {
                      handleEnterItem(item);
                    } else {
                      props.onNavigateIndex(slot.index);
                    }
                  }}
                >
                  {/* Minimalist Vector Icon on Top */}
                  <div class="xmb-node-icon-wrapper">
                    <ConsoleHardwareVisual
                      platformId={item.id}
                      size={isSelected ? 'lg' : 'md'}
                      class="xmb-hardware-icon"
                    />
                    <Show when={isSelected}>
                      <div class="xmb-node-glow-ring" style={{ "box-shadow": `0 0 2rem ${item.glow || '#00d2ff'}` }} />
                    </Show>
                  </div>

                  {/* Console Name & Information Centered Underneath (Debajo) */}
                  <div class="xmb-node-info-underneath">
                    <span class="xmb-node-fullname">{item.shortName}</span>
                    <span class="xmb-node-subname">{item.name}</span>
                    <Show when={isSelected && item.type === 'platform'}>
                      <span class="xmb-node-counter">{gamesCount().toLocaleString()} Juegos</span>
                    </Show>
                  </div>
                </div>
              );
            }}
          </For>
        </div>
      </div>

      {/* 3. Fluid Games Shelf for Focused Console */}
      <Show when={currentPlatform() && previewGames().length > 0}>
        <div class="xmb-games-shelf-area">
          <div class="xmb-shelf-track">
            <For each={previewGames()}>
              {(game) => {
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
                    class="xmb-shelf-poster"
                    onClick={(e) => {
                      e.stopPropagation();
                      if (currentPlatform()) {
                        props.onSelectPlatform(currentPlatform()!);
                      }
                    }}
                  >
                    <div class="xmb-poster-box">
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
                          loading="lazy"
                          onError={() => setHasImageError(true)}
                        />
                      </Show>
                      <div class="xmb-poster-sheen" />
                    </div>
                    <div class="xmb-poster-caption">{game.title}</div>
                  </div>
                );
              }}
            </For>
          </div>
        </div>
      </Show>
    </div>
  );
};

export default PlatformWheel;

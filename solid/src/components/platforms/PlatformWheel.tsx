import { Component, For, onMount, createEffect, Show } from 'solid-js';
import { animateScreenEnter, animateCameraZoomIntoLibrary, animateWaterEmergence } from '@animations/screen-transitions';
import { animateActiveCardBadgeEntrance, animateConsoleCardSwitch } from '@animations/wheel-animations';
import type { PlatformWheelProps, WheelItem } from '@contracts/wheel.types';
import { usePlatformWheelLayout } from './usePlatformWheelLayout';

export type { PlatformWheelHandle } from '@contracts/wheel.types';

export const PlatformWheel: Component<PlatformWheelProps> = (props) => {
  let containerRef!: HTMLDivElement;
  let domeArchRef!: HTMLDivElement;
  let tiltedLibraryRef!: HTMLDivElement;
  let isTransitioning = false;
  let previousIndex = props.selectedIndex;

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

  // 2-Second Cinematic Journey into the Games Library or Instant Section Transition
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
      if (tiltedLibraryRef && domeArchRef) {
        animateCameraZoomIntoLibrary(tiltedLibraryRef, domeArchRef, () => {
          props.onSelectPlatform(target.platform!);
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

    animateScreenEnter(containerRef);
    setTimeout(() => {
      animateWaterEmergence('.tilted-game-card');
      animateActiveCardBadgeEntrance('.dome-console-card.active-center .dome-card-inner');
    }, 60);
  });

  createEffect(() => {
    const currentIdx = props.selectedIndex;
    const direction = currentIdx > previousIndex ? 1 : -1;
    previousIndex = currentIdx;

    setTimeout(() => {
      animateConsoleCardSwitch('.dome-console-card', direction);
      animateActiveCardBadgeEntrance('.dome-console-card.active-center .dome-card-inner');
      animateWaterEmergence('.tilted-game-card');
    }, 20);
  });

  return (
    <div class="console-wheel-container" ref={containerRef}>
      {/* 1. Dynamic Ambient Stage Spotlight */}
      <div
        class="wheel-stage-spotlight"
        style={{
          "background": `radial-gradient(ellipse at 50% 30%, ${currentItem()?.glow || 'rgba(0, 240, 255, 0.6)'} 0%, transparent 65%)`
        }}
      />

      {/* 2. Wide Sweeping Console & System Arch */}
      <div class="wheel-dome-arch-stage" ref={domeArchRef}>
        <For each={allItemSlots()}>
          {(slot) => {
            const item = slot.item;
            const isSystemCard = item.type === 'section';
            const isCenter = slot.offset === 0;

            const transformStyle = () => {
              const off = slot.offset;
              const xOffset = off * 16.5;
              const yOffset = Math.pow(Math.abs(off), 1.32) * 3.6;
              const zOffset = isCenter ? 4 : -Math.abs(off) * 3.5;
              const rotateZ = off * -7.5;
              const rotateY = off * -12;
              const scale = isCenter ? 1.25 : Math.max(0.72, 1 - Math.abs(off) * 0.09);
              const opacity = isCenter ? 1 : Math.max(0.45, 1 - Math.abs(off) * 0.16);

              return {
                transform: `translateX(${xOffset}rem) translateY(${yOffset}rem) translateZ(${zOffset}rem) rotateZ(${rotateZ}deg) rotateY(${rotateY}deg) scale(${scale})`,
                opacity: `${opacity}`,
                "z-index": `${20 - Math.abs(off)}`
              };
            };

            return (
              <div
                class={`dome-console-card ${isCenter ? 'active-center' : ''} ${isSystemCard ? 'system-settings-card' : ''}`}
                style={transformStyle()}
                onClick={(e) => {
                  e.stopPropagation();
                  if (isCenter) {
                    handleEnterItem(item);
                  } else {
                    props.onNavigateIndex(slot.index);
                  }
                }}
              >
                <div
                  class={`dome-card-inner ${isSystemCard ? 'system-card-inner' : ''}`}
                  style={{
                    "border-color": isCenter ? item.color : 'transparent',
                    "box-shadow": isCenter ? `0 0 3.5rem ${item.glow}` : 'none'
                  }}
                >
                  <Show
                    when={isCenter}
                    fallback={
                      <>
                        <span class="dome-card-code">{item.shortName}</span>
                        <span class="dome-card-name">{item.name}</span>
                      </>
                    }
                  >
                    <div class={`active-card-top-tag ${isSystemCard ? 'system-tag' : ''}`}>
                      {item.tag}
                    </div>

                    <div class="active-card-main-title">
                      <span class="active-card-main-code">{item.shortName}</span>
                      <span class="active-card-full-name">{item.name}</span>
                    </div>

                    <Show when={item.type === 'platform'}>
                      <div class="active-card-specs-row">
                        <span class="active-card-spec-chip">Gen {item.generation}a</span>
                        <span class="active-card-spec-chip">{item.year}</span>
                        <span class="active-card-spec-chip highlight">{gamesCount().toLocaleString()} Juegos</span>
                      </div>
                    </Show>

                    <button
                      class={`active-card-action-btn ${isSystemCard ? 'system-action-btn' : ''}`}
                      onClick={(e) => {
                        e.stopPropagation();
                        handleEnterItem(item);
                      }}
                    >
                      <span class="enter-btn-bubble">A</span>
                      <span>{item.type === 'platform' ? 'ENTRAR AL CATALOGO' : 'ABRIR AJUSTES'}</span>
                    </button>
                  </Show>
                </div>

                <div class="dome-card-shadow" />
              </div>
            );
          }}
        </For>
      </div>

      {/* 3. Tilted 3D Library Preview in Background */}
      <Show when={currentPlatform()}>
        <div class="deep-tilted-library-stage" ref={tiltedLibraryRef}>
          <div class="tilted-library-header">
            <span>CATALOGO EN PROFUNDIDAD</span>
            <span>•</span>
            <span>{currentPlatform()?.name}</span>
          </div>

          <div class="tilted-shelf-grid">
            <For each={previewGames()}>
              {(game) => (
                <div class="tilted-game-card">
                  <img src={game.coverImage} alt={game.title} loading="lazy" />
                  <div class="tilted-game-card-info">
                    <div class="tilted-card-title">{game.title}</div>
                    <div class="tilted-card-meta">PUNTUACION {game.rating.toFixed(1)} • {game.releaseYear}</div>
                  </div>
                </div>
              )}
            </For>
          </div>
        </div>
      </Show>
    </div>
  );
};

export default PlatformWheel;

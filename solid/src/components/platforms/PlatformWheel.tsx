import { Component, For, createMemo, onMount, createEffect } from 'solid-js';
import { animateScreenEnter, animateCameraZoomIntoLibrary, animateWaterEmergence } from '@animations/screen-transitions';
import { animateActiveCardBadgeEntrance, animateConsoleCardSwitch } from '@animations/wheel-animations';
import type { Platform, PlatformId, Game } from '@contracts/game.types';
import type { AppSection } from '@contracts/navigation.types';

export interface PlatformWheelHandle {
  triggerEnter: () => void;
}

export interface WheelItem {
  id: string;
  type: 'platform' | 'section';
  section?: AppSection;
  platform?: Platform;
  name: string;
  shortName: string;
  tag: string;
  color: string;
  glow: string;
  year?: number;
  generation?: number;
}

interface PlatformWheelProps {
  platforms: Platform[];
  selectedIndex: number;
  onSelectPlatform: (platform: Platform) => void;
  onSelectSection: (section: AppSection) => void;
  onNavigateIndex: (index: number) => void;
  getGamesCountForPlatform: (platformId: PlatformId) => number;
  getPreviewGamesForPlatform: (platformId: PlatformId) => Game[];
  ref?: (handle: PlatformWheelHandle) => void;
}

export const PlatformWheel: Component<PlatformWheelProps> = (props) => {
  let containerRef!: HTMLDivElement;
  let domeArchRef!: HTMLDivElement;
  let tiltedLibraryRef!: HTMLDivElement;
  let isTransitioning = false;
  let previousIndex = props.selectedIndex;

  // Build combined wheel items: Consoles + Single Dedicated System Settings Card
  const allWheelItems = createMemo<WheelItem[]>(() => {
    const items: WheelItem[] = props.platforms.map((p) => {
      let brand = { color: '#00f0ff', glow: 'rgba(0, 240, 255, 0.6)', tag: 'CONSOLE SYSTEM' };
      switch (p.id) {
        case 'snes':
        case 'n64':
        case 'gba':
          brand = { color: '#e52521', glow: 'rgba(229, 37, 33, 0.6)', tag: 'NINTENDO' };
          break;
        case 'ps1':
        case 'ps2':
          brand = { color: '#006FCD', glow: 'rgba(0, 111, 205, 0.6)', tag: 'SONY PLAYSTATION' };
          break;
        case 'genesis':
        case 'dreamcast':
          brand = { color: '#0088cc', glow: 'rgba(0, 136, 204, 0.6)', tag: 'SEGA' };
          break;
        case 'arcade':
          brand = { color: '#f59e0b', glow: 'rgba(245, 158, 11, 0.6)', tag: 'ARCADE COIN-OP' };
          break;
      }
      return {
        id: p.id,
        type: 'platform',
        platform: p,
        name: p.name,
        shortName: p.shortName,
        tag: brand.tag,
        color: brand.color,
        glow: brand.glow,
        year: p.releaseYear,
        generation: p.generation
      };
    });

    // Single Distinct System Gateway Card (Pantalla, Núcleos, Audio, Hardware)
    items.push({
      id: 'system-settings',
      type: 'section',
      section: 'settings',
      name: 'Pantalla • Núcleos • Audio • Mandos',
      shortName: 'AJUSTES',
      tag: 'HERRAMIENTAS DEL SISTEMA',
      color: '#10b981',
      glow: 'rgba(16, 185, 129, 0.7)'
    });

    return items;
  });

  const currentItem = createMemo(() => {
    const list = allWheelItems();
    if (list.length === 0) return null;
    return list[props.selectedIndex] || list[0];
  });

  const currentPlatform = createMemo(() => {
    const item = currentItem();
    return item?.type === 'platform' ? item.platform || null : null;
  });

  const gamesCount = createMemo(() => {
    const plat = currentPlatform();
    if (!plat) return 0;
    return props.getGamesCountForPlatform(plat.id);
  });

  const previewGames = createMemo(() => {
    const plat = currentPlatform();
    if (!plat) return [];
    return props.getPreviewGamesForPlatform(plat.id).slice(0, 8);
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

  // Calculate wrapped slots along the wide sweeping arch
  const allItemSlots = createMemo(() => {
    const list = allWheelItems();
    const total = list.length;
    if (total === 0) return [];

    return list.map((item, index) => {
      let diff = index - props.selectedIndex;
      if (diff > total / 2) diff -= total;
      if (diff < -total / 2) diff += total;

      return {
        offset: diff,
        item,
        index,
        isCenter: diff === 0
      };
    });
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

            // Wide Sweeping Curve Math across full screen width
            const transformStyle = () => {
              const off = slot.offset;
              const xOffset = off * 16.5; // rem (wide horizontal span)
              const yOffset = Math.pow(Math.abs(off), 1.32) * 3.6; // rem
              const zOffset = slot.isCenter ? 4 : -Math.abs(off) * 3.5; // rem
              const rotateZ = off * -7.5; // deg
              const rotateY = off * -12; // deg
              const scale = slot.isCenter ? 1.25 : Math.max(0.72, 1 - Math.abs(off) * 0.09);
              const opacity = slot.isCenter ? 1 : Math.max(0.45, 1 - Math.abs(off) * 0.16);

              return {
                transform: `translateX(${xOffset}rem) translateY(${yOffset}rem) translateZ(${zOffset}rem) rotateZ(${rotateZ}deg) rotateY(${rotateY}deg) scale(${scale})`,
                opacity: `${opacity}`,
                "z-index": `${20 - Math.abs(off)}`
              };
            };

            return (
              <div
                class={`dome-console-card ${slot.isCenter ? 'active-center' : ''} ${isSystemCard ? 'system-settings-card' : ''}`}
                style={transformStyle()}
                onClick={(e) => {
                  e.stopPropagation();
                  if (slot.isCenter) {
                    handleEnterItem(item);
                  } else {
                    props.onNavigateIndex(slot.index);
                  }
                }}
              >
                <div
                  class={`dome-card-inner ${isSystemCard ? 'system-card-inner' : ''}`}
                  style={{
                    "border-color": slot.isCenter ? item.color : 'transparent',
                    "box-shadow": slot.isCenter ? `0 0 3.5rem ${item.glow}` : 'none'
                  }}
                >
                  {slot.isCenter ? (
                    // Center Card: Full Embedded Console / System Details
                    <>
                      <div class={`active-card-top-tag ${isSystemCard ? 'system-tag' : ''}`}>
                        {item.tag}
                      </div>

                      <div class="active-card-main-title">
                        {isSystemCard && <span class="system-gear-emblem">⚙️</span>}
                        <span class="active-card-main-code">{item.shortName}</span>
                        <span class="active-card-full-name">{item.name}</span>
                      </div>

                      {item.type === 'platform' && (
                        <div class="active-card-specs-row">
                          <span class="active-card-spec-chip">Gen {item.generation}ª</span>
                          <span class="active-card-spec-chip">{item.year}</span>
                          <span class="active-card-spec-chip highlight">{gamesCount().toLocaleString()} Juegos</span>
                        </div>
                      )}

                      <button
                        class={`active-card-action-btn ${isSystemCard ? 'system-action-btn' : ''}`}
                        onClick={(e) => {
                          e.stopPropagation();
                          handleEnterItem(item);
                        }}
                      >
                        <span class="enter-btn-bubble">A</span>
                        <span>{item.type === 'platform' ? 'ENTRAR AL CATÁLOGO' : 'ABRIR AJUSTES'}</span>
                      </button>
                    </>
                  ) : (
                    // Flanking Inactive Cards: Emblem Badge
                    <>
                      {isSystemCard && <span class="inactive-system-icon">⚙️</span>}
                      <span class="dome-card-code">{item.shortName}</span>
                      <span class="dome-card-name">{item.name}</span>
                    </>
                  )}
                </div>

                <div class="dome-card-shadow" />
              </div>
            );
          }}
        </For>
      </div>

      {/* 3. Tilted 3D Library Preview in Background (Only for platforms) */}
      {currentPlatform() && (
        <div class="deep-tilted-library-stage" ref={tiltedLibraryRef}>
          <div class="tilted-library-header">
            <span>CATÁLOGO EN PROFUNDIDAD</span>
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
                    <div class="tilted-card-meta">★ {game.rating.toFixed(1)} • {game.releaseYear}</div>
                  </div>
                </div>
              )}
            </For>
          </div>
        </div>
      )}
    </div>
  );
};

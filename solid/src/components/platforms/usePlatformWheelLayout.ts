import { createMemo } from 'solid-js';
import type {
  WheelItem,
  WheelSlot,
  UsePlatformWheelLayoutOptions,
  UsePlatformWheelLayoutReturn
} from '@contracts/wheel.types';

export function usePlatformWheelLayout(options: UsePlatformWheelLayoutOptions): UsePlatformWheelLayoutReturn {
  const { platforms, selectedIndex, getGamesCountForPlatform, getPreviewGamesForPlatform } = options;

  const allWheelItems = createMemo<WheelItem[]>(() => {
    const items: WheelItem[] = platforms().map((p) => {
      let brand: { color: string; glow: string; tag: string };

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
        default:
          brand = { color: '#00f0ff', glow: 'rgba(0, 240, 255, 0.6)', tag: 'CONSOLE SYSTEM' };
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

    // Single Gateway Card for System Tools
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
    return list.length === 0 ? null : (list[selectedIndex()] || list[0]);
  });

  const currentPlatform = createMemo(() => {
    const item = currentItem();
    return item?.type === 'platform' ? item.platform || null : null;
  });

  const gamesCount = createMemo(() => {
    const plat = currentPlatform();
    return plat ? getGamesCountForPlatform(plat.id) : 0;
  });

  const previewGames = createMemo(() => {
    const plat = currentPlatform();
    return plat ? getPreviewGamesForPlatform(plat.id).slice(0, 8) : [];
  });

  const allItemSlots = createMemo<WheelSlot[]>(() => {
    const list = allWheelItems();
    const total = list.length;
    if (total === 0) return [];

    return list.map((item, index) => {
      let diff = index - selectedIndex();
      if (diff > total / 2) diff -= total;
      if (diff < -total / 2) diff += total;

      const dist = Math.abs(diff);
      const isVisible = dist <= 4;
      const angle = diff * 0.16;
      const radiusX = 58;
      const radiusY = 7;

      const translateX = Math.sin(angle) * radiusX;
      const translateY = -(1 - Math.cos(angle)) * radiusY;
      const scale = dist === 0 ? 1.0 : Math.max(0.72, 0.94 - dist * 0.08);
      const rotateZ = diff * 4.5;
      const opacity = dist === 0 ? 1 : Math.max(0.15, 0.9 - dist * 0.22);
      const zIndex = 100 - Math.round(dist * 10);

      return {
        offset: diff,
        item,
        index,
        distance: dist,
        isVisible,
        angleDeg: (angle * 180) / Math.PI,
        translateX,
        translateY,
        scale,
        rotateZ,
        opacity,
        zIndex
      };
    });
  });

  return {
    allWheelItems,
    currentItem,
    currentPlatform,
    gamesCount,
    previewGames,
    allItemSlots
  };
}

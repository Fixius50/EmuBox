import type { Accessor } from 'solid-js';
import type { Platform, PlatformId, Game } from './game.types';
import type { AppSection } from './navigation.types';

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

export interface WheelSlot {
  offset: number;
  item: WheelItem;
  index: number;
  distance: number;
  isVisible: boolean;
  angleDeg: number;
  translateX: number;
  translateY: number;
  scale: number;
  rotateZ: number;
  opacity: number;
  zIndex: number;
}

export interface UsePlatformWheelLayoutOptions {
  platforms: () => Platform[];
  selectedIndex: () => number;
  getGamesCountForPlatform: (platformId: PlatformId) => number;
  getPreviewGamesForPlatform: (platformId: PlatformId) => Game[];
}

export interface UsePlatformWheelLayoutReturn {
  allWheelItems: Accessor<WheelItem[]>;
  currentItem: Accessor<WheelItem | null>;
  currentPlatform: Accessor<Platform | null>;
  gamesCount: Accessor<number>;
  previewGames: Accessor<Game[]>;
  allItemSlots: Accessor<WheelSlot[]>;
}

export interface PlatformWheelProps {
  platforms: Platform[];
  selectedIndex: number;
  onSelectPlatform: (platform: Platform) => void;
  onSelectSection: (section: AppSection) => void;
  onNavigateIndex: (index: number) => void;
  getGamesCountForPlatform: (platformId: PlatformId) => number;
  getPreviewGamesForPlatform: (platformId: PlatformId) => Game[];
  ref?: (handle: PlatformWheelHandle) => void;
}

export type NavDirection = 'up' | 'down' | 'left' | 'right' | 'NAV_UP' | 'NAV_DOWN' | 'NAV_LEFT' | 'NAV_RIGHT';

export type AppSection = 'library' | 'emulators' | 'settings';

export type LibraryViewMode = 'wheel' | 'games';

export interface SpatialRect {
  left?: number;
  top?: number;
  right?: number;
  bottom?: number;
  width: number;
  height: number;
  x?: number;
  y?: number;
}

export interface SpatialNode {
  id: string;
  containerId: string;
  element?: HTMLElement;
  rect?: SpatialRect;
  disabled?: boolean;
  priority?: number;
}

export interface SpatialContainer {
  id: string;
  defaultFocusId?: string;
  lastFocusId?: string;
  isTrap?: boolean;
}

export type FocusChangeListener = (currentId: string | null, prevId: string | null) => void;

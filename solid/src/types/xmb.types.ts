import type { CatalogGroup } from "@services/library/catalog-groups";
import type { Game, Platform } from "./game.types";
import type { InputAction, InputDeviceStatus } from "./input.types";

export interface XmbFolder {
  id: string;
  title: string;
  games: CatalogGroup[];
}

export interface XmbPosition {
  category: number;
  row: number;
  expanded: boolean;
  game: number;
}

export type XmbCommand = "left" | "right" | "up" | "down" | "enter" | "back";
export interface XmbBounds {
  categories: number;
  rows: number;
  games: number;
}

export interface XmbLibraryProps {
  games: CatalogGroup[];
  platforms: Platform[];
  loading: boolean;
  downloadingIds: Set<string>;
  inputStatus: InputDeviceStatus;
  error?: { gameId: string; message: string } | null;
  onOpenGame: (game: Game) => void;
  onOpenSettings: (tab: string) => void;
  onMaintenance: () => void;
  onFavorite: (id: string) => void;
  onMove: () => void;
  onControllerReady: (handler: ((action: InputAction) => void) | null) => void;
}

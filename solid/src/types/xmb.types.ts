import type { CatalogGroup } from "@services/library/catalog-groups";
import type { JSX } from "solid-js";
import type { Game, Platform } from "./game.types";
import type { DownloadJob } from "./download.types";
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

export interface VirtualKeyboardKey {
  label: string;
  value: string;
}

export interface XmbLibraryProps {
  games: CatalogGroup[];
  platforms: Platform[];
  loading: boolean;
  loadingMessage?: string;
  loadError?: string;
  downloadingIds: Set<string>;
  downloadJobs: DownloadJob[];
  inputStatus: InputDeviceStatus;
  error?: { gameId: string; message: string } | null;
  onOpenGame: (game: Game) => void;
  onOpenSettings: (tab: string) => void;
  onCloseSettings?: () => void;
  settingsPanel?: JSX.Element;
  onFavorite: (id: string) => void;
  onMove: () => void;
  onControllerReady: (handler: ((action: InputAction) => void) | null) => void;
}

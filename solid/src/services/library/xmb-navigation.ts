import type { CatalogGroup } from './catalog-groups';

export interface XmbFolder { id: string; title: string; games: CatalogGroup[] }

export function xmbFolders(games: CatalogGroup[]): XmbFolder[] {
  const folders = new Map<string, XmbFolder>();
  for (const game of games) {
    const id = game.title.normalize('NFKC').toLowerCase();
    const folder = folders.get(id);
    if (folder) folder.games.push(game);
    else folders.set(id, { id, title: game.title, games: [game] });
  }
  return [...folders.values()];
}

export interface XmbPosition {
  category: number;
  row: number;
  expanded: boolean;
  game: number;
}

export type XmbCommand = 'left' | 'right' | 'up' | 'down' | 'enter' | 'back';
export interface XmbBounds { categories: number; rows: number; games: number }

export function moveXmb(position: XmbPosition, command: XmbCommand, bounds: XmbBounds): XmbPosition {
  const next = { ...position };
  if (command === 'back') {
    if (next.expanded) { next.expanded = false; next.game = 0; }
    else next.row = 0;
  } else if (command === 'enter') {
    if (next.row === 0 && bounds.rows > 0) next.row = 1;
    else if (bounds.games > 0) next.expanded = true;
  } else if (command === 'up' || command === 'down') {
    next.row = Math.max(0, Math.min(bounds.rows, next.row + (command === 'down' ? 1 : -1)));
    next.expanded = false;
    next.game = 0;
  } else if (next.expanded) {
    if (command === 'left' && next.game === 0) next.expanded = false;
    else next.game = Math.max(0, Math.min(bounds.games - 1, next.game + (command === 'right' ? 1 : -1)));
  } else {
    next.category = Math.max(0, Math.min(bounds.categories - 1, next.category + (command === 'right' ? 1 : -1)));
    if (next.category !== position.category) { next.row = 0; next.game = 0; }
  }
  return next;
}
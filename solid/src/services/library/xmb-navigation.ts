import type { CatalogGroup } from "./catalog-groups";
import type {
  XmbBounds,
  XmbCommand,
  XmbFolder,
  XmbPosition,
} from "@contracts/xmb.types";
export type {
  XmbBounds,
  XmbCommand,
  XmbFolder,
  XmbPosition,
} from "@contracts/xmb.types";

export function xmbFolders(games: CatalogGroup[]): XmbFolder[] {
  const folders = new Map<string, XmbFolder>();
  for (const game of games) {
    const id = game.canonicalId || game.title.normalize("NFKC").toLowerCase();
    const folder = folders.get(id);
    let versions: CatalogGroup[] | undefined;
    const entries = () => versions ??= game.canonicalId
      ? game.variants.map(variant => ({ ...game, ...variant,
        title: variant.releaseTitle || variant.title, favorite: game.favorite, variants: [variant],
      })) : [game];
    if (folder) folder.games.push(...entries());
    else folders.set(id, { id, title: game.canonicalTitle || game.title, get games() { return entries(); } });
  }
  return [...folders.values()];
}

export function moveXmb(
  position: XmbPosition,
  command: XmbCommand,
  bounds: XmbBounds,
): XmbPosition {
  const next = { ...position };
  if (command === "back") {
    if (next.expanded) {
      next.expanded = false;
      next.game = 0;
    } else next.row = 0;
  } else if (command === "enter") {
    if (next.row === 0 && bounds.rows > 0) next.row = 1;
    else if (bounds.games > 0) next.expanded = true;
  } else if (command === "up" || command === "down") {
    next.row = Math.max(
      0,
      Math.min(bounds.rows, next.row + (command === "down" ? 1 : -1)),
    );
    next.expanded = false;
    next.game = 0;
  } else if (next.expanded) {
    if (command === "left" && next.game === 0) next.expanded = false;
    else
      next.game = Math.max(
        0,
        Math.min(bounds.games - 1, next.game + (command === "right" ? 1 : -1)),
      );
  } else {
    next.category = Math.max(
      0,
      Math.min(
        bounds.categories - 1,
        next.category + (command === "right" ? 1 : -1),
      ),
    );
    if (next.category !== position.category) {
      next.row = 0;
      next.game = 0;
    }
  }
  return next;
}

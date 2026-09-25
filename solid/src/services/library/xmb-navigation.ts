import { catalogTitle, type CatalogGroup } from "./catalog-groups";
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

function seriesRoot(title: string): string | undefined {
  return title.match(/^(007)(?:\s*[.:-]\s*|\s+)[\p{L}].+/u)?.[1]
    || title.match(/^(.{4,}?)\s+(?:[2-9]|[1-9]\d{1,2}|ii|iii|iv|v|vi|vii|viii|ix|x)$/i)?.[1]
    || title.match(/^(.{4,}?):\s+.+$/)?.[1];
}

function packageRoot(title: string): string | undefined {
  return title.match(/^(.+?)(?=\s*[[(](?:v\.?\s*\d|repack\b|pre-installed\b|pre-instalado\b|rus\b|eng\b|build\b)|\s+v\.?\s*\d|\s+pc\s*\|\s*portable\b)/i)?.[1].trim();
}

export function xmbFolders(games: CatalogGroup[]): XmbFolder[] {
  const folders = new Map<string, XmbFolder>();
  const titles = new Set(games.map(game => game.title.normalize("NFKC").toLowerCase()));
  const packages = new Set(games.flatMap(game => {
    const title = game.title.normalize("NFKC").toLowerCase();
    const root = game.matchMethod === "local-title" && packageRoot(title);
    return root && titles.has(root) ? [root] : [];
  }));
  const candidates = new Map<string, Set<string>>();
  for (const game of games) {
    const title = game.title.normalize("NFKC").toLowerCase();
    const root = seriesRoot(title);
    if (!root) continue;
    if (!candidates.has(root)) candidates.set(root, new Set());
    candidates.get(root)!.add(title);
  }
  const series = new Set([...candidates].filter(([root, entries]) => titles.has(root) || entries.size > 1).map(([root]) => root));
  for (const game of games) {
    const title = game.title.normalize("NFKC").toLowerCase();
    const packageTitle = packageRoot(title);
    let family = game.matchMethod === "local-title" && packageTitle && titles.has(packageTitle) ? packageTitle : title;
    let root = seriesRoot(family);
    while (root && series.has(root)) {
      family = root;
      root = seriesRoot(family);
    }
    const id = series.has(family) || packages.has(family) || game.matchMethod === "local-title"
      ? family : game.canonicalId || title;
    const folder = folders.get(id);
    let versions: CatalogGroup[] | undefined;
    const entries = () => versions ??= game.canonicalId
      ? game.variants.map(variant => ({ ...game, ...variant,
        title: variant.releaseTitle || catalogTitle(variant.title), favorite: game.favorite, variants: [variant],
        coverImage: variant.coverImage || game.coverImage,
        description: variant.description || game.description,
        genre: variant.genre || game.genre,
        developer: variant.developer || game.developer,
        publisher: variant.publisher || game.publisher,
      })) : [game];
    if (folder) folder.games.push(...entries());
    else folders.set(id, { id, title: family !== title ? game.title.slice(0, family.length) : game.title, get games() { return entries(); } });
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

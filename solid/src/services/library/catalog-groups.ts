import type { Game } from '@contracts/game.types';

export interface CatalogGroup extends Game {
  variants: Game[];
}

export function catalogTitle(title: string): string {
  const cleaned = title.normalize('NFKC')
    .replace(/\s*[[(](?:19\d{2}|20\d{2}|multi(?:\s*\d+)?|p2p|pre-instalado|pre-installed|repack\s+(?:fitgirl|dodi|elamigos)|scene\s+skidrow|v?\d+(?:\.\d+)+(?:\s*\|\s*build\s+\d+)?|build\s+\d+)[\])]\s*/gi, ' ')
    .replace(/\s+free download\b/gi, '')
    .replace(/\s+(?:repack|scene)\s+(?:fitgirl|dodi|skidrow|elamigos|kaoskrew)\s*$/i, '')
    .replace(/\s*[-–—]\s*(?:fitgirl|dodi|skidrow|elamigos|p2p)\s*$/i, '')
    .replace(/\s*[-–—]\s*build\s+\d+\s*$/i, '')
    .replace(/\s+/g, ' ').trim();
  return cleaned || title.trim();
}

export function groupCatalog(games: Game[]): CatalogGroup[] {
  const variants = new Map<string, Game[]>();
  for (const game of games) {
    const year = game.title.match(/\((19\d{2}|20\d{2})\)/)?.[1] || (game.releaseYear > 0 ? String(game.releaseYear) : 'unknown');
    const key = JSON.stringify([game.platform, catalogTitle(game.title).toLowerCase(), year]);
    const existing = variants.get(key);
    if (existing) existing.push(game);
    else variants.set(key, [game]);
  }
  return [...variants.values()].map(entries => {
    const sorted = [...entries].sort((left, right) => left.id.localeCompare(right.id));
    const representative = sorted.find(game => game.installed) || sorted[0];
    const metadata = [...sorted].sort((left, right) =>
      Number(Boolean(right.coverImage)) + Number(Boolean(right.description)) -
      Number(Boolean(left.coverImage)) - Number(Boolean(left.description)))[0];
    return { ...metadata, ...representative, title: catalogTitle(representative.title),
      coverImage: representative.coverImage || metadata.coverImage,
      description: representative.description || metadata.description,
      favorite: sorted.some(game => game.favorite), variants: sorted };
  });
}
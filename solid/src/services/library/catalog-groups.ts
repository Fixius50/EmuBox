import type { Game } from '@contracts/game.types';

export interface CatalogGroup extends Game {
  variants: Game[];
}

export function catalogTitle(title: string): string {
  const cleaned = title.normalize('NFKC')
    .replace(/\s*[[(](?:19\d{2}|20\d{2}|(?:ru|en|es|fr|de|it|pt|ja|multi\d*)(?:\s*[-/,]\s*(?:ru|en|es|fr|de|it|pt|ja|multi\d*))*|multi\s*\d+|p2p|gog|steam|epic|skidrow|pre-instalado|pre-installed|repack\s+(?:fitgirl|dodi|elamigos)|scene\s+skidrow|v?\s*\d+(?:\.\d+)+(?:\s*\|\s*build\s+\d+)?|v\d{8}(?:-p2p)?|build\s+\d+)[\])]\s*/gi, ' ')
    .replace(/\s+free download\b/gi, '')
    .replace(/\s+(?:(?:pc\s*\|\s*)?repack|scene|license)\s+[\p{L}\p{N}_'. -]+\s*$/iu, '')
    .replace(/\s*[-–—]\s*(?:fitgirl|dodi|skidrow|elamigos|p2p)\s*$/i, '')
    .replace(/\s*[-–—]\s*build\s+\d+(?:\s*\+\s*[^+]+\s+DLC)?\s*$/i, '')
    .replace(/\s+\+\s+(?:\d+\s+)?DLCs?\s*$/i, '')
    .replace(/\s+/g, ' ').trim();
  return cleaned || title.trim();
}

export function groupCatalog(games: Game[]): CatalogGroup[] {
  const titles = new Map<string, { game: Game; year: string | null }[]>();
  for (const game of games) {
    const year = game.title.match(/\((19\d{2}|20\d{2})\)/)?.[1] || (game.releaseYear > 0 ? String(game.releaseYear) : null);
    const key = JSON.stringify([game.platform, catalogTitle(game.title).toLowerCase()]);
    const existing = titles.get(key);
    if (existing) existing.push({ game, year });
    else titles.set(key, [{ game, year }]);
  }
  const variants: Game[][] = [];
  for (const entries of titles.values()) {
    const knownYears = new Set(entries.flatMap(entry => entry.year ? [entry.year] : []));
    if (knownYears.size <= 1) {
      variants.push(entries.map(entry => entry.game));
    } else {
      const byYear = new Map<string, Game[]>();
      for (const { game, year } of entries) {
        const key = year || 'unknown';
        const existing = byYear.get(key);
        if (existing) existing.push(game);
        else byYear.set(key, [game]);
      }
      variants.push(...byYear.values());
    }
  }
  return variants.map(entries => {
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
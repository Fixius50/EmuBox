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
    .replace(/(?:\s*\((?:tenoke|rune|plaza)\)|\s*[-–—]\s*(?:tenoke|rune|plaza))\s*$/i, '')
    .replace(/\s*[-–—]\s*$/, '')
    .replace(/\s+/g, ' ').trim();
  return cleaned || title.trim();
}

export function groupCatalog(games: Game[]): CatalogGroup[] {
  const titles = new Map<string, { game: Game; year: string | null }[]>();
  for (const game of games) {
    const year = game.title.match(/\((19\d{2}|20\d{2})\)/)?.[1] || (game.releaseYear > 0 ? String(game.releaseYear) : null);
    const key = game.canonicalId || JSON.stringify([game.platform, catalogTitle(game.title).toLowerCase()]);
    const existing = titles.get(key);
    if (existing) existing.push({ game, year });
    else titles.set(key, [{ game, year }]);
  }
  const variants: Game[][] = [];
  for (const entries of titles.values()) {
    const knownYears = new Set(entries.flatMap(entry => entry.year ? [entry.year] : []));
    if (entries[0].game.canonicalId || knownYears.size <= 1) {
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
    const available = (field: 'coverImage' | 'description' | 'genre' | 'developer' | 'publisher') =>
      representative[field] || sorted.find(game => game[field])?.[field] || '';
    return { ...representative,
      title: catalogTitle(representative.canonicalTitle || representative.title),
      coverImage: available('coverImage'), description: available('description'),
      genre: available('genre'), developer: available('developer'), publisher: available('publisher'),
      favorite: sorted.some(game => game.favorite), variants: sorted };
  });
}
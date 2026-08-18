import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const platforms = [
  { id: 'snes', name: 'Super Nintendo', emu: 'snes9x', startYear: 1990, endYear: 1996 },
  { id: 'ps1', name: 'Sony PlayStation', emu: 'duckstation', startYear: 1994, endYear: 2000 },
  { id: 'ps2', name: 'Sony PlayStation 2', emu: 'pcsx2', startYear: 2000, endYear: 2007 },
  { id: 'n64', name: 'Nintendo 64', emu: 'mupen64plus', startYear: 1996, endYear: 2001 },
  { id: 'genesis', name: 'Sega Genesis', emu: 'genesis-plus-gx', startYear: 1989, endYear: 1995 },
  { id: 'gba', name: 'Game Boy Advance', emu: 'mgba', startYear: 2001, endYear: 2006 },
  { id: 'dreamcast', name: 'Sega Dreamcast', emu: 'flycast', startYear: 1998, endYear: 2002 },
  { id: 'arcade', name: 'Arcade', emu: 'fbneo', startYear: 1985, endYear: 2000 }
];

const genres = ['Action', 'RPG', 'Platformer', 'Fighting', 'Racing', 'Adventure', 'Shooter', 'Strategy', 'Sports', 'Survival Horror'];
const developers = ['Capcom', 'Konami', 'Square Enix', 'Nintendo', 'Sega', 'Namco', 'SNK', 'Naughty Dog', 'Rare', 'Atlus', 'Midway', 'LucasArts'];

// Seeded PRNG for reproducible dataset generation
function pseudoRandom(seed) {
  let value = seed % 2147483647;
  if (value <= 0) value += 2147483646;
  return function() {
    value = (value * 16807) % 2147483647;
    return (value - 1) / 2147483646;
  };
}

// Hand-curated iconic classic game bases to ensure realistic titles
const iconicGames = [
  { title: "Super Mario World", platform: "snes", genre: "Platformer", dev: "Nintendo", year: 1990, rating: 4.9 },
  { title: "The Legend of Zelda: A Link to the Past", platform: "snes", genre: "Adventure", dev: "Nintendo", year: 1991, rating: 5.0 },
  { title: "Super Metroid", platform: "snes", genre: "Action", dev: "Nintendo", year: 1994, rating: 4.9 },
  { title: "Chrono Trigger", platform: "snes", genre: "RPG", dev: "Square Enix", year: 1995, rating: 5.0 },
  { title: "Final Fantasy VI", platform: "snes", genre: "RPG", dev: "Square Enix", year: 1994, rating: 4.9 },
  { title: "Donkey Kong Country", platform: "snes", genre: "Platformer", dev: "Rare", year: 1994, rating: 4.8 },
  { title: "Castlevania: Symphony of the Night", platform: "ps1", genre: "Action", dev: "Konami", year: 1997, rating: 5.0 },
  { title: "Metal Gear Solid", platform: "ps1", genre: "Action", dev: "Konami", year: 1998, rating: 4.9 },
  { title: "Final Fantasy VII", platform: "ps1", genre: "RPG", dev: "Square Enix", year: 1997, rating: 4.9 },
  { title: "Resident Evil 2", platform: "ps1", genre: "Survival Horror", dev: "Capcom", year: 1998, rating: 4.8 },
  { title: "Crash Bandicoot 3: Warped", platform: "ps1", genre: "Platformer", dev: "Naughty Dog", year: 1998, rating: 4.7 },
  { title: "Tekken 3", platform: "ps1", genre: "Fighting", dev: "Namco", year: 1997, rating: 4.9 },
  { title: "Grand Theft Auto: San Andreas", platform: "ps2", genre: "Action", dev: "Capcom", year: 2004, rating: 4.9 },
  { title: "Shadow of the Colossus", platform: "ps2", genre: "Adventure", dev: "Sony", year: 2005, rating: 4.9 },
  { title: "God of War II", platform: "ps2", genre: "Action", dev: "Sony", year: 2007, rating: 4.8 },
  { title: "Silent Hill 2", platform: "ps2", genre: "Survival Horror", dev: "Konami", year: 2001, rating: 5.0 },
  { title: "Super Mario 64", platform: "n64", genre: "Platformer", dev: "Nintendo", year: 1996, rating: 4.9 },
  { title: "The Legend of Zelda: Ocarina of Time", platform: "n64", genre: "Adventure", dev: "Nintendo", year: 1998, rating: 5.0 },
  { title: "GoldenEye 007", platform: "n64", genre: "Shooter", dev: "Rare", year: 1997, rating: 4.8 },
  { title: "Sonic the Hedgehog 2", platform: "genesis", genre: "Platformer", dev: "Sega", year: 1992, rating: 4.8 },
  { title: "Streets of Rage 2", platform: "genesis", genre: "Action", dev: "Sega", year: 1992, rating: 4.9 },
  { title: "Pokemon Emerald", platform: "gba", genre: "RPG", dev: "Nintendo", year: 2004, rating: 4.9 },
  { title: "The Legend of Zelda: The Minish Cap", platform: "gba", genre: "Adventure", dev: "Capcom", year: 2004, rating: 4.8 },
  { title: "Metroid Fusion", platform: "gba", genre: "Action", dev: "Nintendo", year: 2002, rating: 4.8 },
  { title: "Shenmue", platform: "dreamcast", genre: "Adventure", dev: "Sega", year: 1999, rating: 4.7 },
  { title: "Crazy Taxi", platform: "dreamcast", genre: "Racing", dev: "Sega", year: 1999, rating: 4.6 },
  { title: "Marvel vs. Capcom 2", platform: "dreamcast", genre: "Fighting", dev: "Capcom", year: 2000, rating: 4.9 },
  { title: "Street Fighter II: Champion Edition", platform: "arcade", genre: "Fighting", dev: "Capcom", year: 1992, rating: 5.0 },
  { title: "Metal Slug 3", platform: "arcade", genre: "Action", dev: "SNK", year: 2000, rating: 4.9 },
  { title: "Pac-Man", platform: "arcade", genre: "Action", dev: "Namco", year: 1980, rating: 4.7 }
];

const titlePrefixes = ['Super', 'Ultra', 'Neo', 'Final', 'Mega', 'Cyber', 'Dark', 'Hyper', 'Shin', 'Galactic', 'Dragon', 'Silent', 'Fatal', 'Shadow', 'Chrono', 'Legendary'];
const titleNouns = ['Quest', 'Fighter', 'Racer', 'Warrior', 'Chronicles', 'Adventure', 'Strike', 'Odyssey', 'Force', 'Blade', 'Vengeance', 'Zero', 'Trigger', 'Legacy', 'Storm', 'Buster', 'Galaxy', 'Empire'];
const titleSuffixes = ['DX', 'Turbo', 'Special', 'Championship Edition', 'Revival', 'Gaiden', 'Prime', 'Zero', 'II', 'III', 'IV', 'X', 'EX', 'Plus'];

function generateSvgCover(title, platformId, color, seed) {
  // Generate a clean stylized SVG poster encoded as data URI for zero network dependency
  const bg1 = color || '#3b82f6';
  const shortTitle = title.length > 22 ? title.substring(0, 20) + '...' : title;
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 220 280" width="220" height="280">
    <defs>
      <linearGradient id="g${seed}" x1="0%" y1="0%" x2="100%" y2="100%">
        <stop offset="0%" stop-color="${bg1}" stop-opacity="0.8"/>
        <stop offset="100%" stop-color="#07090e" stop-opacity="0.95"/>
      </linearGradient>
    </defs>
    <rect width="220" height="280" rx="8" fill="url(#g${seed})"/>
    <circle cx="110" cy="110" r="55" fill="none" stroke="${bg1}" stroke-width="3" opacity="0.4"/>
    <polygon points="110,75 140,130 80,130" fill="${bg1}" opacity="0.3"/>
    <text x="110" y="120" font-family="sans-serif" font-size="28" font-weight="900" fill="#ffffff" text-anchor="middle" opacity="0.85">${platformId.toUpperCase()}</text>
    <rect x="16" y="200" width="188" height="60" rx="6" fill="rgba(0,0,0,0.6)"/>
    <text x="110" y="235" font-family="sans-serif" font-size="12" font-weight="700" fill="#ffffff" text-anchor="middle">${shortTitle}</text>
  </svg>`;
  return `data:image/svg+xml;utf8,${encodeURIComponent(svg)}`;
}

function generateGame(index, rng) {
  let title, platform, genre, dev, year, rating;

  if (index < iconicGames.length) {
    const base = iconicGames[index];
    title = base.title;
    platform = platforms.find(p => p.id === base.platform) || platforms[0];
    genre = base.genre;
    dev = base.dev;
    year = base.year;
    rating = base.rating;
  } else {
    const pIdx = Math.floor(rng() * platforms.length);
    platform = platforms[pIdx];
    const prefix = titlePrefixes[Math.floor(rng() * titlePrefixes.length)];
    const noun = titleNouns[Math.floor(rng() * titleNouns.length)];
    const suffix = rng() > 0.6 ? ' ' + titleSuffixes[Math.floor(rng() * titleSuffixes.length)] : '';
    title = `${prefix} ${noun}${suffix} #${index + 1}`;
    genre = genres[Math.floor(rng() * genres.length)];
    dev = developers[Math.floor(rng() * developers.length)];
    year = Math.floor(platform.startYear + rng() * (platform.endYear - platform.startYear + 1));
    rating = Math.round((3.5 + rng() * 1.5) * 10) / 10;
  }

  const isFav = index % 7 === 0 || index < 4;
  const playTime = Math.floor(rng() * 720); // 0 to 12 hours

  return {
    id: `game-${index + 1}`,
    title,
    platform: platform.id,
    platformName: platform.name,
    emulatorId: platform.emu,
    releaseYear: year,
    genre,
    developer: dev,
    publisher: dev,
    rating,
    favorite: isFav,
    coverImage: generateSvgCover(title, platform.id, platform.color || '#3b82f6', index),
    backdropImage: "",
    description: `Una experiencia clásica de ${genre} desarrollada por ${dev} para ${platform.name} en ${year}. Aclamado por su apartado técnico y jugabilidad atemporal en EmuBox.`,
    playTimeMinutes: playTime,
    lastPlayed: playTime > 0 ? "2026-08-10" : null
  };
}

function generateDataset(count) {
  const rng = pseudoRandom(42 + count);
  const games = [];
  for (let i = 0; i < count; i++) {
    games.push(generateGame(i, rng));
  }
  return games;
}

const counts = [20, 100, 500, 1000, 5000, 10000];

console.log("Generando datasets reproducibles para EmuBox-Lab...");
counts.forEach(count => {
  const data = generateDataset(count);
  const filePath = path.join(__dirname, `games-${count}.json`);
  fs.writeFileSync(filePath, JSON.stringify(data, null, 2), 'utf-8');
  console.log(`Generado: ${filePath} (${data.length} juegos)`);
});
console.log("Generación de datasets completada.");

import type { Emulator, Game } from '@contracts/game.types';

export function emulatorBlockReason(emulator?: Emulator | null): string | null {
  if (!emulator) return 'No hay un emulador instalado para esta plataforma';
  if (emulator.compatibility?.status === 'supported') return null;
  if (emulator.compatibility) return emulator.compatibility.reason || 'Emulador no disponible';
  return 'Compatibilidad pendiente de comprobar';
}

export function gameBlockReason(game: Game, emulators: Emulator[]): string | null {
  const candidates = emulators.filter(emulator => emulator.supportedPlatforms.includes(game.platform));
  if (candidates.some(emulator => emulatorBlockReason(emulator) === null)) return null;
  return emulatorBlockReason(candidates[0]);
}
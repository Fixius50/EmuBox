import type { Game, Emulator, CompatibilityAssociation, ExecutionTarget } from '@contracts/game.types';
import type { IEmuBoxBackend } from '@contracts/backend.types';

export class CompatibilityService {
  constructor(private backend: IEmuBoxBackend) {}

  /**
   * Resuelve el destino exacto de ejecución (emulador, binario, argumentos y configs)
   * consultando la jerarquía persistida en SQLite:
   *   1. Asociación explícita del juego (Game -> Emulator) con mayor prioridad o isDefault
   *   2. Emulador predeterminado de la plataforma (System -> Default Emulator)
   *   3. Primer emulador compatible activo
   */
  public async resolveExecutionTarget(game: Game, preferredEmulatorId?: string): Promise<ExecutionTarget> {
    const availableEmulators = await this.getCompatibleEmulatorsForGame(game);
    
    if (availableEmulators.length === 0) {
      throw new Error(`No hay ningún emulador compatible disponible para el sistema ${game.platformName} (${game.platform}).`);
    }

    let targetEmulator: Emulator | undefined;
    let customArgs: string[] = [];
    let customConfigPath: string | undefined;

    // 1. Si se especificó un emulador preferido explícito en tiempo de llamada
    if (preferredEmulatorId) {
      targetEmulator = availableEmulators.find(e => e.id === preferredEmulatorId);
    }

    // 2. Si no, consultar asociaciones persistidas en SQLite para este juego
    if (!targetEmulator) {
      const gameAssocs = await this.getAssociationsForGame(game.id);
      const enabledAssocs = gameAssocs.filter(a => a.enabled);

      const defaultAssoc = enabledAssocs.find(a => a.isDefault) || enabledAssocs[0];
      if (defaultAssoc) {
        targetEmulator = availableEmulators.find(e => e.id === defaultAssoc.emulatorId);
        customArgs = defaultAssoc.customArgs || [];
        customConfigPath = defaultAssoc.customConfigPath;
      }
    }

    // 3. Si no, usar el emulador predeterminado de la plataforma
    if (!targetEmulator) {
      const platforms = await this.backend.getPlatforms();
      const platform = platforms.find(p => p.id === game.platform);
      if (platform && platform.defaultEmulatorId) {
        targetEmulator = availableEmulators.find(e => e.id === platform.defaultEmulatorId);
      }
    }

    // 4. Fallback al primer emulador activo
    if (!targetEmulator) {
      targetEmulator = availableEmulators.find(e => e.status === 'active') || availableEmulators[0];
    }

    const command = targetEmulator.executable;
    const finalArgs = [...targetEmulator.arguments, ...customArgs];
    if (game.romPath) {
      finalArgs.push(game.romPath);
    }

    return {
      game,
      emulator: targetEmulator,
      command,
      args: finalArgs,
      configPath: customConfigPath
    };
  }

  /**
   * Obtiene todos los emuladores compatibles para un juego específico
   */
  public async getCompatibleEmulatorsForGame(game: Game): Promise<Emulator[]> {
    const all = await this.backend.getEmulators();
    return all.filter(e => e.supportedPlatforms.includes(game.platform));
  }

  /**
   * Obtiene todos los juegos compatibles asociados a un emulador
   */
  public async getGamesForEmulator(emulatorId: string): Promise<Game[]> {
    const emulator = await this.backend.getEmulator(emulatorId);
    if (!emulator) return [];

    const allGames = await this.backend.getGames();
    return allGames.filter(g => emulator.supportedPlatforms.includes(g.platform));
  }

  /**
   * Registra o actualiza la asociación entre un juego y un emulador en SQLite
   */
  public async setAssociation(association: CompatibilityAssociation): Promise<void> {
    return this.backend.setGameAssociation(association);
  }

  /**
   * Elimina una asociación entre un juego y un emulador de SQLite
   */
  public async removeAssociation(gameId: string, emulatorId: string): Promise<void> {
    return this.backend.removeGameAssociation(gameId, emulatorId);
  }

  /**
   * Obtiene las asociaciones configuradas en SQLite para un juego
   */
  public async getAssociationsForGame(gameId: string): Promise<CompatibilityAssociation[]> {
    return this.backend.getGameAssociations(gameId);
  }
}

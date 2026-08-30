import type { Game, Emulator, PlatformId, CompatibilityAssociation, ExecutionTarget } from '@contracts/game.types';
import type { IEmuBoxBackend } from '@contracts/backend.types';

export class CompatibilityService {
  private associations: Map<string, CompatibilityAssociation[]> = new Map();

  constructor(private backend: IEmuBoxBackend) {}

  /**
   * Resuelve el destino exacto de ejecución (emulador, binario, argumentos y configs)
   * consultando la jerarquía:
   *   1. Asociación explícita del juego (Game -> Emulator)
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

    // 1. Si se especificó un emulador preferido explícito
    if (preferredEmulatorId) {
      targetEmulator = availableEmulators.find(e => e.id === preferredEmulatorId);
    }

    // 2. Si no, buscar asociación guardada para este juego
    if (!targetEmulator) {
      const gameAssocs = this.associations.get(game.id) || [];
      const defaultAssoc = gameAssocs.find(a => a.enabled && a.isDefault) || gameAssocs.find(a => a.enabled);
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
      targetEmulator = availableEmulators[0];
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
   * Registra o actualiza la asociación entre un juego y un emulador
   */
  public setAssociation(association: CompatibilityAssociation): void {
    const list = this.associations.get(association.gameId) || [];
    const filtered = list.filter(a => a.emulatorId !== association.emulatorId);
    if (association.isDefault) {
      filtered.forEach(a => a.isDefault = false);
    }
    filtered.push(association);
    this.associations.set(association.gameId, filtered);
  }

  /**
   * Obtiene las asociaciones configuradas para un juego
   */
  public getAssociationsForGame(gameId: string): CompatibilityAssociation[] {
    return this.associations.get(gameId) || [];
  }
}

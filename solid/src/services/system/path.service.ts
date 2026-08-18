/**
 * PathService: Centralizes all XDG directory resolution and path generation.
 * Guarantees that no UI component ever hardcodes paths.
 */
export class PathService {
  private baseConfigDir: string;
  private baseDataDir: string;
  private baseCacheDir: string;

  constructor(customConfig?: string, customData?: string, customCache?: string) {
    this.baseConfigDir = customConfig || '~/.config/emubox';
    this.baseDataDir = customData || '~/.local/share/emubox';
    this.baseCacheDir = customCache || '~/.cache/emubox';
  }

  public getConfigDir(): string {
    return this.baseConfigDir;
  }

  public getConfigFile(): string {
    return `${this.baseConfigDir}/config.json`;
  }

  public getEmulatorsConfigFile(): string {
    return `${this.baseConfigDir}/emulators.json`;
  }

  public getControllersConfigFile(): string {
    return `${this.baseConfigDir}/controllers.json`;
  }

  public getDataDir(): string {
    return this.baseDataDir;
  }

  public getRomsDir(platform?: string): string {
    return platform ? `${this.baseDataDir}/roms/${platform}` : `${this.baseDataDir}/roms`;
  }

  public getSavesDir(platform?: string): string {
    return platform ? `${this.baseDataDir}/saves/${platform}` : `${this.baseDataDir}/saves`;
  }

  public getStatesDir(platform?: string): string {
    return platform ? `${this.baseDataDir}/states/${platform}` : `${this.baseDataDir}/states`;
  }

  public getScreenshotsDir(): string {
    return `${this.baseDataDir}/screenshots`;
  }

  public getCoversDir(): string {
    return `${this.baseDataDir}/covers`;
  }

  public getBiosDir(): string {
    return `${this.baseDataDir}/bios`;
  }

  public getLogsDir(): string {
    return `${this.baseDataDir}/logs`;
  }

  public getCacheDir(): string {
    return this.baseCacheDir;
  }
}

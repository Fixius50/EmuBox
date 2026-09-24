import type {
  Emulator,
  Game,
  Platform,
  PlatformId,
  SystemSettings,
} from "@contracts/game.types";
import type {
  DownloadJob,
  DownloadSourceOption,
} from "@contracts/download.types";
import type { StoreAccount, StoreProviderInfo } from "@contracts/store.types";
import type { AudioInfo, DisplayInfo } from "@contracts/system.types";
import { TauriBackendService } from "@services/backend/tauri-backend.service";

function exampleGame(
  id: string,
  title: string,
  platform: PlatformId,
  installed = false,
): Game {
  return {
    id: `preview-${id}`,
    title,
    platform,
    platformName: platform.toUpperCase(),
    releaseYear: 2000,
    genre: "Aventura",
    developer: "Estudio de ejemplo",
    publisher: "Estudio de ejemplo",
    rating: 4,
    playTimeMinutes: 0,
    favorite: false,
    coverImage: "",
    installed,
    description:
      "Titulo ficticio de previsualizacion. No contiene archivos de juego.",
  };
}

export class DevelopmentBackendService extends TauriBackendService {
  private games: Game[] = [
    exampleGame("orbita-snes", "Orbita Cero", "snes", true),
    exampleGame("orbita-ps1", "Orbita Cero", "ps1"),
    exampleGame("cristal", "Bosque de Cristal", "gba", true),
    exampleGame("horizonte", "Horizonte Rojo", "ps2"),
    exampleGame("circuito", "Circuito Lunar", "snes"),
  ];
  private emulators: Emulator[] = [
    {
      id: "preview-emulator",
      name: "Motor de ejemplo",
      version: "preview",
      supportedPlatforms: ["snes", "gba"],
      coreType: "libretro",
      status: "inactive",
      executable: "",
      arguments: [],
    },
  ];
  private settings: SystemSettings = {
    display: {
      vsync: true,
      crtShader: "none",
    },
    audio: {
      masterVolume: 80,
      uiSoundEffects: false,
      backgroundMusic: false,
      audioLatencyMs: 50,
    },
    gamepad: { deadzone: 0.15, vibration: false, swapSouthEastButtons: false },
    library: {
      datasetLimit: 0,
      showMissingCovers: true,
      defaultPlatform: "all",
    },
    system: { showFps: false, seedCompletedTorrents: false },
    updates: { autoUpdate: false, channel: "stable", checkOnStartup: false },
  };

  public override get isTauriEnvironment(): boolean {
    return false;
  }
  public override async getGames(): Promise<Game[]> {
    return structuredClone(this.games);
  }
  public override async getPlatforms(): Promise<Platform[]> {
    return [];
  }
  public override async getSettings(): Promise<SystemSettings> {
    return structuredClone(this.settings);
  }
  public override async getAudioInfo(): Promise<AudioInfo> {
    const media =
      typeof navigator === "undefined" ? undefined : navigator.mediaDevices;
    if (!media?.enumerateDevices)
      throw new Error(
        "La deteccion de audio no esta disponible en este navegador.",
      );
    const devices = (await media.enumerateDevices()).filter(
      (device) => device.kind === "audioinput" || device.kind === "audiooutput",
    );
    return {
      masterVolume: null,
      uiSoundEffects: this.settings.audio.uiSoundEffects,
      backgroundMusic: this.settings.audio.backgroundMusic,
      latencyMs: null,
      sampleRate: null,
      deviceNamesLimited:
        devices.length === 0 ||
        devices.some((device) => !device.label || !device.deviceId),
      devices: devices
        .filter((device) => device.deviceId && device.deviceId !== "default")
        .map((device, index) => ({
          id: device.deviceId,
          name:
            device.label ||
            `${device.kind === "audioinput" ? "Entrada" : "Salida"} ${index + 1}`,
          isDefault: false,
          type: device.kind === "audioinput" ? "source" : "sink",
        })),
    };
  }
  public override async getDisplayInfo(): Promise<DisplayInfo> {
    const width = typeof window === 'undefined' ? 1920 : window.innerWidth;
    const height = typeof window === 'undefined' ? 1080 : window.innerHeight;
    return {
      resolution: `${width}x${height}`,
      width,
      height,
      refreshRate: 60,
      devicePixelRatio: typeof window === 'undefined' ? 1 : window.devicePixelRatio,
      colorDepth: typeof screen === 'undefined' ? null : screen.colorDepth,
      hdrSupported: null,
      activeCompositor: 'browser',
      gamescopeActive: false,
    };
  }
  public override async getEmulators(): Promise<Emulator[]> {
    return structuredClone(this.emulators);
  }
  public override async getDownloadJobs(): Promise<DownloadJob[]> {
    return [];
  }
  public override async getDownloadSources(): Promise<DownloadSourceOption[]> {
    return [];
  }
  public override async getStoreProviders(): Promise<StoreProviderInfo[]> {
    return [];
  }
  public override async getStoreAccounts(): Promise<StoreAccount[]> {
    return [];
  }

  public override async saveSettings(
    settings: SystemSettings,
  ): Promise<boolean> {
    this.settings = structuredClone(settings);
    return true;
  }

  public override async saveEmulator(emulator: Emulator): Promise<void> {
    const saved = structuredClone(emulator);
    this.emulators = this.emulators.some((entry) => entry.id === saved.id)
      ? this.emulators.map((entry) => (entry.id === saved.id ? saved : entry))
      : [...this.emulators, saved];
  }

  public override async deleteEmulator(id: string): Promise<void> {
    this.emulators = this.emulators.filter((entry) => entry.id !== id);
  }

  public override async toggleFavorite(id: string): Promise<boolean> {
    const game = this.games.find((entry) => entry.id === id);
    if (!game) throw new Error("Juego de ejemplo no encontrado");
    game.favorite = !game.favorite;
    return game.favorite;
  }
}

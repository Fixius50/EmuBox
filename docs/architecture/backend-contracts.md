# EmuBox Backend Architecture & Contracts Specification

## 1. Overview & Principles

EmuBox is engineered with a strict, decoupled boundary between the **UI Presentation Layer** (SolidJS + Kobalte + CSS + Anime.js) and the **Execution Backend Layer**.

```text
┌───────────────────────────────────────────────┐
│                  EMUBOX UI                    │
│                                               │
│ SolidJS + Kobalte + CSS + Anime.js            │
└───────────────────────┬───────────────────────┘
                        │
                  IEmuBoxBackend
                        │
             ┌──────────┴──────────┐
             │                     │
        MockBackend          TauriBackend
             │                     │
             │                    IPC
             │                     │
             │               ┌─────┴─────┐
             │               │ Rust/Tauri│
             │               └─────┬─────┘
             │                     │
             │       ┌─────────────┼─────────────┐
             │       │             │             │
             │    Filesystem    Processes     Hardware
             │       │             │             │
             │      ROMs       Emuladores      Gamepad
             │
        Desarrollo
```

### Core Tenet
> **The UI must remain 100% agnostic to whether it is running against the in-memory MockBackend or real Linux system services via Tauri IPC.**

---

## 2. Master TypeScript Contract (`IEmuBoxBackend`)

```typescript
export interface IEmuBoxBackend {
  // System & Environment
  getSystemInfo(): Promise<SystemInfo>;
  runFirstRunDetection(): Promise<FirstRunDetectionResult>;

  // Central Versioned Configuration (XDG Base Directory)
  getConfig(): Promise<EmuBoxConfig>;
  saveConfig(config: EmuBoxConfig): Promise<void>;

  // Legacy/Runtime Quick Settings
  getSettings(): Promise<SystemSettings>;
  saveSettings(settings: SystemSettings): Promise<boolean>;

  // Games & Library
  scanGames(request?: ScanGamesRequest): Promise<ScanGamesResult>;
  getGames(filter?: GameFilter): Promise<Game[]>;
  getGameById(id: string): Promise<Game | null>;
  toggleFavorite(gameId: string): Promise<boolean>;

  // Platforms & Consoles
  getPlatforms(): Promise<Platform[]>;

  // Emulators & Libretro Cores (CRUD)
  getEmulators(): Promise<Emulator[]>;
  saveEmulator(emulator: Emulator): Promise<void>;
  deleteEmulator(id: string): Promise<void>;

  // Game Execution & Lifecycle
  launchGame(gameIdOrRequest: string | LaunchGameRequest, emulatorId?: string): Promise<LaunchResult>;
  stopGame(): Promise<void>;

  // Gamepad & Input
  getGamepadStatus(): Promise<GamepadStatus>;
}
```

---

## 3. Data Models

### `SystemInfo`
Telemetry representing hardware capabilities, display composition, and Linux kernel environment:
* `osName`: Operating system distribution and kernel string.
* `kernelVersion`: Active Linux kernel release.
* `architecture`: System architecture (`x86_64`).
* `gpuRenderer`: WebGL/RADV/NVIDIA graphics driver string.
* `cpuModel`: Processor identification string.
* `cpuCores`: Number of logical cores.
* `totalMemoryMb` / `usedMemoryMb`: RAM allocation.
* `gamescopeAvailable`: Boolean indicating whether Gamescope compositor is active.
* `activeCompositor`: Compositor identifier (`gamescope-wayland`, `wayland`, `x11`).

### `EmuBoxConfig`
Single, central, versioned JSON configuration model:
* `version`: Schema version number.
* `paths`: Standard XDG paths for `roms`, `saves`, `states`, `screenshots`, `covers`, `logs`.
* `display`: `resolution`, `refreshRate`, `fullscreen`, `vsync`, `gamescopeEnabled`, `gamescopeScaling`, `crtShader`.
* `audio`: `volume`, `uiSoundEffects`, `backgroundMusic`, `latencyMs`.
* `input`: `deadzone`, `vibrationEnabled`, `swapSouthEastButtons`, `pollRateHz`.
* `emulators`: `defaultMapping`, `customBinariesPath`.
* `interface`: `locale`, `theme`, `animations`, `showFpsOverlay`, `performanceMode`.

### `Emulator`
Abstracted execution profile for engines:
* `id`: Unique identifier (e.g., `duckstation`, `snes9x`).
* `name`: Human-readable engine title.
* `version`: Installed or bundled version string.
* `supportedPlatforms`: Target consoles (`['ps1']`, `['snes']`, etc.).
* `coreType`: `'libretro' | 'standalone'`.
* `status`: Dynamic availability `'active' | 'inactive' | 'missing_bios'`.
* `executable`: Executable binary name in system `$PATH` (e.g. `retroarch`, `duckstation-qt`).
* `arguments`: Command-line flag array (e.g. `['-L', 'snes9x_libretro.so']`).

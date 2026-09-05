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

  // Central Versioned Configuration (system appliance config)
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
* `architecture`: Runtime architecture: `x86_64`, `aarch64`, or `unsupported`.
* `kernelArchitecture`: Kernel-reported `uname -m`, separate from runtime architecture.
* `hardware`: `cpuArchitecture`, `cpuModel`, `cpuCores`, `totalMemoryMb`, `freeMemoryMb`,
  `gpuVendor`, `gpuRenderer`, `vulkanDriverVersion`, `vulkanSupported`, `drmAvailable`,
  `gamescopeAvailable`, `recommendedCompositor`, `deviceModel`.
* `hardware.openglSupported`, `hardware.openglRenderer`, `hardware.openglAccelerated`
     report EGL/OpenGL capabilities separately from Vulkan. These describe detected
     availability, not a guarantee that every application uses that renderer.
* `gamescopeAvailable` describes executable availability, not an active session.
* `display.activeCompositor` and `display.gamescopeActive` describe display state.

Installer support is limited to Arch Linux x86_64 and Arch Linux ARM aarch64.
ARM32 is unsupported. CPU support does not imply GPU or emulator availability.
Mocks accept an architecture constructor option;
browser code never infers host architecture from `navigator`.

### `EmuBoxConfig`
Single, central, versioned JSON configuration model:
* `version`: Schema version number.
* `paths`: Canonical appliance paths for `roms`, `saves`, `states`, `screenshots`, `covers`, `logs` under `/etc/emubox` and the system data/cache directories.
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
* `architectures`: Native CPU allowlist from `data/emulator-capabilities.json`.
* `requirements`: `minCpuCores`, `minMemoryMb`, `vulkan`.
* `compatibility`: `status`, `reason`, `hostArchitecture`, `binaryArchitecture`.

Statuses: `supported`, `unsupported_architecture`, `not_installed`,
`invalid_binary`, `requirements_not_met`. Script wrappers validate their native
interpreter; their internal command chains cannot be proven by inspecting a shebang.

Compatibility is recomputed by Rust, including executable resolution and native
libretro core checks. The UI uses this result to disable launch with a reason;
catalog IDs, platforms, downloads and game storage remain architecture-independent.
RPCS3 remains visible even when unavailable. Matrix support is not evidence that
a compatible binary is installed or that a particular game will perform well.

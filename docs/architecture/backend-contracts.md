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
                   TauriBackend
                        │
                       IPC
                        │
                   Rust/Tauri
                        │
           Filesystem / Processes / Hardware
```

### Core Tenet
> **The UI delegates to native services through IPC. Missing runtime, unavailable capability and real failures must remain explicit; no fabricated success or hardware.**

The Rust API surface is split deliberately:

* `src-tauri/src/api/invoke.rs` is the single Tauri command registry.
* `src-tauri/src/api/events.rs` is the single registry for event names emitted by Rust.
* `src-tauri/src/commands/` contains thin IPC adapters grouped by domain.
* `src-tauri/src/services/` owns filesystem, processes, network, hardware and persistence behavior.

`npm run arch:check` validates that Rust commands declared with `#[tauri::command]`
are registered in the Tauri handler and that frontend `TauriBackend` invokes only
registered commands.

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
* `graphicsAccelerated`, `graphicsBackend` (`opengl`, `vulkan`, `software`, `auto`),
     `gpuKind` (`physical`, `virtual`, `software`, `unknown`) and `isVirtualMachine` separate
     rendering capability from CPU architecture and compositor selection.
* `gamescopeReady` reports its own Vulkan/DRM/executable prerequisites, not GPU
     availability. `recommendedCompositor` can be `cage`, `gamescope` or `unavailable`.
     OpenGL + Cage is preferred automatically; Gamescope requires a compatible
     preference or an unavailable Cage route. These are probe results, not runtime certification.
* GPU vendor `mali` denotes the GPU family; CPU `aarch64` does not imply a GPU vendor.
* `display.activeCompositor` and `display.gamescopeActive` describe display state.

`hardware.graphics` is the authoritative evidence snapshot. `detectionState` is
`accelerated`, `software` or `indeterminate`; a legacy boolean false only means
acceleration was not confirmed. `devices` groups DRM card/render nodes by sysfs
identity, including PCI address and render major/minor identifiers. `probes`
contains API, state, reason, exit code and observations with nullable `deviceId`.
Correlation is `device_identifier`, `single_device_inference` or `unknown`.
Renderer names are not cross-device identifiers. An unresolved explicit identifier
never falls back to single-device inference.

`selectedDeviceId` is nullable when selection cannot be correlated uniquely;
`activeDeviceId` is null until actual process evidence exists. `backend` is the
recommended API; `operationalBackend` records explicit software fallback without
changing the detection state. `fallbackReason` records that choice. `inventoryComplete`
and `inventoryReason` distinguish failed inventory from an empty inventory.
Indeterminate detection uses automatic compositor initialization, not forced CPU.
The launcher consumes this same service through `--graphics-session`, with
`--graphics-info` for structured read-only diagnostics.

Installer support is limited to Arch Linux x86_64 and Arch Linux ARM aarch64.
ARM32 is unsupported. CPU support does not imply GPU or emulator availability.
Browser code never infers host architecture from `navigator`.

Telemetry that cannot be queried is nullable: `SystemInfo.display` and `audio`,
unmeasured display/audio fields, gamepad capability counts and primary index.
Log timestamps can be absent for unstructured session logs; log source is a
native identifier, and unavailable recent journal errors are `null`, not an
invented empty history. OTA methods explicitly reject unsupported operations.

### `EmuBoxConfig`
Single, central, versioned JSON configuration model:
* `version`: Schema version number.
* `paths`: Canonical appliance paths for `roms`, `saves`, `states`, `screenshots`, `covers`, `logs` under `/etc/emubox` and the system data/cache directories.
* `display`: preferencias persistidas `vsync` y `crtShader`. Resolucion, frecuencia, fullscreen y seleccion de compositor son evidencia adaptativa de la sesion mediante `get_display_info`, no configuracion persistida.
* `audio`: `volume`, `uiSoundEffects`, `backgroundMusic`, `latencyMs`.
* `input`: `deadzone`, `vibrationEnabled`, `swapSouthEastButtons`, `pollRateHz`.
* `emulators`: `defaultMapping`, `customBinariesPath`.
* `interface`: `locale`, `theme`, `animations`, `showFpsOverlay`. El paralelismo de arranque se decide una vez segun los recursos disponibles al iniciar.

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

Los perfiles que escriben renderer nativo reciben la misma seleccion
`operationalBackend` que usa EmuBox: OpenGL se propaga como OpenGL/GL y Vulkan
como Vulkan. Los estados `software` y `auto` son conservadores: conservan el
renderer del emulador en vez de forzar Vulkan por disponibilidad aislada.
Al lanzar, Bubblewrap monta solo lectura el archivo gestionado del perfil bajo
su destino exacto en el user directory privado del emulador: `$HOME/.config`
para RetroArch, PCSX2, DuckStation y PPSSPP; Dolphin usa su user directory
explícito bajo `$HOME/.local/share/dolphin-emu` mediante `-u`. La tabla de perfiles
es cerrada; se valida que el archivo sea regular, canónico y permanezca dentro de
`/var/lib/emubox/emulators/<id>/config`. No se monta la configuración general del
host ni configuraciones de otros emuladores.

## Confined Launch

`launch_game` runs blocking validation off the IPC thread and always enters the
native Bubblewrap policy. Stored executable/argument overrides are not launch
authority; only compiled emulator profiles and protected system tools are accepted.
Legacy `romPath`, nonempty `customArgs` and custom association configs are rejected.
`execute_command` is removed. `kill_process` can stop only the active sandbox Child.
See [execution profiles and limitations](execution-sandbox.md).

## Explicit Source Search

`search_jackett(gameId)` queries only the managed local instance and returns
bounded candidates without API keys or private download links. `select_jackett_result`
accepts a cached result ID belonging to that version and adds a source, not a job.
The frontend must still explicitly request `download_game`. qBittorrent replaces
aria2 behind the persisted `bittorrent` provider ID; no source/job migration is needed.

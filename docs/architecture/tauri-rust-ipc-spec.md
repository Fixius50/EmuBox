# Tauri 2.0 Rust IPC Specification for EmuBox

## 1. Overview

This document describes the **Rust / Tauri 2.0** IPC boundary implementing `IEmuBoxBackend`.

## Native architecture contract

Supported runtime CPUs are x86_64 and aarch64, not ARM32. `get_system_info`
returns `architecture` (`x86_64`, `aarch64`, `unsupported`) and separate
`kernelArchitecture`. Nested `hardware` reports CPU, RAM, GPU vendor/renderer,
Vulkan, DRM, Gamescope availability, recommended compositor and device model.
`hardware.openglSupported`, `openglRenderer` and `openglAccelerated` also report
EGL/OpenGL availability, including accelerated virtual GPUs without Vulkan.
`graphicsAccelerated`, `graphicsBackend`, `gpuKind`, `isVirtualMachine` and
`gamescopeReady` distinguish acceleration, selected API and compositor requirements.
Automatic selection prefers accelerated OpenGL with Cage; Gamescope is chosen only
when its own requirements and preference/alternative route permit it. Vulkan is not
an EmuBox requirement. `recommendedCompositor` may be `unavailable`.

`get_emulators` returns `architectures`, `requirements` and `compatibility`
alongside the existing profile. Compatibility includes `status`, `reason`,
`hostArchitecture` and optional `binaryArchitecture`. Launch revalidates these
capabilities and the executable/core before spawning; frontend checks alone are
not authoritative. No architecture is added to Game IDs, ROM paths or manifests.

The native ARM CI runner is configured, but ARM build and graphical runtime
results remain pending. An ARM64 runtime does not guarantee RPCS3 availability.

---

## 2. Command Index

| Command | Arguments | Return Type | Description |
| :--- | :--- | :--- | :--- |
| `get_system_info` | *None* | `SystemInfo` | Queries kernel, GPU via EGL/OpenGL/Vulkan/DRM, and independent compositor status |
| `first_run_detection` | *None* | `FirstRunDetectionResult` | Probes for GPU vendor, connected controllers, and installed engine binaries |
| `get_config` | *None* | `EmuBoxConfig` | Reads and parses `/etc/emubox/config.json` |
| `save_config` | `{ config: EmuBoxConfig }` | `void` | Writes atomized JSON to `/etc/emubox/config.json` |
| `get_settings` | *None* | `SystemSettings` | Quick settings compatibility accessor |
| `save_settings` | `{ settings: SystemSettings }` | `boolean` | Quick settings persistence |
| `scan_games` | `{ request?: ScanGamesRequest }` | `ScanGamesResult` | Traverses `/var/lib/emubox/games/*` and generates metadata |
| `get_games` | `{ filter?: GameFilter }` | `Vec<Game>` | Returns filtered and indexed game catalog |
| `get_game_by_id` | `{ id: String }` | `Option<Game>` | Returns specific game record |
| `toggle_favorite` | `{ gameId: String }` | `boolean` | Atomically updates favorite flag |
| `get_platforms` | *None* | `Vec<Platform>` | Returns system platforms |
| `get_emulators` | *None* | `Vec<Emulator>` | Reads SQLite and verifies binary existence |
| `save_emulator` | `{ emulator: Emulator }` | `void` | Adds or updates emulator configuration |
| `delete_emulator` | `{ id: String }` | `void` | Removes emulator profile |
| `launch_game` | `{ request: LaunchGameRequest }` | `LaunchResult` | Spawns emulator directly; Gamescope wrapper only when explicitly requested and supported |
| `stop_game` | *None* | `void` | Sends graceful `SIGTERM` (followed by `SIGKILL` if needed) to active PID |
| `get_gamepad_status` | *None* | `GamepadStatus` | Queries `libevdev` / `gilrs` for connected physical gamepads |

---

## 3. Rust Process & Compositor Execution Flow

When `launch_game` is invoked:
1. Rust reads the emulator profile from configuration.
2. Resolves ROM absolute path inside `/var/lib/emubox/games/<platform>`.
3. Starts `<executable> <arguments> <romPath>` directly. Only an explicit
   `useGamescope: true` plus detected Gamescope prerequisites enables the
   `gamescope -f --` wrapper; it is not selected just because Vulkan is available.
4. Stores `Child` PID in thread-safe state (`Arc<Mutex<Option<Child>>>`).
5. Emits `game-started` event via Tauri Event API.
6. Spawns background monitor thread waiting on process exit, emitting `game-terminated` when complete.

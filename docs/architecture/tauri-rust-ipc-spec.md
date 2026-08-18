# Tauri 2.0 Rust IPC Specification for EmuBox

## 1. Overview

This document specifies the exact IPC commands and signatures that the upcoming **Rust / Tauri 2.0** core must implement to fulfill `IEmuBoxBackend`.

---

## 2. Command Index

| Command | Arguments | Return Type | Description |
| :--- | :--- | :--- | :--- |
| `get_system_info` | *None* | `SystemInfo` | Queries kernel, GPU adapter via Vulkan/DRM, and Gamescope compositor status |
| `first_run_detection` | *None* | `FirstRunDetectionResult` | Probes for GPU vendor, connected controllers, and installed engine binaries |
| `get_config` | *None* | `EmuBoxConfig` | Reads and parses `~/.config/emubox/config.json` |
| `save_config` | `{ config: EmuBoxConfig }` | `void` | Writes atomized JSON to `~/.config/emubox/config.json` |
| `get_settings` | *None* | `SystemSettings` | Quick settings compatibility accessor |
| `save_settings` | `{ settings: SystemSettings }` | `boolean` | Quick settings persistence |
| `scan_games` | `{ request?: ScanGamesRequest }` | `ScanGamesResult` | Traverses `~/.local/share/emubox/roms/*` and generates metadata |
| `get_games` | `{ filter?: GameFilter }` | `Vec<Game>` | Returns filtered and indexed game catalog |
| `get_game_by_id` | `{ id: String }` | `Option<Game>` | Returns specific game record |
| `toggle_favorite` | `{ gameId: String }` | `boolean` | Atomically updates favorite flag |
| `get_platforms` | *None* | `Vec<Platform>` | Returns system platforms |
| `get_emulators` | *None* | `Vec<Emulator>` | Reads `~/.config/emubox/emulators.json` and verifies binary existence |
| `save_emulator` | `{ emulator: Emulator }` | `void` | Adds or updates emulator configuration |
| `delete_emulator` | `{ id: String }` | `void` | Removes emulator profile |
| `launch_game` | `{ request: LaunchGameRequest }` | `LaunchResult` | Spawns emulator subprocess inside Gamescope session and tracks PID |
| `stop_game` | *None* | `void` | Sends graceful `SIGTERM` (followed by `SIGKILL` if needed) to active PID |
| `get_gamepad_status` | *None* | `GamepadStatus` | Queries `libevdev` / `gilrs` for connected physical gamepads |

---

## 3. Rust Process & Compositor Execution Flow

When `launch_game` is invoked:
1. Rust reads the emulator profile from configuration.
2. Resolves ROM absolute path inside `~/.local/share/emubox/roms/<platform>/`.
3. If Gamescope is enabled:
   ```bash
   gamescope -W 1920 -H 1080 -f -r <refreshRate> -- <executable> <arguments> <romPath>
   ```
4. Stores `Child` PID in thread-safe state (`Arc<Mutex<Option<Child>>>`).
5. Emits `game-started` event via Tauri Event API.
6. Spawns background monitor thread waiting on process exit, emitting `game-terminated` when complete.

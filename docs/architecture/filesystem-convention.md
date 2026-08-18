# EmuBox Filesystem Hierarchy & Conventions

## 1. XDG Base Directory Compliance

EmuBox strictly complies with the **XDG Base Directory Specification** on Arch Linux.
No user-specific paths or hardcoded developer folders are used.

```text
Sistema
│
├── Binarios y Assets del Sistema
│   └── /opt/emubox/
│       ├── bin/emubox (Tauri Runtime)
│       └── assets/
│
├── Configuración del Usuario ($XDG_CONFIG_HOME)
│   └── ~/.config/emubox/
│       ├── config.json (Master EmuBox Configuration)
│       ├── emulators.json (Discovered Engine Profiles)
│       └── controllers.json (Gamepad Mappings)
│
├── Datos del Usuario ($XDG_DATA_HOME)
│   └── ~/.local/share/emubox/
│       ├── roms/
│       │   ├── snes/
│       │   ├── ps1/
│       │   ├── ps2/
│       │   ├── n64/
│       │   ├── genesis/
│       │   ├── gba/
│       │   ├── dreamcast/
│       │   └── arcade/
│       │
│       ├── saves/          (Battery RAM & In-game Saves)
│       ├── states/         (Realtime Emulator Save States)
│       ├── screenshots/    (User Gameplay Screen Captures)
│       ├── covers/         (Box Art & Fanart Cached Thumbnails)
│       └── logs/           (Session & Process Launch Logs)
│
└── Caché ($XDG_CACHE_HOME)
    └── ~/.cache/emubox/
        └── vram_shaders/   (Pre-compiled Vulkan Pipeline Caches)
```

---

## 2. Directory Resolution Hierarchy

1. `$XDG_CONFIG_HOME` -> Defaults to `$HOME/.config/emubox`
2. `$XDG_DATA_HOME` -> Defaults to `$HOME/.local/share/emubox`
3. `$XDG_CACHE_HOME` -> Defaults to `$HOME/.cache/emubox`

All directories are created with permissions `0755` by `installer/setup/directories.sh`.

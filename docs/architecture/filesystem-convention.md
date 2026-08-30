# EmuBox Filesystem Hierarchy & Conventions

## 1. XDG Base Directory Compliance

EmuBox strictly complies with the **XDG Base Directory Specification** on Arch Linux.
No user-specific paths or hardcoded developer folders are used.

```text
Sistema
│
├── Binarios y Runtime
│   └── /opt/emubox/
│       ├── bin/emubox (Tauri Runtime)
│       └── assets/
│
├── Almacenamiento Persistente Canónico (/var/lib/emubox/)
│   ├── emubox.db (Base de datos SQLite: games, emulators, systems, associations)
│   ├── games/
│   │   ├── snes/
│   │   ├── ps1/
│   │   ├── ps2/
│   │   ├── n64/
│   │   ├── genesis/
│   │   ├── gba/
│   │   ├── dreamcast/
│   │   └── arcade/
│   ├── emulators/      (Entornos aislados de emuladores instalados)
│   ├── bios/           (Archivos de BIOS oficiales requeridas)
│   ├── saves/          (Saves de batería / memorias)
│   ├── states/         (Save states en tiempo real)
│   ├── covers/         (Carátulas y fanart en caché)
│   └── screenshots/    (Capturas de pantalla del usuario)
│
├── Configuración del Usuario ($XDG_CONFIG_HOME)
│   └── ~/.config/emubox/
│       ├── config.json (Master EmuBox Configuration)
│       └── controllers.json (Gamepad Mappings)
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

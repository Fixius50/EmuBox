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


## 2. Directory Resolution Hierarchy

1. `$XDG_CONFIG_HOME` -> Defaults to `$HOME/.config/emubox`
2. `$XDG_DATA_HOME` -> Defaults to `$HOME/.local/share/emubox`
3. `$XDG_CACHE_HOME` -> Defaults to `$HOME/.cache/emubox`

All directories are created with permissions `0755` by `installer/setup/directories.sh`.

# EmuBox Filesystem Hierarchy

EmuBox is a dedicated Arch Linux appliance. Runtime software, system
configuration, persistent data, regenerable cache, logs and transient runtime
state use separate canonical locations.

```text
/etc/emubox/                         System configuration
├── config.json
├── emulators.json
└── platforms.json

/opt/emubox/                         EmuBox software and runtime
└── bin/emubox

/var/lib/emubox/                     Persistent EmuBox data
├── emubox.db
├── games/<platform>/
├── emulators/<id>/{bin,config,logs}/
├── bios/<platform>/
├── saves/<platform>/
├── states/<platform>/
└── screenshots/

/var/cache/emubox/                   Regenerable data
├── shaders/
├── metadata/
├── covers/
└── downloads/                        Temporary .part files

/var/log/emubox/                     System/application logs
/run/emubox/                         Transient runtime state
```

## Canonical Rules

- Games are stored in `/var/lib/emubox/games/<platform>/`.
- The optional games partition should be mounted at
    `/var/lib/emubox/games`; EmuBox does not inspect or hardcode a device name.
- Emulators managed by EmuBox live under `/var/lib/emubox/emulators/<id>/`.
    System-installed binaries remain managed by pacman in `/usr/bin` or `/opt`;
    EmuBox may expose them through managed symlinks.
- BIOS, saves, states and screenshots are persistent data under
    `/var/lib/emubox`.
- Covers, shaders, metadata and partial downloads are disposable cache under
    `/var/cache/emubox`.
- Logs belong in `/var/log/emubox`; transient PID/socket/session state belongs
    in `/run/emubox`.
- `/var/lib/emubox/roms` is retained only as a compatibility symlink to
    `/var/lib/emubox/games`.

The migration helper `scripts/migrate-filesystem.sh` copies legacy XDG data
without overwriting canonical files. It is safe to run more than once.

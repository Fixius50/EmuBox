# EmuBox Filesystem Hierarchy & Conventions

## 1. Appliance filesystem model

EmuBox is an Arch Linux appliance and uses a canonical system-wide layout under OS-owned directories. It does not use user-local XDG storage for runtime data, ROMs, hardware state, cache, or logs.

```text
/etc/emubox/
├── config.json
├── emulators.json
├── platforms.json
├── download-links.txt
└── bios-manifest.json

/opt/emubox/
└── bin/emubox

/var/lib/emubox/
├── emubox.db
├── stores/<provider>/
├── stores/<provider>/
├── games/<platform>/
├── emulators/<id>/bin/<architecture>/
├── emulators/<id>/{config,logs}/
├── bios/<platform>/
├── saves/<platform>/
├── states/<platform>/
├── screenshots/
├── sandbox/<hash-game-emulator>/
├── jackett/Jackett/
└── templates/

/var/cache/emubox/
├── shaders/
├── metadata/
├── covers/
├── downloads/
└── temp/

/var/log/emubox/
├── session.log
├── events.jsonl
└── events.jsonl.1 ... events.jsonl.4

/run/emubox/
├── pid/
├── sockets/
└── session/
```

## 2. Canonical rules

- The only supported native architectures are `x86_64` and `aarch64` (not ARM32).
- Build outputs are `/opt/emubox/bin/emubox-linux-x86_64` and
	`/opt/emubox/bin/emubox-linux-aarch64`; `/opt/emubox/bin/emubox` is the validated
	native deployment path. It is not a universal ELF.
- Managed emulator binaries use the architecture subdirectory above. System
	packages still resolve through PATH or absolute paths under `/usr/bin`; native
	libretro `.so` files must match the runtime architecture too.
- Game IDs, manifests, ROMs, BIOS and saves are not partitioned by CPU architecture.
- Build and Git run as `emubox`, not root; only system setup uses elevated privileges.
- Build/update logs use a temporary directory if system logs are not writable.
- ROMs and game files live in `/var/lib/emubox/games/<platform>/`.
- The optional compatibility symlink `/var/lib/emubox/roms` may exist only as a pointer to `/var/lib/emubox/games` and must not be treated as the canonical source of truth.
- Traditional saves/states/screenshots paths remain for existing data. Confined launches use private state under `/var/lib/emubox/sandbox/<hash-game-emulator>`; they do not automatically import old saves or personal configs. BIOS remain under `/var/lib/emubox/bios`.
- Regenerable caches use `/var/cache/emubox/`. Download staging lives under `/var/lib/emubox/games/<platform>/.emubox-staging/<job-id>` so publication can rename on the same filesystem. qBittorrent profiles and payloads are isolated under that job's `qbittorrent/` directory.
- Logs belong in `/var/log/emubox`; transient runtime state belongs in `/run/emubox`.
- System configuration lives in `/etc/emubox` and is versioned by the appliance.
- Jackett is installed separately under root-owned `/opt/jackett`; service state and API key live in `/var/lib/emubox/jackett/Jackett`. It is not part of the game sandbox or personal home.
- Store-provider sessions are private files under `/var/lib/emubox/stores/<provider>/`. SQLite stores only account identity, entitlement, installation state and explicit links to `canonical_games`; it never stores passwords, cookies or access/refresh tokens. Game sandboxes do not receive this directory.
- Store-provider sessions are private files under `/var/lib/emubox/stores/<provider>/`. SQLite stores only account identity, entitlement, installation state and explicit links to `canonical_games`; it never stores passwords, cookies or access/refresh tokens. Game sandboxes do not receive this directory.

## 3. Legacy migration behavior

The migration helper `scripts/migrate-filesystem.sh` is intentionally one-way: it copies legacy user data from older local XDG folders only when present, without overwriting the canonical appliance paths. This is a recovery compatibility mechanism, not the active runtime architecture.

The canonical architecture is the authoritative layout. References to `~/.config/emubox`, `~/.local/share/emubox`, `~/.cache/emubox`, or bare `roms/`/`logs/` paths are migration-only artifacts, not active appliance paths.

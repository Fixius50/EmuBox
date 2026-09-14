# Downloads and PS3 Architecture

## PS3 and RPCS3

PS3 is a first-class platform in `GameService` and uses the standalone `rpcs3`
profile. The scanner accepts PS3 disc/package extensions and recognizes an
installed game directory containing `PS3_GAME`. Launch resolution goes through
`CompatibilityService`, so enabled game-to-emulator associations and their
custom arguments are applied before `ProcessService` starts RPCS3.

## Downloads

La fase de integración de juegos y descargas está implementada. El catálogo
puede mostrar metadatos antes de que exista una ROM local; la descarga concreta
requiere una fuente autorizada registrada para el `gameId`. El dataset JSON de
catálogo no contiene enlaces de ROM.

Authorized JSON manifest URLs can be entered one per line in
`/etc/emubox/download-links.txt`. Blank lines and `#` comments are ignored.
EmuBox supports two manifest formats:

1. **Hydra-compatible format (Standard):**
   An object with `downloads[]` (or root array of download items) containing:
   `title`, `uris` (array of download URLs/magnets), `fileSize` (human-readable string
   e.g. `"13.58 GB"`, `"450 MB"` or bytes), and `uploadDate` (ISO 8601 string).
   Explicit item/manifest platforms take precedence, including PS4 and 3DS.
   Otherwise EmuBox considers title tags, whole-word source hints and unambiguous
   filename extensions such as `.sfc` or `.3ds`. HTTP hosts and query strings are
   not platform evidence; magnet names use structured URL decoding. Ambiguous
   `.pkg`, `.pbp`, `.rvz` and `.ciso` extensions do not imply a console. The legacy
   fallback remains `pc`, not a guarantee of platform identification or playability.
2. **Legacy EmuBox format:**
   An array or object with `games[]` providing `platform` and a direct HTTP/HTTPS `url`,
   with optional `name`, `gameId`, `checksum` and `sizeBytes`.

`import_download_links`, `import_downloads_from_json`, and `import_downloads_from_url`
register sources and catalog metadata without creating download jobs. The UI
receives an error for partial batch failures while successful imports remain saved.
The normalizer handles both formats; the unreachable duplicate importer was removed.
HTTP cache version 2 re-evaluates classifications from the previous rules. The UI
invokes `download_game` for the selected source. Transfer completion and game
preparation are separate: Inno Setup EXE extraction and PS3 PKG preparation use
isolated native tools, and downloaded bytes alone never establish installation.
The PS3 preparator requires local firmware and validates installation evidence;
the final launch candidate is selected explicitly. Arbitrary EXE installers are
not executed.

The manager owns the lifecycle and source snapshots; providers transfer content.
Validated packages are published to an exclusive directory:

```text
/var/lib/emubox/games/<platform>/<job-id>/
```

Private staging is excluded from the scanner. Verification precedes publication,
ZIP preparation rejects unsafe paths and managed packages avoid duplicate imports.
HTTP and BitTorrent (aria2) are separate providers; remote torrent descriptors
are obtained over HTTP before BitTorrent transfers their content. Downloads start
only after explicit source selection. See [download-providers.md](download-providers.md)
for dependencies, seeding policy, preparation and unsupported host connectors.

The Rust implementation now lives under `services/downloads/`; its manager separates
queue, transfer and publication. Installer detection, sandbox, firmware, validation
and launch configuration have dedicated modules. Library scanning and persistence
live under `services/library/games/`. Existing service names remain public reexports,
so IPC commands and source/job identities are unchanged by the refactor.

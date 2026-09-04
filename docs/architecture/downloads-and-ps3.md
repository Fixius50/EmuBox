# Downloads and PS3 Architecture

## PS3 and RPCS3

PS3 is a first-class platform in `GameService` and uses the standalone `rpcs3`
profile. The scanner accepts PS3 disc/package extensions and recognizes an
installed game directory containing `PS3_GAME`. Launch resolution goes through
`CompatibilityService`, so enabled game-to-emulator associations and their
custom arguments are applied before `ProcessService` starts RPCS3.

## Downloads

Authorized JSON manifest URLs can be entered one per line in
`/etc/emubox/download-links.txt`. Blank lines and `#` comments are ignored.
EmuBox supports two manifest formats:

1. **Hydra-compatible format (Standard):**
   An object with `downloads[]` (or root array of download items) containing:
   `title`, `uris` (array of download URLs/magnets), `fileSize` (human-readable string
   e.g. `"13.58 GB"`, `"450 MB"` or bytes), and `uploadDate` (ISO 8601 string).
   EmuBox automatically infers the target platform based on title tags (e.g. `[PS1]`,
   `[PS2]`, `[PS3]`, `[PSP]`, `[SNES]`), file extensions (`.pkg`, `.sfc`, etc.), and source
   hints, defaulting to PC/Linux.
2. **Legacy EmuBox format:**
   An array or object with `games[]` providing `platform` and a direct HTTP/HTTPS `url`,
   with optional `name`, `gameId`, `checksum` and `sizeBytes`.

`import_download_links`, `import_downloads_from_json`, and `import_downloads_from_url`
create queued jobs automatically and register the games in the catalog so that they are
immediately visible in the library.

EmuBox owns the download lifecycle. `DownloadService` stores sources and jobs
in SQLite and writes HTTP downloads to:

```text
/var/lib/emubox/games/<platform>/<filename>
```

Downloads are written to a sibling `.part` file and renamed only after the
transfer completes. An optional SHA-256 checksum is verified before completion.
The library watcher then discovers the final file and `GameService` updates the
library. Download code never inserts games directly into the games table without going
through the catalog and scanner pipeline.

Sources of type HTTP/HTTPS are automatically queued for download execution.
Torrent and magnet sources are registered in the catalog and SQLite sources table,
while their direct execution remains reserved for a dedicated adapter.
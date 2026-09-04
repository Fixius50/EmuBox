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
Each manifest is an array or an object with `games[]`; entries provide `platform`
and a direct HTTP/HTTPS `url`, with optional `name`, `gameId`, `checksum` and
`sizeBytes`. `import_download_links` creates queued jobs automatically and never
treats the manifest itself as a game file.

EmuBox owns the download lifecycle. `DownloadService` stores sources and jobs
in SQLite and writes HTTP downloads to:

```text
/var/lib/emubox/games/<platform>/<filename>
```

Downloads are written to a sibling `.part` file and renamed only after the
transfer completes. An optional SHA-256 checksum is verified before completion.
The library watcher then discovers the final file and `GameService` updates the
library. Download code never inserts games directly into the games table.

Only HTTP/HTTPS sources are enabled in the initial implementation. Torrent and
magnet sources are rejected until a dedicated adapter is implemented and
audited. Hydra is not embedded, does not launch games, and does not manage the
EmuBox library; its queue, retry, resume, and progress patterns are reference
material only.
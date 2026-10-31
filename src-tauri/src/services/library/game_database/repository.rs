use super::dat::Release;
use super::{canonical_title, normalize_title, now, AUTHORITY};
use crate::{errors::EmuBoxError, services::db_service::DatabaseService};
use rusqlite::{params, OptionalExtension};
use sha2::{Digest, Sha256};
use std::collections::{BTreeSet, HashSet};

fn hash_id(prefix: &str, parts: &[&str]) -> String {
    let digest = Sha256::digest(parts.join("\u{1f}").as_bytes());
    format!("{prefix}-{:x}", digest)
}

fn without_package_suffix(title: &str) -> Option<&str> {
    let trimmed = title.trim();
    let lower = trimmed.to_ascii_lowercase();
    for suffix in [" free download", " [p2p]", " [gog]", " [steam]", " [epic]", " repack fitgirl", " repack dodi", " scene skidrow", " license gog"] {
        if lower.ends_with(suffix) {
            return Some(trimmed[..trimmed.len() - suffix.len()].trim_end());
        }
    }
    None
}

pub fn ensure_local_index() -> Result<usize, EmuBoxError> {
    let mut connection = DatabaseService::get_connection()?;
    let transaction = connection
        .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
        .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
    let public_platforms: HashSet<String> = {
        let mut statement = transaction.prepare("SELECT DISTINCT platform_id FROM canonical_games WHERE authority='libretro'")
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        let platforms = statement.query_map([], |row| row.get(0))
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?
            .collect::<Result<_, _>>()
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        platforms
    };
    let rows = {
        let mut statement = transaction.prepare("SELECT game.id, game.title, game.platform_id, match.canonical_game_id, canonical.favorite FROM games AS game LEFT JOIN catalog_game_matches AS match ON match.catalog_game_id=game.id LEFT JOIN canonical_games AS canonical ON canonical.id=match.canonical_game_id WHERE match.catalog_game_id IS NULL OR (canonical.authority='local-derived' AND (lower(trim(game.title)) LIKE '% free download' OR lower(trim(game.title)) LIKE '% [p2p]' OR lower(trim(game.title)) LIKE '% [gog]' OR lower(trim(game.title)) LIKE '% [steam]' OR lower(trim(game.title)) LIKE '% [epic]' OR lower(trim(game.title)) LIKE '% repack fitgirl' OR lower(trim(game.title)) LIKE '% repack dodi' OR lower(trim(game.title)) LIKE '% scene skidrow' OR lower(trim(game.title)) LIKE '% license gog' OR ((instr(game.title,'[')>0 OR instr(game.title,'(')>0) AND EXISTS (SELECT 1 FROM canonical_games AS public WHERE public.platform_id=game.platform_id AND public.authority='libretro'))))")
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<i64>>(4)?,
                ))
            })
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        rows
    };
    let mut changed = 0;
    for (game_id, title, platform, existing_match, was_favorite) in &rows {
        let display = canonical_title(without_package_suffix(title).unwrap_or(title));
        let normalized = normalize_title(&display);
        if public_platforms.contains(platform) {
            if let Some(matched) = authoritative_match(&transaction, platform, title)? {
                save_match(&transaction, game_id, &matched)?;
                changed += 1;
                continue;
            }
        }
        let canonical_id = hash_id("game-db-local", &[platform, &normalized]);
        if existing_match.as_deref() == Some(&canonical_id) || (existing_match.is_some() && without_package_suffix(title).is_none()) {
            continue;
        }
        transaction.execute("INSERT INTO canonical_games(id,authority,authority_id,title,normalized_title,platform_id,updated_at) VALUES(?1,'local-derived',?2,?3,?4,?5,?6) ON CONFLICT(id) DO UPDATE SET updated_at=excluded.updated_at",
            params![canonical_id, canonical_id, display, normalized, platform, now()])
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        if existing_match.is_some() {
            if was_favorite == &Some(1) {
                transaction.execute("UPDATE canonical_games SET favorite=1 WHERE id=?1", [&canonical_id]).map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
            }
            transaction.execute("UPDATE catalog_game_matches SET canonical_game_id=?1,matched_at=?2 WHERE catalog_game_id=?3 AND confidence=60", params![canonical_id, now(), game_id])
                .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
            transaction.execute("DELETE FROM canonical_games WHERE id=?1 AND authority='local-derived' AND NOT EXISTS (SELECT 1 FROM catalog_game_matches WHERE canonical_game_id=?1)", [existing_match]).map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
            changed += 1;
            continue;
        }
        transaction.execute("INSERT OR IGNORE INTO canonical_aliases(canonical_game_id,platform_id,normalized_alias,kind) VALUES(?1,?2,?3,'local-title')",
            params![canonical_id, platform, normalize_title(title)])
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        transaction
            .execute(
                "INSERT INTO catalog_game_matches VALUES(?1,?2,NULL,'local-title',60,?3)",
                params![game_id, canonical_id, now()],
            )
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        changed += 1;
    }
    transaction
        .commit()
        .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
    Ok(changed)
}

pub(super) fn import_releases(platform: &str, releases: &[Release]) -> Result<usize, EmuBoxError> {
    let mut connection = DatabaseService::get_connection()?;
    let transaction = connection
        .transaction()
        .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
    let mut canonical_ids = BTreeSet::new();
    for release in releases {
        let canonical_normalized = normalize_title(&release.canonical_title);
        let canonical_id = hash_id("game-db-libretro", &[platform, &canonical_normalized]);
        let release_normalized = normalize_title(&release.title);
        let release_identity = release
            .sha1
            .as_deref()
            .or(release.md5.as_deref())
            .or(release.crc.as_deref())
            .or(release.serial.as_deref())
            .unwrap_or(&release_normalized);
        let release_id = hash_id("release-db-libretro", &[platform, release_identity]);
        canonical_ids.insert(canonical_id.clone());
        transaction.execute("INSERT INTO canonical_games(id,authority,authority_id,title,normalized_title,platform_id,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7) ON CONFLICT(id) DO UPDATE SET title=excluded.title,normalized_title=excluded.normalized_title,updated_at=excluded.updated_at",
            params![canonical_id, AUTHORITY, canonical_id, release.canonical_title, canonical_normalized, platform, now()])
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        transaction.execute("INSERT INTO game_releases(id,canonical_game_id,authority,authority_id,title,normalized_title,region,serial,crc,md5,sha1) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11) ON CONFLICT(id) DO UPDATE SET title=excluded.title,normalized_title=excluded.normalized_title,region=excluded.region,serial=excluded.serial,crc=excluded.crc,md5=excluded.md5,sha1=excluded.sha1",
            params![release_id, canonical_id, AUTHORITY, release_id, release.title, release_normalized, release.region, release.serial, release.crc, release.md5, release.sha1])
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        for (alias, kind) in [
            (&canonical_normalized, "canonical-title"),
            (&release_normalized, "release-title"),
        ] {
            transaction
                .execute(
                    "INSERT OR IGNORE INTO canonical_aliases VALUES(?1,?2,?3,?4)",
                    params![canonical_id, platform, alias, kind],
                )
                .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        }
    }
    let games = {
        let mut statement = transaction
            .prepare("SELECT id,title FROM games WHERE platform_id=?1")
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        let rows = statement
            .query_map([platform], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        rows
    };
    for (game_id, title) in games {
        if let Some(matched) = authoritative_match(&transaction, platform, &title)? {
            save_match(&transaction, &game_id, &matched)?;
        }
    }
    transaction.execute("DELETE FROM canonical_games WHERE authority='local-derived' AND NOT EXISTS(SELECT 1 FROM catalog_game_matches WHERE canonical_game_id=canonical_games.id)", [])
        .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
    transaction
        .commit()
        .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
    Ok(canonical_ids.len())
}

struct CanonicalMatch {
    canonical_id: String,
    release_id: Option<String>,
    method: &'static str,
    confidence: i32,
}

#[derive(Clone, Copy)]
enum MatchKind {
    ReleaseTitle,
    CanonicalTitle,
}

impl MatchKind {
    fn alias_kind(self) -> &'static str {
        match self {
            Self::ReleaseTitle => "release-title",
            Self::CanonicalTitle => "canonical-title",
        }
    }

    fn method(self) -> &'static str {
        match self {
            Self::ReleaseTitle => "libretro-release-title",
            Self::CanonicalTitle => "libretro-canonical-title",
        }
    }

    fn confidence(self) -> i32 {
        match self {
            Self::ReleaseTitle => 100,
            Self::CanonicalTitle => 90,
        }
    }
}

fn authoritative_match(
    connection: &rusqlite::Connection,
    platform: &str,
    title: &str,
) -> Result<Option<CanonicalMatch>, EmuBoxError> {
    let exact = unique_match(
        connection,
        platform,
        &normalize_title(title),
        MatchKind::ReleaseTitle,
    )?;
    if exact.is_some() {
        return Ok(exact);
    }
    let canonical = unique_match(
        connection,
        platform,
        &normalize_title(&canonical_title(title)),
        MatchKind::CanonicalTitle,
    )?;
    if canonical.is_some() {
        return Ok(canonical);
    }
    if let Some(unpacked) = without_package_suffix(title) {
        let release = unique_match(connection, platform, &normalize_title(unpacked), MatchKind::ReleaseTitle)?;
        if release.is_some() {
            return Ok(release);
        }
        let canonical = unique_match(connection, platform, &normalize_title(&canonical_title(unpacked)), MatchKind::CanonicalTitle)?;
        if canonical.is_some() {
            return Ok(canonical);
        }
    }
    contained_public_match(connection, platform, title)
}

fn annotation_only(tokens: &[&str]) -> bool {
    if tokens.is_empty() {
        return true;
    }
    if tokens.len() == 2 && matches!(tokens[0], "rev" | "revision" | "disc" | "disk" | "v") {
        return tokens[1].len() <= 3 && tokens[1].chars().all(|value| value.is_ascii_alphanumeric());
    }
    tokens.iter().all(|token| matches!(*token,
        "en" | "es" | "fr" | "de" | "it" | "ja" | "ru" | "multi" |
        "english" | "spanish" | "español" | "français" | "french" | "german" |
        "deutsch" | "italian" | "japanese" | "japonés" | "russian" |
        "usa" | "europe" | "world" | "japan" | "spain" | "asia" |
        "beta" | "demo" | "proto" | "p2p" | "gog" | "steam" | "epic"))
}

fn contained_public_match(connection: &rusqlite::Connection, platform: &str, title: &str) -> Result<Option<CanonicalMatch>, EmuBoxError> {
    if !title.contains(['[', '(']) || title.len() > 256 {
        return Ok(None);
    }
    let normalized = normalize_title(title);
    let words: Vec<_> = normalized.split_whitespace().collect();
    if words.len() < 2 || words.len() > 16 {
        return Ok(None);
    }
    for length in (1..words.len()).rev() {
        let mut found: Option<CanonicalMatch> = None;
        for start in 0..=words.len() - length {
            let end = start + length;
            if !annotation_only(&words[..start]) || !annotation_only(&words[end..]) {
                continue;
            }
            let alias = words[start..end].join(" ");
            let matched = unique_match(connection, platform, &alias, MatchKind::ReleaseTitle)?
                .or(unique_match(connection, platform, &alias, MatchKind::CanonicalTitle)?);
            if let Some(matched) = matched {
                if found.as_ref().is_some_and(|previous| previous.canonical_id != matched.canonical_id) {
                    return Ok(None);
                }
                found = Some(matched);
            }
        }
        if found.is_some() {
            return Ok(found);
        }
    }
    Ok(None)
}

fn save_match(
    transaction: &rusqlite::Transaction<'_>,
    game_id: &str,
    matched: &CanonicalMatch,
) -> Result<(), EmuBoxError> {
    transaction
        .execute(
            "UPDATE canonical_games SET favorite=1 WHERE id=?1 AND EXISTS(
            SELECT 1 FROM catalog_game_matches AS previous
            JOIN canonical_games AS canonical ON canonical.id=previous.canonical_game_id
            WHERE previous.catalog_game_id=?2 AND previous.confidence<?3 AND canonical.favorite=1
        )",
            params![matched.canonical_id, game_id, matched.confidence],
        )
        .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
    transaction.execute(
        "INSERT INTO catalog_game_matches(catalog_game_id,canonical_game_id,release_id,method,confidence,matched_at)
         VALUES(?1,?2,?3,?4,?5,?6)
         ON CONFLICT(catalog_game_id) DO UPDATE SET
            canonical_game_id=excluded.canonical_game_id,release_id=excluded.release_id,
            method=excluded.method,confidence=excluded.confidence,matched_at=excluded.matched_at
         WHERE excluded.confidence>catalog_game_matches.confidence",
        params![game_id, matched.canonical_id, matched.release_id, matched.method, matched.confidence, now()],
    ).map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
    Ok(())
}

fn unique_match(
    connection: &rusqlite::Connection,
    platform: &str,
    alias: &str,
    kind: MatchKind,
) -> Result<Option<CanonicalMatch>, EmuBoxError> {
    if alias.is_empty() {
        return Ok(None);
    }
    let mut statement = connection.prepare("SELECT DISTINCT alias.canonical_game_id, CASE WHEN ?3='release-title' THEN release.id ELSE NULL END FROM canonical_aliases AS alias LEFT JOIN game_releases AS release ON release.canonical_game_id=alias.canonical_game_id AND release.normalized_title=alias.normalized_alias WHERE alias.platform_id=?1 AND alias.normalized_alias=?2 AND alias.kind=?3 LIMIT 2")
        .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
    let rows = statement
        .query_map(params![platform, alias, kind.alias_kind()], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
        })
        .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
    if rows.len() != 1 {
        return Ok(None);
    }
    let (canonical, release) = rows.into_iter().next().unwrap();
    Ok(Some(CanonicalMatch {
        canonical_id: canonical,
        release_id: release,
        method: kind.method(),
        confidence: kind.confidence(),
    }))
}

pub fn options(canonical_id: &str) -> Result<crate::models::CanonicalGameOptions, EmuBoxError> {
    let connection = DatabaseService::get_connection()?;
    let representative: String = connection.query_row("SELECT game.id FROM games AS game JOIN catalog_game_matches AS match ON match.catalog_game_id=game.id WHERE match.canonical_game_id=?1 ORDER BY game.rom_path IS NOT NULL DESC, game.id LIMIT 1", [canonical_id], |row| row.get(0)).optional()
        .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?
        .ok_or_else(|| EmuBoxError::NotFound(format!("Juego canonico inexistente: {canonical_id}")))?;
    let game = crate::services::GameService::get_game_by_id(representative)?
        .ok_or_else(|| EmuBoxError::NotFound(format!("Juego inexistente: {canonical_id}")))?;
    let catalog_games = {
        let mut statement = connection.prepare("SELECT game.id,COALESCE(release.id,game.id),COALESCE(release.title,game.title),release.region,game.rom_path IS NOT NULL FROM games AS game JOIN catalog_game_matches AS match ON match.catalog_game_id=game.id LEFT JOIN game_releases AS release ON release.id=match.release_id WHERE match.canonical_game_id=?1 ORDER BY game.rom_path IS NOT NULL DESC, COALESCE(release.title,game.title), game.id")
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        let rows = statement
            .query_map([canonical_id], |row| {
                Ok(crate::models::GameReleaseOption {
                    catalog_game_id: row.get(0)?,
                    id: row.get(1)?,
                    title: row.get(2)?,
                    region: row.get(3)?,
                    installed: row.get(4)?,
                    source_count: 0,
                    downloadable_source_count: 0,
                })
            })
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        rows
    };
    let mut releases = Vec::new();
    let mut sources = Vec::new();
    for mut release in catalog_games {
        let options = crate::services::DownloadService::list_sources(&release.catalog_game_id)?;
        release.source_count = options.len();
        release.downloadable_source_count =
            options.iter().filter(|source| source.downloadable).count();
        releases.push(release);
        sources.extend(options);
    }
    Ok(crate::models::CanonicalGameOptions {
        game,
        releases,
        sources,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_prioritizes_exact_release_and_rejects_ambiguous_identity() {
        let connection = rusqlite::Connection::open_in_memory().unwrap();
        connection.execute_batch(
            "CREATE TABLE canonical_aliases(canonical_game_id TEXT, platform_id TEXT, normalized_alias TEXT, kind TEXT);
             CREATE TABLE game_releases(id TEXT, canonical_game_id TEXT, normalized_title TEXT);
             INSERT INTO canonical_aliases VALUES
                ('canonical-one','nes','fixture usa','release-title'),
                ('canonical-one','nes','fixture','canonical-title'),
                ('canonical-two','nes','fixture','canonical-title');
             INSERT INTO game_releases VALUES ('release-one','canonical-one','fixture usa');"
        ).unwrap();
        let matched = authoritative_match(&connection, "nes", "Fixture (USA)")
            .unwrap()
            .unwrap();
        assert_eq!(matched.canonical_id, "canonical-one");
        assert_eq!(matched.release_id.as_deref(), Some("release-one"));
        assert_eq!(matched.confidence, 100);
        assert_eq!(matched.method, "libretro-release-title");
        let packaged = authoritative_match(&connection, "nes", "Fixture (USA) Free Download")
            .unwrap()
            .expect("un sufijo de paquete no debe impedir enlazar la release publica unica");
        assert_eq!(packaged.canonical_id, "canonical-one");
        assert_eq!(packaged.release_id.as_deref(), Some("release-one"));
        let annotated = authoritative_match(&connection, "nes", "[Español] Fixture (USA) [Rev A]")
            .unwrap()
            .expect("el titulo literal del DAT debe encontrarse dentro de marcas de idioma y revision");
        assert_eq!(annotated.release_id.as_deref(), Some("release-one"));
        assert_eq!(authoritative_match(&connection, "nes", "☆Fixture™ (USA) [Español]")
            .unwrap().unwrap().release_id.as_deref(), Some("release-one"));
        assert!(authoritative_match(&connection, "nes", "[Español] Fixture [Rev A]")
            .unwrap().is_none());
        assert!(authoritative_match(&connection, "nes", "Fixture (USA) [Special Edition]")
            .unwrap().is_none());
        assert!(authoritative_match(&connection, "nes", "Fixture - Special Edition")
            .unwrap()
            .is_none());
        assert!(authoritative_match(&connection, "nes", "Fixture")
            .unwrap()
            .is_none());
        assert!(authoritative_match(&connection, "snes", "Fixture (USA)")
            .unwrap()
            .is_none());
        connection
            .execute(
                "DELETE FROM canonical_aliases WHERE canonical_game_id='canonical-two'",
                [],
            )
            .unwrap();
        let matched = authoritative_match(&connection, "nes", "Fixture (Europe)")
            .unwrap()
            .unwrap();
        assert_eq!(matched.canonical_id, "canonical-one");
        assert!(matched.release_id.is_none());
        assert_eq!(matched.confidence, 90);
        assert_eq!(matched.method, "libretro-canonical-title");
    }
}

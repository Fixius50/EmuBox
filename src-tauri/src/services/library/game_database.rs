use crate::{errors::EmuBoxError, services::db_service::DatabaseService};
use reqwest::header::{ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED};
use rusqlite::{params, OptionalExtension};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, io::Read, time::Duration};

const AUTHORITY: &str = "libretro";
const LICENSE: &str = "CC-BY-SA-4.0";
const MAX_DATABASE_BYTES: usize = 16 * 1024 * 1024;
const SUCCESS_CHECK_INTERVAL: i64 = 24 * 60 * 60;
const ERROR_RETRY_INTERVAL: i64 = 60 * 60;
const BASE: &str = "https://raw.githubusercontent.com/libretro/libretro-database/master";

struct Source {
    platform: &'static str,
    path: &'static str,
}

const SOURCES: &[Source] = &[
    Source {
        platform: "nes",
        path: "metadat/no-intro/Nintendo%20-%20Nintendo%20Entertainment%20System.dat",
    },
    Source {
        platform: "snes",
        path: "metadat/no-intro/Nintendo%20-%20Super%20Nintendo%20Entertainment%20System.dat",
    },
    Source {
        platform: "gba",
        path: "metadat/no-intro/Nintendo%20-%20Game%20Boy%20Advance.dat",
    },
    Source {
        platform: "n64",
        path: "metadat/no-intro/Nintendo%20-%20Nintendo%2064.dat",
    },
    Source {
        platform: "genesis",
        path: "metadat/no-intro/Sega%20-%20Mega%20Drive%20-%20Genesis.dat",
    },
    Source {
        platform: "3ds",
        path: "metadat/no-intro/Nintendo%20-%20Nintendo%203DS.dat",
    },
    Source {
        platform: "nds",
        path: "metadat/no-intro/Nintendo%20-%20Nintendo%20DS.dat",
    },
    Source {
        platform: "psp",
        path: "metadat/no-intro/Sony%20-%20PlayStation%20Portable.dat",
    },
    Source {
        platform: "ps1",
        path: "metadat/redump/Sony%20-%20PlayStation.dat",
    },
    Source {
        platform: "ps2",
        path: "metadat/redump/Sony%20-%20PlayStation%202.dat",
    },
    Source {
        platform: "ps3",
        path: "metadat/redump/Sony%20-%20PlayStation%203.dat",
    },
    Source {
        platform: "gamecube",
        path: "metadat/redump/Nintendo%20-%20GameCube.dat",
    },
    Source {
        platform: "dreamcast",
        path: "metadat/redump/Sega%20-%20Dreamcast.dat",
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct Release {
    title: String,
    canonical_title: String,
    region: Option<String>,
    serial: Option<String>,
    crc: Option<String>,
    md5: Option<String>,
    sha1: Option<String>,
}

fn hash_id(prefix: &str, parts: &[&str]) -> String {
    let digest = Sha256::digest(parts.join("\u{1f}").as_bytes());
    format!("{prefix}-{:x}", digest)
}

pub fn normalize_title(value: &str) -> String {
    value
        .trim_start_matches(|character: char| !character.is_alphanumeric())
        .chars()
        .flat_map(char::to_lowercase)
        .map(|character| {
            if character.is_alphanumeric() {
                character
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn edition_marker(value: &str) -> bool {
    let lower = value.to_lowercase();
    [
        "usa",
        "europe",
        "world",
        "japan",
        "asia",
        "korea",
        "brazil",
        "australia",
        "rev ",
        "revision",
        "disc ",
        "disk ",
        "side ",
        "beta",
        "demo",
        "proto",
        "sample",
        "unl",
        "unlicensed",
        "virtual console",
        "psn",
        "v1.",
        "v2.",
        "en,",
        "fr,",
        "de,",
        "es,",
        "it,",
        "ja,",
    ]
    .iter()
    .any(|marker| lower == *marker || lower.starts_with(marker))
}

pub fn canonical_title(value: &str) -> String {
    let mut title = value
        .trim_start_matches(|character: char| !character.is_alphanumeric())
        .trim()
        .to_string();
    loop {
        let Some(end) = title.strip_suffix(')') else {
            break;
        };
        let Some(start) = end.rfind('(') else { break };
        if !edition_marker(end[start + 1..].trim()) {
            break;
        }
        title.truncate(start);
        title = title.trim_end().to_string();
    }
    for separator in [" [", " {"].iter() {
        if let Some(index) = title.rfind(separator) {
            let marker = title[index + 2..].trim_end_matches([']', '}']).trim();
            if edition_marker(marker) {
                title.truncate(index);
            }
        }
    }
    title
        .trim()
        .trim_end_matches(['-', '–', '—', ':'])
        .trim()
        .to_string()
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Word(String),
    Text(String),
    Open,
    Close,
}

fn lex(input: &str) -> Result<Vec<Token>, EmuBoxError> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(character) = chars.next() {
        if character.is_whitespace() {
            continue;
        }
        match character {
            '(' => tokens.push(Token::Open),
            ')' => tokens.push(Token::Close),
            '"' => {
                let mut text = String::new();
                let mut closed = false;
                while let Some(next) = chars.next() {
                    match next {
                        '"' => {
                            closed = true;
                            break;
                        }
                        '\\' => match chars.next() {
                            Some('"') => text.push('"'),
                            Some('\\') => text.push('\\'),
                            Some(value) => {
                                text.push('\\');
                                text.push(value);
                            }
                            None => break,
                        },
                        value => text.push(value),
                    }
                }
                if !closed {
                    return Err(EmuBoxError::InvalidConfiguration(
                        "DAT maestro contiene texto sin cerrar".into(),
                    ));
                }
                tokens.push(Token::Text(text));
            }
            value => {
                let mut word = value.to_string();
                while chars
                    .peek()
                    .is_some_and(|next| !next.is_whitespace() && !matches!(next, '(' | ')' | '"'))
                {
                    word.push(chars.next().unwrap());
                }
                tokens.push(Token::Word(word));
            }
        }
    }
    Ok(tokens)
}

fn field(tokens: &[Token], key: &str) -> Option<String> {
    tokens.windows(2).find_map(|pair| match pair {
        [Token::Word(name), Token::Text(value)] if name == key => Some(value.clone()),
        [Token::Word(name), Token::Word(value)] if name == key => Some(value.clone()),
        _ => None,
    })
}

fn parse_dat(input: &str) -> Result<Vec<Release>, EmuBoxError> {
    let tokens = lex(input)?;
    let mut releases = Vec::new();
    let mut index = 0;
    while index + 1 < tokens.len() {
        if tokens[index] != Token::Word("game".into()) || tokens[index + 1] != Token::Open {
            index += 1;
            continue;
        }
        let start = index + 2;
        let mut depth = 1;
        index = start;
        while index < tokens.len() && depth > 0 {
            match tokens[index] {
                Token::Open => depth += 1,
                Token::Close => depth -= 1,
                _ => {}
            }
            index += 1;
        }
        if depth != 0 {
            return Err(EmuBoxError::InvalidConfiguration(
                "DAT maestro contiene parentesis sin cerrar".into(),
            ));
        }
        let body = &tokens[start..index - 1];
        let Some(title) = field(body, "name").or_else(|| field(body, "description")) else {
            continue;
        };
        let canonical = canonical_title(&title);
        if canonical.is_empty() {
            continue;
        }
        releases.push(Release {
            title,
            canonical_title: canonical,
            region: field(body, "region"),
            serial: field(body, "serial"),
            crc: field(body, "crc").map(|value| value.to_uppercase()),
            md5: field(body, "md5").map(|value| value.to_uppercase()),
            sha1: field(body, "sha1").map(|value| value.to_uppercase()),
        });
    }
    if releases.is_empty() {
        return Err(EmuBoxError::InvalidConfiguration(
            "DAT maestro sin juegos reconocibles".into(),
        ));
    }
    Ok(releases)
}

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

fn check_due(checked_at: i64, failed: bool, current_time: i64) -> bool {
    current_time.saturating_sub(checked_at)
        >= if failed {
            ERROR_RETRY_INTERVAL
        } else {
            SUCCESS_CHECK_INTERVAL
        }
}

pub fn ensure_local_index() -> Result<usize, EmuBoxError> {
    let mut connection = DatabaseService::get_connection()?;
    let transaction = connection
        .transaction()
        .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
    let rows = {
        let mut statement = transaction.prepare("SELECT game.id, game.title, game.platform_id FROM games AS game LEFT JOIN catalog_game_matches AS match ON match.catalog_game_id=game.id WHERE match.catalog_game_id IS NULL")
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        rows
    };
    for (game_id, title, platform) in &rows {
        let display = canonical_title(title);
        let normalized = normalize_title(&display);
        let release_key = normalize_title(title);
        let authoritative =
            unique_match(&transaction, platform, &release_key, Some("release-title"))?.or(
                unique_match(&transaction, platform, &normalized, Some("canonical-title"))?,
            );
        if let Some((canonical_id, release_id, method, confidence)) = authoritative {
            transaction
                .execute(
                    "INSERT INTO catalog_game_matches VALUES(?1,?2,?3,?4,?5,?6)",
                    params![game_id, canonical_id, release_id, method, confidence, now()],
                )
                .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
            continue;
        }
        let canonical_id = hash_id("game-db-local", &[platform, &normalized]);
        transaction.execute("INSERT INTO canonical_games(id,authority,authority_id,title,normalized_title,platform_id,updated_at) VALUES(?1,'local-derived',?2,?3,?4,?5,?6) ON CONFLICT(id) DO UPDATE SET updated_at=excluded.updated_at",
            params![canonical_id, canonical_id, display, normalized, platform, now()])
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        transaction.execute("INSERT OR IGNORE INTO canonical_aliases(canonical_game_id,platform_id,normalized_alias,kind) VALUES(?1,?2,?3,'local-title')",
            params![canonical_id, platform, normalize_title(title)])
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        transaction
            .execute(
                "INSERT INTO catalog_game_matches VALUES(?1,?2,NULL,'local-title',60,?3)",
                params![game_id, canonical_id, now()],
            )
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
    }
    transaction
        .commit()
        .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
    Ok(rows.len())
}

fn import_releases(platform: &str, releases: &[Release]) -> Result<usize, EmuBoxError> {
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
        let release_key = normalize_title(&title);
        let canonical_key = normalize_title(&canonical_title(&title));
        let exact = unique_match(&transaction, platform, &release_key, Some("release-title"))?;
        let matched = exact.or(unique_match(
            &transaction,
            platform,
            &canonical_key,
            Some("canonical-title"),
        )?);
        if let Some((canonical_id, release_id, method, confidence)) = matched {
            transaction.execute("INSERT INTO catalog_game_matches VALUES(?1,?2,?3,?4,?5,?6) ON CONFLICT(catalog_game_id) DO UPDATE SET canonical_game_id=excluded.canonical_game_id,release_id=excluded.release_id,method=excluded.method,confidence=excluded.confidence,matched_at=excluded.matched_at WHERE excluded.confidence>catalog_game_matches.confidence",
                params![game_id, canonical_id, release_id, method, confidence, now()])
                .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        }
    }
    transaction.execute("DELETE FROM canonical_games WHERE authority='local-derived' AND NOT EXISTS(SELECT 1 FROM catalog_game_matches WHERE canonical_game_id=canonical_games.id)", [])
        .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
    transaction
        .commit()
        .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
    Ok(canonical_ids.len())
}

fn unique_match(
    connection: &rusqlite::Connection,
    platform: &str,
    alias: &str,
    kind: Option<&str>,
) -> Result<Option<(String, Option<String>, &'static str, i32)>, EmuBoxError> {
    if alias.is_empty() {
        return Ok(None);
    }
    let mut statement = connection.prepare("SELECT DISTINCT alias.canonical_game_id, CASE WHEN ?3='release-title' THEN release.id ELSE NULL END FROM canonical_aliases AS alias LEFT JOIN game_releases AS release ON release.canonical_game_id=alias.canonical_game_id AND release.normalized_title=alias.normalized_alias WHERE alias.platform_id=?1 AND alias.normalized_alias=?2 AND (?3 IS NULL OR alias.kind=?3) LIMIT 2")
        .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
    let rows = statement
        .query_map(params![platform, alias, kind], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
        })
        .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
    if rows.len() != 1 {
        return Ok(None);
    }
    let (canonical, release) = rows.into_iter().next().unwrap();
    Ok(Some((
        canonical,
        release,
        if kind == Some("release-title") {
            "libretro-release-title"
        } else {
            "libretro-canonical-title"
        },
        if kind == Some("release-title") {
            100
        } else {
            90
        },
    )))
}

pub fn sync_all(mut updated: impl FnMut(&str, usize)) -> Result<usize, EmuBoxError> {
    let client = reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(45))
        .user_agent("EmuBox/1.0 (Libretro database sync; no credentials)")
        .build()
        .map_err(|error| EmuBoxError::ProcessFailed(error.to_string()))?;
    let mut imported = 0;
    let mut errors = Vec::new();
    for source in SOURCES {
        let url = format!("{BASE}/{}", source.path);
        let connection = DatabaseService::get_connection()?;
        let cached = connection.query_row("SELECT etag,modified,checked_at,error IS NOT NULL FROM game_database_sources WHERE platform_id=?1", [source.platform], |row| Ok((row.get::<_, Option<String>>(0)?, row.get::<_, Option<String>>(1)?, row.get::<_, i64>(2)?, row.get::<_, bool>(3)?))).optional()
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        if cached
            .as_ref()
            .is_some_and(|(_, _, checked_at, failed)| !check_due(*checked_at, *failed, now()))
        {
            continue;
        }
        let mut request = client.get(&url);
        if let Some((etag, modified, _, _)) = cached {
            if let Some(etag) = etag {
                request = request.header(IF_NONE_MATCH, etag);
            }
            if let Some(modified) = modified {
                request = request.header(IF_MODIFIED_SINCE, modified);
            }
        }
        let result = (|| -> Result<usize, EmuBoxError> {
            let response = request.send().map_err(|error| {
                EmuBoxError::ProcessFailed(format!("{}: {error}", source.platform))
            })?;
            if response.status() == reqwest::StatusCode::NOT_MODIFIED {
                connection.execute("UPDATE game_database_sources SET checked_at=?2,error=NULL WHERE platform_id=?1", params![source.platform, now()]).map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
                return Ok(0);
            }
            if !response.status().is_success() {
                return Err(EmuBoxError::ProcessFailed(format!(
                    "{}: HTTP {}",
                    source.platform,
                    response.status()
                )));
            }
            if response
                .content_length()
                .is_some_and(|length| length as usize > MAX_DATABASE_BYTES)
            {
                return Err(EmuBoxError::InvalidConfiguration(format!(
                    "{}: base maestra supera 16 MiB",
                    source.platform
                )));
            }
            let etag = response
                .headers()
                .get(ETAG)
                .and_then(|value| value.to_str().ok())
                .map(str::to_string);
            let modified = response
                .headers()
                .get(LAST_MODIFIED)
                .and_then(|value| value.to_str().ok())
                .map(str::to_string);
            let mut bytes = Vec::new();
            response
                .take((MAX_DATABASE_BYTES + 1) as u64)
                .read_to_end(&mut bytes)
                .map_err(|error| EmuBoxError::ProcessFailed(error.to_string()))?;
            if bytes.len() > MAX_DATABASE_BYTES {
                return Err(EmuBoxError::InvalidConfiguration(format!(
                    "{}: base maestra supera 16 MiB",
                    source.platform
                )));
            }
            let text = String::from_utf8(bytes).map_err(|_| {
                EmuBoxError::InvalidConfiguration(format!("{}: DAT no es UTF-8", source.platform))
            })?;
            let releases = parse_dat(&text)?;
            let count = import_releases(source.platform, &releases)?;
            connection.execute("INSERT INTO game_database_sources(platform_id,authority,url,etag,modified,checked_at,imported_at,entry_count,error) VALUES(?1,?2,?3,?4,?5,?6,?6,?7,NULL) ON CONFLICT(platform_id) DO UPDATE SET authority=excluded.authority,url=excluded.url,etag=excluded.etag,modified=excluded.modified,checked_at=excluded.checked_at,imported_at=excluded.imported_at,entry_count=excluded.entry_count,error=NULL",
                params![source.platform, AUTHORITY, url, etag, modified, now(), releases.len() as i64]).map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
            updated(source.platform, count);
            Ok(count)
        })();
        match result {
            Ok(count) => imported += count,
            Err(error) => {
                let _=connection.execute("INSERT INTO game_database_sources(platform_id,authority,url,checked_at,error) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(platform_id) DO UPDATE SET checked_at=excluded.checked_at,error=excluded.error", params![source.platform,AUTHORITY,url,now(),error.to_string()]);
                errors.push(error.to_string());
            }
        }
    }
    if errors.is_empty() {
        Ok(imported)
    } else {
        Err(EmuBoxError::Unknown(format!(
            "Game Database parcial ({LICENSE}): {}",
            errors.join("; ")
        )))
    }
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
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, bool>(4)?,
                ))
            })
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        rows
    };
    let mut releases = Vec::new();
    let mut sources = Vec::new();
    for (catalog_game_id, release_id, title, region, installed) in catalog_games {
        let options = crate::services::DownloadService::list_sources(&catalog_game_id)?;
        releases.push(crate::models::GameReleaseOption {
            id: release_id,
            catalog_game_id,
            title,
            region,
            installed,
            source_count: options.len(),
            downloadable_source_count: options.iter().filter(|source| source.downloadable).count(),
        });
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
    fn dat_parser_preserves_versions_and_identity_fields() {
        let releases = parse_dat(r##"clrmamepro ( name "Fixture" )
    game ( name "#Mario Bros. (USA) (Rev 1)" description "ignored" region "USA" serial "NES-MA-USA" rom ( name "Mario.nes" size 4 crc 01020304 md5 AABB sha1 CCDDEE ) )
    game ( name "Mario Bros. (Europe)" rom ( name "Mario Europe.nes" crc 05060708 ) )"##).unwrap();
        assert_eq!(releases.len(), 2);
        assert_eq!(releases[0].canonical_title, "Mario Bros.");
        assert_eq!(releases[0].serial.as_deref(), Some("NES-MA-USA"));
        assert_eq!(releases[0].crc.as_deref(), Some("01020304"));
        assert_ne!(
            hash_id("release", &["nes", &releases[0].title]),
            hash_id("release", &["nes", &releases[1].title])
        );
    }
    #[test]
    fn title_matching_is_conservative_and_strips_only_known_markers() {
        assert_eq!(
            canonical_title("^^Mario Bros. (USA) (Rev 1)"),
            "Mario Bros."
        );
        assert_eq!(
            canonical_title("Mario (The Lost Levels)"),
            "Mario (The Lost Levels)"
        );
        assert_eq!(normalize_title("  #Mario: Bros.  "), "mario bros");
        assert!(!edition_marker("The Lost Levels"));
        assert!(!check_due(1_000, false, 1_000 + SUCCESS_CHECK_INTERVAL - 1));
        assert!(check_due(1_000, false, 1_000 + SUCCESS_CHECK_INTERVAL));
        assert!(!check_due(1_000, true, 1_000 + ERROR_RETRY_INTERVAL - 1));
        assert!(check_due(1_000, true, 1_000 + ERROR_RETRY_INTERVAL));
    }
    #[test]
    fn local_variants_upgrade_to_authoritative_identity_without_changing_catalog_ids() {
        use crate::{models::game::CatalogEntry, services::GameService};
        GameService::get_platforms().unwrap();
        let ids = ["game-db-canonical-fixture-a", "game-db-canonical-fixture-b"];
        for (id, title) in [
            (ids[0], "#Canonical Fixture (USA)"),
            (ids[1], "^^Canonical Fixture [Rev 1]"),
        ] {
            GameService::upsert_catalog_entry(CatalogEntry {
                id: id.into(),
                title: title.into(),
                platform_id: "nes".into(),
                platform_name: "Nintendo Entertainment System".into(),
                release_year: None,
                genre: None,
                developer: None,
                publisher: None,
                rating: None,
                cover_image: None,
                backdrop_image: None,
                description: None,
            })
            .unwrap();
        }
        ensure_local_index().unwrap();
        let connection = DatabaseService::get_connection().unwrap();
        let local_count: i64 = connection.query_row("SELECT COUNT(DISTINCT canonical_game_id) FROM catalog_game_matches WHERE catalog_game_id IN (?1,?2)", ids, |row| row.get(0)).unwrap();
        assert_eq!(local_count, 1);
        drop(connection);
        import_releases(
            "nes",
            &[Release {
                title: "Canonical Fixture (USA)".into(),
                canonical_title: "Canonical Fixture".into(),
                region: Some("USA".into()),
                serial: None,
                crc: Some("ABCDEF01".into()),
                md5: None,
                sha1: None,
            }],
        )
        .unwrap();
        let connection = DatabaseService::get_connection().unwrap();
        let matches: Vec<(String, String, i64)> = {
            let mut statement = connection.prepare("SELECT match.catalog_game_id,canonical.authority,match.confidence FROM catalog_game_matches AS match JOIN canonical_games AS canonical ON canonical.id=match.canonical_game_id WHERE match.catalog_game_id IN (?1,?2) ORDER BY match.catalog_game_id").unwrap();
            statement
                .query_map(ids, |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
                .unwrap()
                .collect::<Result<_, _>>()
                .unwrap()
        };
        assert_eq!(matches.len(), 2);
        assert!(matches
            .iter()
            .all(|entry| entry.1 == AUTHORITY && entry.2 >= 90));
        assert_eq!(matches[0].0, ids[0]);
        assert_eq!(matches[1].0, ids[1]);
        let canonical_id: String = connection
            .query_row(
                "SELECT canonical_game_id FROM catalog_game_matches WHERE catalog_game_id=?1",
                [ids[0]],
                |row| row.get(0),
            )
            .unwrap();
        for (index, id) in ids.iter().enumerate() {
            connection.execute("INSERT INTO download_sources(id,game_id,name,source_type,uri,available) VALUES(?1,?2,?3,'http',?4,1)",
                params![format!("game-db-source-{index}"), id, format!("Source {index}"), format!("https://example.invalid/{index}")]).unwrap();
        }
        drop(connection);
        let options = options(&canonical_id).unwrap();
        assert_eq!(options.releases.len(), 2);
        assert_eq!(options.sources.len(), 2);
        assert!(ids.iter().all(|id| options
            .sources
            .iter()
            .any(|option| option.source.game_id == *id)));
        let connection = DatabaseService::get_connection().unwrap();
        connection
            .execute(
                "DELETE FROM download_sources WHERE id LIKE 'game-db-source-%'",
                [],
            )
            .unwrap();
        for id in ids {
            connection
                .execute("DELETE FROM games WHERE id=?1", [id])
                .unwrap();
        }
        connection
            .execute(
                "DELETE FROM canonical_games WHERE normalized_title='canonical fixture'",
                [],
            )
            .unwrap();
    }
}

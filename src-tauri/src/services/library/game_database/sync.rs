use super::{dat::parse_dat, repository::import_releases};
use super::{now, AUTHORITY, LICENSE};
use crate::{errors::EmuBoxError, services::db_service::DatabaseService};
use reqwest::header::{ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED};
use rusqlite::{params, OptionalExtension};
use std::{io::Read, time::Duration};

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

fn check_due(checked_at: i64, failed: bool, current_time: i64) -> bool {
    current_time.saturating_sub(checked_at)
        >= if failed {
            ERROR_RETRY_INTERVAL
        } else {
            SUCCESS_CHECK_INTERVAL
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn refresh_intervals_remain_bounded() {
        assert!(!check_due(1_000, false, 1_000 + SUCCESS_CHECK_INTERVAL - 1));
        assert!(check_due(1_000, false, 1_000 + SUCCESS_CHECK_INTERVAL));
        assert!(!check_due(1_000, true, 1_000 + ERROR_RETRY_INTERVAL - 1));
        assert!(check_due(1_000, true, 1_000 + ERROR_RETRY_INTERVAL));
    }
}

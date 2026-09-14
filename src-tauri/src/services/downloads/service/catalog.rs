use super::DownloadService;
use crate::errors::EmuBoxError;
use crate::models::{DownloadSource, DownloadSourceType};
use crate::services::{db_service::DatabaseService, game_service::GameService};
use crate::services::{game_service::CatalogEntry, paths};
use rusqlite::{params, OptionalExtension};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, fs, sync::Mutex};
static CATALOG_IMPORT: Mutex<()> = Mutex::new(());

impl DownloadService {
    pub fn import_from_json(json_content: &str) -> Result<Vec<DownloadSource>, EmuBoxError> {
        Self::import_manifest_content(json_content, None)
    }

    pub fn import_from_url(url_str: &str) -> Result<Vec<DownloadSource>, EmuBoxError> {
        Self::import_fetched(Self::fetch_manifest(url_str)?)
    }

    fn fetch_manifest(
        url_str: &str,
    ) -> Result<crate::services::manifest_cache::FetchResult, EmuBoxError> {
        let connection = DatabaseService::get_connection()?;
        crate::services::manifest_cache::fetch(
            &connection,
            url_str,
            chrono::Utc::now().timestamp().max(0) as u64,
        )
    }

    fn import_fetched(
        fetched: crate::services::manifest_cache::FetchResult,
    ) -> Result<Vec<DownloadSource>, EmuBoxError> {
        use crate::services::manifest_cache::{self, FetchResult};
        match fetched {
            FetchResult::Fresh => Ok(Vec::new()),
            FetchResult::Unchanged(cache) => {
                manifest_cache::remember(&DatabaseService::get_connection()?, &cache)?;
                Ok(Vec::new())
            }
            FetchResult::Changed {
                content,
                filename,
                cache,
            } => Self::import_manifest_cached(&content, filename.as_deref(), Some(&cache)),
        }
    }

    pub fn import_link_file() -> Result<Vec<DownloadSource>, EmuBoxError> {
        Self::import_link_file_with_progress(|| {})
    }

    pub fn import_link_file_with_progress(
        mut updated: impl FnMut(),
    ) -> Result<Vec<DownloadSource>, EmuBoxError> {
        let Ok(_import_guard) = CATALOG_IMPORT.try_lock() else {
            return Ok(Vec::new());
        };
        let file = paths::download_links_file();
        let content = fs::read_to_string(&file).map_err(|error| {
            EmuBoxError::StorageUnavailable(format!("No se pudo leer {file}: {error}"))
        })?;
        let links: Vec<_> = content
            .lines()
            .enumerate()
            .filter_map(|(index, line)| {
                let link = line.split('#').next().unwrap_or("").trim();
                (!link.is_empty()).then_some((index + 1, link))
            })
            .collect();
        eprintln!(
            "[Catalog] {} manifiestos desde {file}; solo metadatos",
            links.len()
        );
        let mut sources = Vec::new();
        for chunk in links.chunks(4) {
            std::thread::scope(|scope| {
                let handles: Vec<_> = chunk
                    .iter()
                    .map(|(line, link)| (*line, scope.spawn(move || Self::fetch_manifest(link))))
                    .collect();
                for (line, handle) in handles {
                    match handle.join() {
                        Ok(Ok(fetched)) => match Self::import_fetched(fetched) {
                            Ok(mut imported) => {
                                eprintln!("[Catalog] linea {line}: {} fuentes modificadas (cache incremental)", imported.len());
                                if !imported.is_empty() {
                                    updated();
                                }
                                sources.append(&mut imported);
                            }
                            Err(error) => eprintln!("[Catalog] linea {line}: {error}"),
                        },
                        Ok(Err(error)) => eprintln!("[Catalog] linea {line}: {error}"),
                        Err(_) => eprintln!("[Catalog] linea {line}: importador interrumpido"),
                    }
                }
            });
        }
        Ok(sources)
    }

    pub fn import_manifest_content(
        manifest: &str,
        source_name_fallback: Option<&str>,
    ) -> Result<Vec<DownloadSource>, EmuBoxError> {
        Self::import_manifest_cached(manifest, source_name_fallback, None)
    }

    pub(super) fn import_manifest_cached(
        manifest: &str,
        source_name_fallback: Option<&str>,
        cache: Option<&crate::services::manifest_cache::CacheRecord>,
    ) -> Result<Vec<DownloadSource>, EmuBoxError> {
        let original: serde_json::Value =
            serde_json::from_str(manifest.trim_start_matches('\u{feff}'))
                .map_err(|e| EmuBoxError::InvalidConfiguration(format!("JSON inválido: {}", e)))?;
        let entries = original
            .get("downloads")
            .and_then(|value| value.as_array())
            .or_else(|| original.get("games").and_then(|value| value.as_array()))
            .or_else(|| original.as_array())
            .ok_or_else(|| {
                EmuBoxError::InvalidConfiguration(
                    "El manifiesto debe contener downloads[], games[] o un array".into(),
                )
            })?;
        let normalized: Vec<_> = entries
            .iter()
            .filter_map(crate::services::manifest_service::normalize)
            .collect();
        let skipped = entries.len() - normalized.len();
        if skipped > 0 {
            eprintln!("[Catalog] {skipped} entradas omitidas: titulo o URLs validas ausentes");
        }
        let parsed = serde_json::json!({"name":original.get("name"), "platform":original.get("platform"), "downloads":normalized});

        let mut sources = Vec::new();
        let mut connection = DatabaseService::get_connection()?;
        let transaction = connection
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        transaction.execute_batch("CREATE INDEX IF NOT EXISTS idx_download_sources_game_uri ON download_sources(game_id, uri);
            CREATE TABLE IF NOT EXISTS manifest_entry_cache (manifest_url TEXT NOT NULL, game_id TEXT NOT NULL,
                digest TEXT NOT NULL, PRIMARY KEY(manifest_url, game_id));
            CREATE TABLE IF NOT EXISTS manifest_sources (manifest_url TEXT NOT NULL, game_id TEXT NOT NULL,
                source_id TEXT NOT NULL, PRIMARY KEY(manifest_url, game_id, source_id));
            CREATE INDEX IF NOT EXISTS idx_manifest_sources_source ON manifest_sources(source_id);")
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        let mut previous = HashMap::<String, String>::new();
        let mut retired = Vec::<String>::new();
        if let Some(cache) = cache {
            let mut statement = transaction
                .prepare("SELECT game_id, digest FROM manifest_entry_cache WHERE manifest_url=?1")
                .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
            let rows = statement
                .query_map(params![cache.url], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
            for row in rows {
                let (game, digest) =
                    row.map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
                previous.insert(game, digest);
            }
        }

        // 1. Detectar formato estándar Hydra / downloads[]
        let downloads_array = parsed
            .get("downloads")
            .and_then(|d| d.as_array())
            .or_else(|| {
                // Si el objeto raíz es directamente un array de objetos con "uris" o "title"
                parsed.as_array().filter(|arr| {
                    arr.iter()
                        .any(|item| item.get("uris").is_some() && item.get("title").is_some())
                })
            });

        if let Some(downloads) = downloads_array {
            let manifest_name = parsed
                .get("name")
                .and_then(|n| n.as_str())
                .or(source_name_fallback);
            let manifest_platform = parsed.get("platform").and_then(|p| p.as_str());

            for item in downloads {
                let title = item
                    .get("title")
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .trim();
                if title.is_empty() {
                    continue;
                }

                let uris: Vec<String> = item
                    .get("uris")
                    .and_then(|u| u.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(str::to_string))
                            .collect()
                    })
                    .unwrap_or_default();

                if uris.is_empty() {
                    continue;
                }

                let item_platform = item.get("platform").and_then(|p| p.as_str());
                let platform = Self::infer_platform(
                    item_platform,
                    manifest_platform,
                    title,
                    &uris,
                    manifest_name,
                );

                let size_bytes = item
                    .get("sizeBytes")
                    .or_else(|| item.get("fileSize"))
                    .and_then(crate::models::download::parse_file_size_value);
                let release_year = item
                    .get("releaseYear")
                    .and_then(|value| value.as_u64())
                    .map(|year| year as u32);

                let game_id = item
                    .get("gameId")
                    .and_then(|v| v.as_str())
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("download-{}-{}", platform, slug(title)));

                let entry_digest = format!(
                    "{:x}",
                    Sha256::digest(serde_json::json!([item, platform]).to_string().as_bytes())
                );
                if let Some(cache) = cache {
                    if previous.remove(&game_id).as_deref() == Some(&entry_digest) {
                        continue;
                    }
                    let mut statement = transaction.prepare("SELECT source_id FROM manifest_sources WHERE manifest_url=?1 AND game_id=?2")
                        .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
                    let rows = statement
                        .query_map(params![cache.url, game_id], |row| row.get::<_, String>(0))
                        .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
                    for row in rows {
                        retired.push(
                            row.map_err(|error| {
                                EmuBoxError::StorageUnavailable(error.to_string())
                            })?,
                        );
                    }
                    transaction
                        .execute(
                            "DELETE FROM manifest_sources WHERE manifest_url=?1 AND game_id=?2",
                            params![cache.url, game_id],
                        )
                        .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
                }
                GameService::upsert_catalog_entry_on(
                    &transaction,
                    CatalogEntry {
                        id: game_id.clone(),
                        title: title.to_string(),
                        platform_id: platform.clone(),
                        platform_name: GameService::platform_name(&platform),
                        release_year,
                        genre: item
                            .get("genre")
                            .and_then(|v| v.as_str())
                            .map(str::to_string),
                        developer: item
                            .get("developer")
                            .and_then(|v| v.as_str())
                            .map(str::to_string),
                        publisher: item
                            .get("publisher")
                            .and_then(|v| v.as_str())
                            .map(str::to_string),
                        rating: item
                            .get("rating")
                            .and_then(|v| v.as_f64())
                            .map(|v| v as f32),
                        cover_image: item
                            .get("coverImage")
                            .or_else(|| item.get("cover"))
                            .and_then(|v| v.as_str())
                            .map(str::to_string),
                        backdrop_image: item
                            .get("backdropImage")
                            .and_then(|v| v.as_str())
                            .map(str::to_string),
                        description: item
                            .get("description")
                            .and_then(|v| v.as_str())
                            .map(str::to_string),
                    },
                )?;
                for uri in uris {
                    let source_type = match crate::services::manifest_service::source_access(&uri) {
                        Some("magnet") => DownloadSourceType::Magnet,
                        Some("torrent") => DownloadSourceType::Torrent,
                        Some("unsupported") | None => DownloadSourceType::Other,
                        _ => DownloadSourceType::Http,
                    };
                    let existing = transaction.query_row(
                        "SELECT id FROM download_sources WHERE game_id = ?1 AND uri = ?2 LIMIT 1",
                        params![game_id, uri], |row| row.get::<_, String>(0),
                    ).optional().map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
                    let identity =
                        serde_json::json!([game_id, uri, item.get("sourceId")]).to_string();
                    let source = DownloadSource {
                        id: existing.unwrap_or_else(|| {
                            format!("source-{:x}", Sha256::digest(identity.as_bytes()))
                        }),
                        game_id: game_id.clone(),
                        name: title.to_string(),
                        source_type,
                        uri,
                        size_bytes,
                        checksum: item
                            .get("checksum")
                            .and_then(|value| value.as_str())
                            .map(str::to_string),
                        available: item
                            .get("available")
                            .and_then(|value| value.as_bool())
                            .unwrap_or(true),
                    };
                    if let Some(cache) = cache {
                        transaction
                            .execute(
                                "INSERT OR IGNORE INTO manifest_sources VALUES (?1, ?2, ?3)",
                                params![cache.url, game_id, source.id],
                            )
                            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
                    }
                    sources.push(Self::create_source_on(&transaction, source)?);
                }
                if let Some(cache) = cache {
                    transaction
                        .execute(
                            "INSERT INTO manifest_entry_cache VALUES (?1, ?2, ?3)
                        ON CONFLICT(manifest_url, game_id) DO UPDATE SET digest=excluded.digest",
                            params![cache.url, game_id, entry_digest],
                        )
                        .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
                }
            }
            if let Some(cache) = cache {
                for game_id in previous.keys() {
                    let mut statement = transaction.prepare("SELECT source_id FROM manifest_sources WHERE manifest_url=?1 AND game_id=?2")
                        .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
                    for row in statement
                        .query_map(params![cache.url, game_id], |row| row.get::<_, String>(0))
                        .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?
                    {
                        retired.push(
                            row.map_err(|error| {
                                EmuBoxError::StorageUnavailable(error.to_string())
                            })?,
                        );
                    }
                    transaction
                        .execute(
                            "DELETE FROM manifest_sources WHERE manifest_url=?1 AND game_id=?2",
                            params![cache.url, game_id],
                        )
                        .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
                    transaction
                        .execute(
                            "DELETE FROM manifest_entry_cache WHERE manifest_url=?1 AND game_id=?2",
                            params![cache.url, game_id],
                        )
                        .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
                }
                for source in retired {
                    transaction
                        .execute(
                            "UPDATE download_sources SET available=0 WHERE id=?1
                        AND NOT EXISTS (SELECT 1 FROM manifest_sources WHERE source_id=?1)",
                            params![source],
                        )
                        .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
                }
                crate::services::manifest_cache::remember(&transaction, cache)?;
            }
            transaction
                .commit()
                .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
            return Ok(sources);
        }

        // 2. Formato legado (games[] o array plano de { platform, url, name })
        let entries = parsed
            .get("games")
            .and_then(|games| games.as_array())
            .cloned()
            .or_else(|| parsed.as_array().cloned())
            .ok_or_else(|| {
                EmuBoxError::InvalidConfiguration(
                    "El manifiesto debe contener downloads[] o games[]".to_string(),
                )
            })?;

        for entry in entries {
            let platform = entry.get("platform").and_then(|v| v.as_str()).unwrap_or("");
            if !Self::supported_platform(platform) {
                continue;
            }
            let download_uri = entry
                .get("url")
                .or_else(|| entry.get("uri"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if download_uri.is_empty() {
                continue;
            }
            let name = entry
                .get("name")
                .or_else(|| entry.get("title"))
                .and_then(|v| v.as_str())
                .map(str::to_string)
                .unwrap_or_else(|| "Untitled".to_string());
            let game_id = entry
                .get("gameId")
                .and_then(|v| v.as_str())
                .map(str::to_string)
                .unwrap_or_else(|| format!("download-{}-{}", platform, slug(&name)));

            let source = DownloadSource {
                id: entry
                    .get("sourceId")
                    .and_then(|v| v.as_str())
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("source-{}-{}", platform, slug(download_uri))),
                game_id: game_id.clone(),
                name: name.clone(),
                source_type: DownloadSourceType::Http,
                uri: download_uri.to_string(),
                size_bytes: entry.get("sizeBytes").and_then(|v| v.as_u64()).or_else(|| {
                    entry
                        .get("fileSize")
                        .and_then(crate::models::download::parse_file_size_value)
                }),
                checksum: entry
                    .get("checksum")
                    .and_then(|v| v.as_str())
                    .map(str::to_string),
                available: entry
                    .get("available")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true),
            };

            GameService::upsert_catalog_entry_on(
                &transaction,
                CatalogEntry {
                    id: game_id.clone(),
                    title: name,
                    platform_id: platform.to_string(),
                    platform_name: GameService::platform_name(platform),
                    release_year: entry
                        .get("releaseYear")
                        .and_then(|v| v.as_u64())
                        .map(|v| v as u32),
                    genre: entry
                        .get("genre")
                        .and_then(|v| v.as_str())
                        .map(str::to_string),
                    developer: entry
                        .get("developer")
                        .and_then(|v| v.as_str())
                        .map(str::to_string),
                    publisher: entry
                        .get("publisher")
                        .and_then(|v| v.as_str())
                        .map(str::to_string),
                    rating: entry
                        .get("rating")
                        .and_then(|v| v.as_f64())
                        .map(|v| v as f32),
                    cover_image: entry
                        .get("coverImage")
                        .and_then(|v| v.as_str())
                        .map(str::to_string),
                    backdrop_image: entry
                        .get("backdropImage")
                        .and_then(|v| v.as_str())
                        .map(str::to_string),
                    description: entry
                        .get("description")
                        .and_then(|v| v.as_str())
                        .map(str::to_string),
                },
            )?;
            sources.push(Self::create_source_on(&transaction, source)?);
        }

        transaction
            .commit()
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        Ok(sources)
    }
}

fn slug(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
}

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Instant;
use reqwest::blocking::Client;
use sha2::{Digest, Sha256};
use rusqlite::{params, OptionalExtension};
use crate::errors::EmuBoxError;
use crate::models::{CreateDownloadRequest, DownloadJob, DownloadSource, DownloadSourceType, DownloadStatus};
use crate::services::db_service::DatabaseService;
use crate::services::game_service::{CatalogEntry, GameService};
use crate::services::paths;

struct RuntimeControl {
    paused: Arc<AtomicBool>,
    cancelled: Arc<AtomicBool>,
}

static ACTIVE_JOBS: OnceLock<Mutex<HashMap<String, RuntimeControl>>> = OnceLock::new();
static CATALOG_IMPORT: Mutex<()> = Mutex::new(());

fn active_jobs() -> &'static Mutex<HashMap<String, RuntimeControl>> {
    ACTIVE_JOBS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub struct DownloadService;

impl DownloadService {
    pub fn import_from_json(json_content: &str) -> Result<Vec<DownloadSource>, EmuBoxError> {
        Self::import_manifest_content(json_content, None)
    }

    pub fn import_from_url(url_str: &str) -> Result<Vec<DownloadSource>, EmuBoxError> {
        let (manifest, filename) = Self::fetch_manifest(url_str)?;
        Self::import_manifest_content(&manifest, filename.as_deref())
    }

    fn fetch_manifest(url_str: &str) -> Result<(String, Option<String>), EmuBoxError> {
        let url = reqwest::Url::parse(url_str)
            .map_err(|e| EmuBoxError::InvalidConfiguration(format!("URL inválida: {}", e)))?;
        if !matches!(url.scheme(), "http" | "https") {
            return Err(EmuBoxError::InvalidConfiguration("Solo se aceptan URLs HTTP/HTTPS".to_string()));
        }
        let client = Client::builder()
            .connect_timeout(std::time::Duration::from_secs(5))
            .timeout(std::time::Duration::from_secs(25))
            .build().map_err(|error| EmuBoxError::Unknown(error.to_string()))?;
        let response = client.get(url_str).send()
            .and_then(|response| response.error_for_status())
            .map_err(|e| EmuBoxError::Unknown(format!("No se pudo descargar el manifiesto: {}", e)))?;
        let mut manifest_text = String::new();
        response.take(32 * 1024 * 1024 + 1).read_to_string(&mut manifest_text)
            .map_err(|error| EmuBoxError::Unknown(format!("No se pudo leer el manifiesto: {error}")))?;
        if manifest_text.len() > 32 * 1024 * 1024 {
            return Err(EmuBoxError::InvalidConfiguration("El manifiesto supera 32 MiB".into()));
        }
        let filename_hint = url.path_segments().and_then(|mut segments| segments.rfind(|segment| !segment.is_empty())).map(str::to_owned);
        Ok((manifest_text, filename_hint))
    }

    pub fn import_link_file() -> Result<Vec<DownloadSource>, EmuBoxError> {
        Self::import_link_file_with_progress(|| {})
    }

    pub fn import_link_file_with_progress(mut updated: impl FnMut()) -> Result<Vec<DownloadSource>, EmuBoxError> {
        let Ok(_import_guard) = CATALOG_IMPORT.try_lock() else { return Ok(Vec::new()); };
        let file = paths::download_links_file();
        let content = fs::read_to_string(&file)
            .map_err(|error| EmuBoxError::StorageUnavailable(format!("No se pudo leer {file}: {error}")))?;
        let links: Vec<_> = content.lines().enumerate().filter_map(|(index, line)| {
            let link = line.split('#').next().unwrap_or("").trim();
            (!link.is_empty()).then_some((index + 1, link))
        }).collect();
        eprintln!("[Catalog] {} manifiestos desde {file}; solo metadatos", links.len());
        let mut sources = Vec::new();
        for chunk in links.chunks(4) {
            std::thread::scope(|scope| {
                let handles: Vec<_> = chunk.iter().map(|(line, link)| {
                    (*line, scope.spawn(move || Self::fetch_manifest(link)))
                }).collect();
                for (line, handle) in handles {
                    match handle.join() {
                        Ok(Ok((manifest, filename))) => {
                            match Self::import_manifest_content(&manifest, filename.as_deref()) {
                                Ok(mut imported) => {
                                    eprintln!("[Catalog] linea {line}: {} fuentes registradas", imported.len());
                                    sources.append(&mut imported);
                                    updated();
                                }
                                Err(error) => eprintln!("[Catalog] linea {line}: {error}"),
                            }
                        }
                        Ok(Err(error)) => eprintln!("[Catalog] linea {line}: {error}"),
                        Err(_) => eprintln!("[Catalog] linea {line}: importador interrumpido"),
                    }
                }
            });
        }
        Ok(sources)
    }

    pub fn import_manifest_content(manifest: &str, source_name_fallback: Option<&str>) -> Result<Vec<DownloadSource>, EmuBoxError> {
        let original: serde_json::Value = serde_json::from_str(manifest.trim_start_matches('\u{feff}'))
            .map_err(|e| EmuBoxError::InvalidConfiguration(format!("JSON inválido: {}", e)))?;
        let entries = original.get("downloads").and_then(|value| value.as_array())
            .or_else(|| original.get("games").and_then(|value| value.as_array()))
            .or_else(|| original.as_array())
            .ok_or_else(|| EmuBoxError::InvalidConfiguration("El manifiesto debe contener downloads[], games[] o un array".into()))?;
        let normalized: Vec<_> = entries.iter().filter_map(super::manifest_service::normalize).collect();
        let skipped = entries.len() - normalized.len();
        if skipped > 0 { eprintln!("[Catalog] {skipped} entradas omitidas: titulo o URLs validas ausentes"); }
        let parsed = serde_json::json!({"name":original.get("name"), "platform":original.get("platform"), "downloads":normalized});

        let mut sources = Vec::new();
        let mut connection = DatabaseService::get_connection()?;
        let transaction = connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        transaction.execute_batch("CREATE INDEX IF NOT EXISTS idx_download_sources_game_uri ON download_sources(game_id, uri);")
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;

        // 1. Detectar formato estándar Hydra / downloads[]
        let downloads_array = parsed.get("downloads").and_then(|d| d.as_array())
            .or_else(|| {
                // Si el objeto raíz es directamente un array de objetos con "uris" o "title"
                parsed.as_array().filter(|arr| arr.iter().any(|item| item.get("uris").is_some() && item.get("title").is_some()))
            });

        if let Some(downloads) = downloads_array {
            let manifest_name = parsed.get("name").and_then(|n| n.as_str()).or(source_name_fallback);
            let manifest_platform = parsed.get("platform").and_then(|p| p.as_str());

            for item in downloads {
                let title = item.get("title").and_then(|t| t.as_str()).unwrap_or("").trim();
                if title.is_empty() {
                    continue;
                }

                let uris: Vec<String> = item.get("uris")
                    .and_then(|u| u.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
                    .unwrap_or_default();

                if uris.is_empty() {
                    continue;
                }

                let item_platform = item.get("platform").and_then(|p| p.as_str());
                let platform = Self::infer_platform(item_platform, manifest_platform, title, &uris, manifest_name);

                let size_bytes = item.get("sizeBytes").or_else(|| item.get("fileSize")).and_then(crate::models::download::parse_file_size_value);
                let release_year = item.get("releaseYear").and_then(|value| value.as_u64()).map(|year| year as u32);

                let game_id = item.get("gameId").and_then(|v| v.as_str()).map(str::to_string)
                    .unwrap_or_else(|| format!("download-{}-{}", platform, slug(title)));

                GameService::upsert_catalog_entry_on(&transaction, CatalogEntry {
                    id: game_id.clone(),
                    title: title.to_string(),
                    platform_id: platform.clone(),
                    platform_name: GameService::platform_name(&platform),
                    release_year,
                    genre: item.get("genre").and_then(|v| v.as_str()).map(str::to_string),
                    developer: item.get("developer").and_then(|v| v.as_str()).map(str::to_string),
                    publisher: item.get("publisher").and_then(|v| v.as_str()).map(str::to_string),
                    rating: item.get("rating").and_then(|v| v.as_f64()).map(|v| v as f32),
                    cover_image: item.get("coverImage").or_else(|| item.get("cover")).and_then(|v| v.as_str()).map(str::to_string),
                    backdrop_image: item.get("backdropImage").and_then(|v| v.as_str()).map(str::to_string),
                    description: item.get("description").and_then(|v| v.as_str()).map(str::to_string),
                })?;
                for uri in uris {
                    let source_type = match super::manifest_service::source_access(&uri) {
                        Some("magnet") => DownloadSourceType::Magnet,
                        Some("torrent") => DownloadSourceType::Torrent,
                        _ => DownloadSourceType::Http,
                    };
                    let existing = transaction.query_row(
                        "SELECT id FROM download_sources WHERE game_id = ?1 AND uri = ?2 LIMIT 1",
                        params![game_id, uri], |row| row.get::<_, String>(0),
                    ).optional().map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
                    let identity = serde_json::json!([game_id, uri, item.get("sourceId")]).to_string();
                    let source = DownloadSource {
                        id: existing.unwrap_or_else(|| format!("source-{:x}", Sha256::digest(identity.as_bytes()))),
                        game_id: game_id.clone(), name: title.to_string(), source_type, uri, size_bytes,
                        checksum: item.get("checksum").and_then(|value| value.as_str()).map(str::to_string),
                        available: item.get("available").and_then(|value| value.as_bool()).unwrap_or(true),
                    };
                    sources.push(Self::create_source_on(&transaction, source)?);
                }
            }
            transaction.commit().map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
            return Ok(sources);
        }

        // 2. Formato legado (games[] o array plano de { platform, url, name })
        let entries = parsed.get("games").and_then(|games| games.as_array()).cloned()
            .or_else(|| parsed.as_array().cloned())
            .ok_or_else(|| EmuBoxError::InvalidConfiguration("El manifiesto debe contener downloads[] o games[]".to_string()))?;

        for entry in entries {
            let platform = entry.get("platform").and_then(|v| v.as_str()).unwrap_or("");
            if !Self::supported_platform(platform) {
                continue;
            }
            let download_uri = entry.get("url").or_else(|| entry.get("uri")).and_then(|v| v.as_str()).unwrap_or("");
            if download_uri.is_empty() {
                continue;
            }
            let name = entry.get("name").or_else(|| entry.get("title")).and_then(|v| v.as_str()).map(str::to_string)
                .unwrap_or_else(|| "Untitled".to_string());
            let game_id = entry.get("gameId").and_then(|v| v.as_str()).map(str::to_string)
                .unwrap_or_else(|| format!("download-{}-{}", platform, slug(&name)));

            let source = DownloadSource {
                id: entry.get("sourceId").and_then(|v| v.as_str()).map(str::to_string).unwrap_or_else(|| format!("source-{}-{}", platform, slug(download_uri))),
                game_id: game_id.clone(),
                name: name.clone(),
                source_type: DownloadSourceType::Http,
                uri: download_uri.to_string(),
                size_bytes: entry.get("sizeBytes").and_then(|v| v.as_u64()).or_else(|| entry.get("fileSize").and_then(crate::models::download::parse_file_size_value)),
                checksum: entry.get("checksum").and_then(|v| v.as_str()).map(str::to_string),
                available: entry.get("available").and_then(|v| v.as_bool()).unwrap_or(true),
            };

            GameService::upsert_catalog_entry_on(&transaction, CatalogEntry {
                id: game_id.clone(),
                title: name,
                platform_id: platform.to_string(),
                platform_name: GameService::platform_name(platform),
                release_year: entry.get("releaseYear").and_then(|v| v.as_u64()).map(|v| v as u32),
                genre: entry.get("genre").and_then(|v| v.as_str()).map(str::to_string),
                developer: entry.get("developer").and_then(|v| v.as_str()).map(str::to_string),
                publisher: entry.get("publisher").and_then(|v| v.as_str()).map(str::to_string),
                rating: entry.get("rating").and_then(|v| v.as_f64()).map(|v| v as f32),
                cover_image: entry.get("coverImage").and_then(|v| v.as_str()).map(str::to_string),
                backdrop_image: entry.get("backdropImage").and_then(|v| v.as_str()).map(str::to_string),
                description: entry.get("description").and_then(|v| v.as_str()).map(str::to_string),
            })?;
            sources.push(Self::create_source_on(&transaction, source)?);
        }

        transaction.commit().map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        Ok(sources)
    }

    pub fn infer_platform(
        item_platform: Option<&str>,
        manifest_platform: Option<&str>,
        title: &str,
        uris: &[String],
        manifest_hint: Option<&str>,
    ) -> String {
        if let Some(p) = item_platform {
            let p_lower = p.to_ascii_lowercase();
            if Self::supported_platform(&p_lower) {
                return p_lower;
            }
        }
        if let Some(p) = manifest_platform {
            let p_lower = p.to_ascii_lowercase();
            if Self::supported_platform(&p_lower) {
                return p_lower;
            }
        }

        let title_lower = title.to_ascii_lowercase();

        if title_lower.contains("[pc]") || title_lower.contains("(pc)") || title_lower.contains("steamrip") || title_lower.contains("gog") {
            return "pc".to_string();
        }
        if title_lower.contains("[ps3]") || title_lower.contains("(ps3)") || title_lower.contains("ps3") || title_lower.contains("rpcs3") {
            return "ps3".to_string();
        }
        if title_lower.contains("[ps2]") || title_lower.contains("(ps2)") || title_lower.contains("pcsx2") {
            return "ps2".to_string();
        }
        if title_lower.contains("[ps1]") || title_lower.contains("(ps1)") || title_lower.contains("[psx]") || title_lower.contains("(psx)") || title_lower.contains("duckstation") {
            return "ps1".to_string();
        }
        if title_lower.contains("[psp]") || title_lower.contains("(psp)") || title_lower.contains("ppsspp") {
            return "psp".to_string();
        }
        if title_lower.contains("[wiiu]") || title_lower.contains("(wiiu)") || title_lower.contains("wii u") || title_lower.contains("cemu") {
            return "wiiu".to_string();
        }
        if title_lower.contains("[wii]") || title_lower.contains("(wii)") {
            return "wii".to_string();
        }
        if title_lower.contains("[gamecube]") || title_lower.contains("(gamecube)") || title_lower.contains("[gcn]") || title_lower.contains("dolphin") {
            return "gamecube".to_string();
        }
        if title_lower.contains("[snes]") || title_lower.contains("(snes)") || title_lower.contains("super nintendo") {
            return "snes".to_string();
        }
        if title_lower.contains("[gba]") || title_lower.contains("(gba)") || title_lower.contains("game boy advance") || title_lower.contains("mgba") {
            return "gba".to_string();
        }
        if title_lower.contains("[n64]") || title_lower.contains("(n64)") || title_lower.contains("nintendo 64") {
            return "n64".to_string();
        }
        if title_lower.contains("[nds]") || title_lower.contains("(nds)") || title_lower.contains("nintendo ds") || title_lower.contains("melonds") {
            return "nds".to_string();
        }
        if title_lower.contains("[genesis]") || title_lower.contains("(genesis)") || title_lower.contains("megadrive") || title_lower.contains("mega drive") {
            return "genesis".to_string();
        }
        if title_lower.contains("[dreamcast]") || title_lower.contains("(dreamcast)") || title_lower.contains("flycast") {
            return "dreamcast".to_string();
        }
        if title_lower.contains("[arcade]") || title_lower.contains("(arcade)") || title_lower.contains("mame") {
            return "arcade".to_string();
        }

        for uri in uris {
            let u_lower = uri.to_ascii_lowercase();
            if u_lower.contains(".pkg") {
                return "ps3".to_string();
            }
            if u_lower.contains(".sfc") || u_lower.contains(".smc") {
                return "snes".to_string();
            }
            if u_lower.contains(".gba") {
                return "gba".to_string();
            }
            if u_lower.contains(".z64") || u_lower.contains(".n64") || u_lower.contains(".v64") {
                return "n64".to_string();
            }
            if u_lower.contains(".nds") {
                return "nds".to_string();
            }
            if u_lower.contains(".cdi") || u_lower.contains(".gdi") {
                return "dreamcast".to_string();
            }
            if u_lower.contains(".rvz") || u_lower.contains(".gcm") || u_lower.contains(".ciso") {
                return "gamecube".to_string();
            }
            if u_lower.contains(".wua") || u_lower.contains(".wux") || u_lower.contains(".rpx") {
                return "wiiu".to_string();
            }
            if u_lower.contains(".pbp") {
                return "psp".to_string();
            }
            if u_lower.contains("steamrip") || u_lower.contains("gog") || u_lower.contains(".exe") {
                return "pc".to_string();
            }
        }

        if let Some(hint) = manifest_hint {
            let h_lower = hint.to_ascii_lowercase();
            if h_lower.contains("psx-roms") || h_lower.contains("ps1") {
                return "ps1".to_string();
            }
            if h_lower.contains("ps2") {
                return "ps2".to_string();
            }
            if h_lower.contains("ps3") {
                return "ps3".to_string();
            }
            if h_lower.contains("psp") {
                return "psp".to_string();
            }
            if h_lower.contains("snes") {
                return "snes".to_string();
            }
            if h_lower.contains("gba") {
                return "gba".to_string();
            }
            if h_lower.contains("n64") {
                return "n64".to_string();
            }
            if h_lower.contains("nds") {
                return "nds".to_string();
            }
            if h_lower.contains("linux") || h_lower.contains("pc") || h_lower.contains("repack") {
                return "pc".to_string();
            }
            if h_lower.contains("psx") {
                return "ps1".to_string();
            }
        }

        "pc".to_string()
    }

    fn supported_platform(value: &str) -> bool {
        matches!(value, "ps1" | "ps2" | "ps3" | "psp" | "gamecube" | "wii" | "wiiu" | "n64" | "snes" | "gba" | "nds" | "genesis" | "dreamcast" | "arcade" | "pc" | "linux" | "all")
    }

    pub fn create_source(source: DownloadSource) -> Result<DownloadSource, EmuBoxError> {
        let connection = DatabaseService::get_connection()?;
        Self::create_source_on(&connection, source)
    }

    fn create_source_on(conn: &rusqlite::Connection, source: DownloadSource) -> Result<DownloadSource, EmuBoxError> {
        if source.id.trim().is_empty() || source.game_id.trim().is_empty() || source.name.trim().is_empty() {
            return Err(EmuBoxError::InvalidConfiguration("La fuente necesita id, gameId y nombre".to_string()));
        }
        let source_type_str = match &source.source_type {
            DownloadSourceType::Http => "http",
            DownloadSourceType::Torrent => "torrent",
            DownloadSourceType::Magnet => "magnet",
        };
        conn.execute(
            "INSERT INTO download_sources (id, game_id, name, source_type, uri, size_bytes, checksum, available)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET game_id = excluded.game_id, name = excluded.name, source_type = excluded.source_type, uri = excluded.uri, size_bytes = excluded.size_bytes, checksum = excluded.checksum, available = excluded.available;",
            params![source.id, source.game_id, source.name, source_type_str, source.uri, source.size_bytes, source.checksum, if source.available { 1 } else { 0 }],
        ).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
        Ok(source)
    }

    pub fn create_job(request: CreateDownloadRequest) -> Result<DownloadJob, EmuBoxError> {
        let source = request.source;
        let option = super::manifest_service::source_option(source.clone());
        if !option.downloadable {
            return Err(EmuBoxError::InvalidConfiguration(option.reason.unwrap_or_else(|| "Fuente no disponible".into())));
        }
        if source.game_id != request.game_id {
            return Err(EmuBoxError::InvalidConfiguration("La fuente no pertenece al juego solicitado".into()));
        }
        Self::create_source(source.clone())?;
        let _ = GameService::upsert_catalog_entry(CatalogEntry {
            id: request.game_id.clone(),
            title: source.name.clone(),
            platform_id: request.platform.clone(),
            platform_name: GameService::platform_name(&request.platform),
            release_year: None,
            genre: None,
            developer: None,
            publisher: None,
            rating: None,
            cover_image: None,
            backdrop_image: None,
            description: None,
        });
        let destination = Self::destination_path(&request.platform, &source.uri)?;
        let existing = DatabaseService::get_connection()?.query_row(
            "SELECT id FROM download_jobs WHERE source_id = ?1 AND status IN ('queued', 'downloading', 'paused', 'completed') ORDER BY rowid DESC LIMIT 1",
            params![source.id],
            |row| row.get::<_, String>(0),
        ).ok();
        if let Some(existing_id) = existing {
            return Self::get_job(&existing_id)?.ok_or_else(|| EmuBoxError::NotFound(existing_id));
        }
        let id = format!("download-{}", uuid_like());
        let conn = DatabaseService::get_connection()?;
        conn.execute(
            "INSERT INTO download_jobs (id, game_id, source_id, platform, destination_path, status, total_bytes)
             VALUES (?1, ?2, ?3, ?4, ?5, 'queued', ?6);",
            params![id, request.game_id, source.id, request.platform, destination.to_string_lossy().to_string(), source.size_bytes],
        ).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
        Self::get_job(&id)?.ok_or_else(|| EmuBoxError::Unknown("No se pudo crear el trabajo de descarga".to_string()))
    }

    pub fn list_jobs() -> Result<Vec<DownloadJob>, EmuBoxError> {
        let conn = DatabaseService::get_connection()?;
        let mut stmt = conn.prepare("SELECT id, game_id, source_id, platform, destination_path, status, progress, downloaded_bytes, total_bytes, speed_bytes_per_second, error FROM download_jobs ORDER BY rowid DESC")
            .map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
        let rows = stmt.query_map([], Self::row_to_job).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
        Ok(rows.flatten().collect())
    }

    pub fn import_and_start() -> Result<Vec<DownloadJob>, EmuBoxError> {
        Self::import_link_file()?;
        Self::list_jobs()
    }

    /// Descarga un juego de catálogo a partir de su `gameId`, reutilizando la fuente
    /// importada desde el manifiesto o creada manualmente para ese juego.
    pub fn download_game(game_id: String) -> Result<DownloadJob, EmuBoxError> {
        Self::download_game_from_source(game_id, None)
    }

    pub fn list_sources(game_id: &str) -> Result<Vec<super::manifest_service::SourceOption>, EmuBoxError> {
        let conn = DatabaseService::get_connection()?;
        let mut statement = conn.prepare("SELECT id, game_id, name, uri, size_bytes, checksum, available FROM download_sources WHERE game_id = ?1 ORDER BY rowid")
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        let rows = statement.query_map(params![game_id], |row| {
            let uri: String = row.get(3)?;
            let source_type = match super::manifest_service::source_access(&uri) {
                Some("magnet") => DownloadSourceType::Magnet,
                Some("torrent") => DownloadSourceType::Torrent,
                _ => DownloadSourceType::Http,
            };
            Ok(super::manifest_service::source_option(DownloadSource {
                id: row.get(0)?, game_id: row.get(1)?, name: row.get(2)?, uri, source_type,
                size_bytes: row.get(4)?, checksum: row.get(5)?, available: row.get::<_, i64>(6)? != 0,
            }))
        }).map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))
    }

    pub fn download_game_from_source(game_id: String, source_id: Option<String>) -> Result<DownloadJob, EmuBoxError> {
        let conn = DatabaseService::get_connection()?;
        let platform: String = conn.query_row(
            "SELECT platform_id FROM games WHERE id = ?1",
            params![game_id],
            |row| row.get(0),
        ).map_err(|_| EmuBoxError::NotFound(format!("Juego no encontrado en el catálogo: {}", game_id)))?;

        let sources = Self::list_sources(&game_id)?;
        let chosen = if let Some(source_id) = source_id {
            sources.into_iter().find(|option| option.source.id == source_id)
                .ok_or_else(|| EmuBoxError::NotFound("La fuente no pertenece a este juego".into()))?
        } else {
            if sources.len() > 1 {
                return Err(EmuBoxError::InvalidConfiguration("Selecciona una fuente; las URLs pueden ser partes o versiones distintas".into()));
            }
            sources.into_iter().next().ok_or_else(|| EmuBoxError::NotFound("No hay fuentes registradas para este juego".into()))?
        };
        if !chosen.downloadable {
            return Err(EmuBoxError::InvalidConfiguration(chosen.reason.unwrap_or_else(|| "Fuente no disponible".into())));
        }
        let source = chosen.source;

        let job = Self::create_job(CreateDownloadRequest { game_id, platform, source })?;
        Self::start(job.id)
    }

    pub fn get_job(id: &str) -> Result<Option<DownloadJob>, EmuBoxError> {
        let conn = DatabaseService::get_connection()?;
        let mut stmt = conn.prepare("SELECT id, game_id, source_id, platform, destination_path, status, progress, downloaded_bytes, total_bytes, speed_bytes_per_second, error FROM download_jobs WHERE id = ?1")
            .map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
        let mut rows = stmt.query_map(params![id], Self::row_to_job).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
        rows.next().transpose().map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))
    }

    pub fn start(id: String) -> Result<DownloadJob, EmuBoxError> {
        let job = Self::get_job(&id)?.ok_or_else(|| EmuBoxError::NotFound(format!("Descarga no encontrada: {}", id)))?;
        if !active_jobs().lock().unwrap().is_empty() {
            return Ok(job);
        }
        let (source_uri, checksum): (String, Option<String>) = DatabaseService::get_connection()?.query_row("SELECT uri, checksum FROM download_sources WHERE id = ?1", params![job.source_id], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
        if !matches!(super::manifest_service::source_access(&source_uri), Some("http" | "unverified_http")) {
            return Err(EmuBoxError::InvalidConfiguration("Esta fuente requiere un conector de alojamiento o BitTorrent; no es una descarga HTTP directa".into()));
        }
        let paused = Arc::new(AtomicBool::new(false));
        let cancelled = Arc::new(AtomicBool::new(false));
        active_jobs().lock().unwrap().insert(id.clone(), RuntimeControl { paused: paused.clone(), cancelled: cancelled.clone() });
        Self::update_status(&id, DownloadStatus::Downloading, None)?;
        let worker_id = id.clone();
        thread::spawn(move || {
            let result = Self::run_http(worker_id.clone(), job.game_id.clone(), source_uri, checksum, job.destination_path, paused, cancelled);
            active_jobs().lock().unwrap().remove(&worker_id);
            if let Err(error) = result {
                let _ = Self::update_status(&worker_id, DownloadStatus::Failed, Some(error.to_string()));
            }
            Self::start_next_queued();
        });
        Self::get_job(&id)?.ok_or(EmuBoxError::NotFound(id))
    }

    pub fn pause(id: &str) -> Result<DownloadJob, EmuBoxError> {
        if let Some(control) = active_jobs().lock().unwrap().get(id) { control.paused.store(true, Ordering::Relaxed); }
        Self::update_status(id, DownloadStatus::Paused, None)?;
        Self::get_job(id)?.ok_or_else(|| EmuBoxError::NotFound(id.to_string()))
    }

    pub fn resume(id: String) -> Result<DownloadJob, EmuBoxError> {
        if let Some(control) = active_jobs().lock().unwrap().get(&id) { control.paused.store(false, Ordering::Relaxed); return Self::get_job(&id)?.ok_or(EmuBoxError::NotFound(id)); }
        Self::start(id)
    }

    pub fn cancel(id: &str) -> Result<DownloadJob, EmuBoxError> {
        if let Some(control) = active_jobs().lock().unwrap().get(id) { control.cancelled.store(true, Ordering::Relaxed); }
        Self::update_status(id, DownloadStatus::Cancelled, None)?;
        Self::get_job(id)?.ok_or_else(|| EmuBoxError::NotFound(id.to_string()))
    }

    fn run_http(id: String, game_id: String, uri: String, checksum: Option<String>, destination: String, paused: Arc<AtomicBool>, cancelled: Arc<AtomicBool>) -> Result<(), EmuBoxError> {
        let destination = PathBuf::from(destination);
        let parent = destination.parent().ok_or_else(|| EmuBoxError::InvalidConfiguration("Destino inválido".to_string()))?;
        fs::create_dir_all(parent).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
        let partial_name = destination.file_name().and_then(|name| name.to_str()).unwrap_or("download.bin");
        let cache_root = paths::downloads_cache_dir();
        let partial = Path::new(&cache_root).join(format!("{}.part", partial_name));
        fs::create_dir_all(&cache_root).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
        let existing = partial.metadata().map(|m| m.len()).unwrap_or(0);
        let client = Client::builder().connect_timeout(std::time::Duration::from_secs(15))
            .build().map_err(request_error)?;
        let mut request = client.get(&uri);
        if existing > 0 { request = request.header(reqwest::header::RANGE, format!("bytes={}-", existing)); }
        let mut response = request.send().map_err(request_error)?;
        let mut downloaded = if existing > 0 && response.status() == reqwest::StatusCode::PARTIAL_CONTENT { existing } else { 0 };
        if existing > 0 && downloaded == 0 { response = client.get(&uri).send().map_err(request_error)?; }
        if !response.status().is_success() { return Err(EmuBoxError::Unknown(format!("HTTP {}", response.status()))); }
        if response.headers().get(reqwest::header::CONTENT_TYPE).and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.to_ascii_lowercase().contains("text/html")) {
            return Err(EmuBoxError::InvalidConfiguration("La fuente devuelve una pagina web, no un archivo descargable directo.".into()));
        }
        let total = response.content_length().map(|size| size + downloaded);
        let mut file = if downloaded > 0 { OpenOptions::new().append(true).open(&partial) } else { File::create(&partial) }
            .map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
        let started = Instant::now();
        let mut buffer = [0u8; 64 * 1024];
        loop {
            if cancelled.load(Ordering::Relaxed) { return Ok(()); }
            if paused.load(Ordering::Relaxed) { thread::sleep(std::time::Duration::from_millis(100)); continue; }
            let read = response.read(&mut buffer).map_err(|e| EmuBoxError::Unknown(e.to_string()))?;
            if read == 0 { break; }
            file.write_all(&buffer[..read]).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
            downloaded += read as u64;
            let speed = (downloaded as f64 / started.elapsed().as_secs_f64().max(0.001)) as u64;
            Self::update_progress(&id, downloaded, total, speed)?;
        }
        fs::rename(&partial, &destination).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
        if let Some(expected) = checksum {
            let mut file = File::open(&destination).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
            let mut hasher = Sha256::new();
            let mut hash_buffer = [0u8; 64 * 1024];
            loop {
                let read = file.read(&mut hash_buffer).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
                if read == 0 { break; }
                hasher.update(&hash_buffer[..read]);
            }
            let actual = format!("{:x}", hasher.finalize());
            if actual != expected.trim().to_ascii_lowercase() {
                let _ = fs::remove_file(&destination);
                return Err(EmuBoxError::Unknown("El checksum SHA-256 de la descarga no coincide".to_string()));
            }
        }
        Self::update_status(&id, DownloadStatus::Completed, None)?;
        let file_size = fs::metadata(&destination).map(|metadata| metadata.len()).unwrap_or(0);
        let _ = GameService::mark_installed(&game_id, &destination.to_string_lossy(), file_size);
        Ok(())
    }

    fn destination_path(platform: &str, uri: &str) -> Result<PathBuf, EmuBoxError> {
        if !platform.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') || platform.is_empty() { return Err(EmuBoxError::InvalidConfiguration("Plataforma inválida".to_string())); }
        let raw_filename = if uri.starts_with("magnet:") {
            decode_magnet_dn(uri)
        } else {
            let url = reqwest::Url::parse(uri).map_err(|e| EmuBoxError::InvalidConfiguration(e.to_string()))?;
            url.path_segments().and_then(|mut segments| segments.rfind(|s| !s.is_empty())).unwrap_or("download.bin").to_string()
        };
        let filename = raw_filename.replace(['/', '\\'], "_");
        let filename = if filename.is_empty() { "download.bin".to_string() } else { filename };
        Ok(Path::new(&paths::games_dir()).join(platform).join(filename))
    }

    fn update_status(id: &str, status: DownloadStatus, error: Option<String>) -> Result<(), EmuBoxError> {
        let value = match status { DownloadStatus::Queued => "queued", DownloadStatus::Downloading => "downloading", DownloadStatus::Paused => "paused", DownloadStatus::Completed => "completed", DownloadStatus::Failed => "failed", DownloadStatus::Cancelled => "cancelled" };
        DatabaseService::get_connection()?.execute("UPDATE download_jobs SET status = ?1, error = ?2 WHERE id = ?3", params![value, error, id]).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
        Ok(())
    }

    fn update_progress(id: &str, downloaded: u64, total: Option<u64>, speed: u64) -> Result<(), EmuBoxError> {
        let progress = total.map(|total| (downloaded as f32 / total.max(1) as f32).min(1.0)).unwrap_or(0.0);
        DatabaseService::get_connection()?.execute("UPDATE download_jobs SET progress = ?1, downloaded_bytes = ?2, total_bytes = COALESCE(?3, total_bytes), speed_bytes_per_second = ?4 WHERE id = ?5", params![progress, downloaded, total, speed, id]).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
        Ok(())
    }

    fn start_next_queued() {
        if active_jobs().lock().unwrap().is_empty() {
            if let Ok(jobs) = Self::list_jobs() {
                if let Some(next) = jobs.into_iter().find(|job| matches!(job.status, DownloadStatus::Queued)) {
                    let _ = Self::start(next.id);
                }
            }
        }
    }

    fn row_to_job(row: &rusqlite::Row<'_>) -> rusqlite::Result<DownloadJob> {
        let status: String = row.get(5)?;
        Ok(DownloadJob { id: row.get(0)?, game_id: row.get(1)?, source_id: row.get(2)?, platform: row.get(3)?, destination_path: row.get(4)?, status: match status.as_str() { "downloading" => DownloadStatus::Downloading, "paused" => DownloadStatus::Paused, "completed" => DownloadStatus::Completed, "failed" => DownloadStatus::Failed, "cancelled" => DownloadStatus::Cancelled, _ => DownloadStatus::Queued }, progress: row.get(6)?, downloaded_bytes: row.get(7)?, total_bytes: row.get(8)?, speed_bytes_per_second: row.get(9)?, error: row.get(10)? })
    }
}

fn uuid_like() -> String { format!("{}-{}", std::process::id(), chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()) }

fn request_error(error: reqwest::Error) -> EmuBoxError {
    use std::error::Error;
    let mut detail = String::new();
    let mut cause: Option<&dyn Error> = Some(&error);
    while let Some(current) = cause {
        detail.push_str(&current.to_string().to_ascii_lowercase());
        cause = current.source();
    }
    EmuBoxError::IpcError(network_message(&detail, error.is_timeout()).into())
}

fn network_message(detail: &str, timeout: bool) -> &'static str {
    if detail.contains("certificate") || detail.contains("certificado") {
        "No se pudo validar el certificado HTTPS de la fuente. No se ha desactivado la comprobacion de seguridad."
    } else if timeout {
        "La fuente no respondio a tiempo. Comprueba la conexion o selecciona otra fuente."
    } else if detail.contains("dns") || detail.contains("name resolution") {
        "No se pudo resolver el dominio de la fuente."
    } else {
        "No se pudo conectar con la fuente de descarga. Puede estar caida o requerir un conector de alojamiento."
    }
}

fn slug(value: &str) -> String {
    value.chars().map(|character| if character.is_ascii_alphanumeric() { character.to_ascii_lowercase() } else { '-' }).collect::<String>()
}

fn decode_magnet_dn(uri: &str) -> String {
    let dn_part = uri.split('&').find_map(|p| {
        if let Some(val) = p.strip_prefix("dn=") {
            Some(val)
        } else if let Some(idx) = p.find("?dn=") {
            Some(&p[idx + 4..])
        } else {
            None
        }
    });
    if let Some(raw) = dn_part {
        let mut decoded = String::new();
        let mut chars = raw.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '%' {
                let h1 = chars.next();
                let h2 = chars.next();
                if let (Some(h1), Some(h2)) = (h1, h2) {
                    let hex: String = [h1, h2].iter().collect();
                    if let Ok(b) = u8::from_str_radix(&hex, 16) {
                        decoded.push(b as char);
                        continue;
                    }
                    decoded.push('%');
                    decoded.push(h1);
                    decoded.push(h2);
                } else {
                    decoded.push('%');
                    if let Some(h1) = h1 { decoded.push(h1); }
                }
            } else if c == '+' {
                decoded.push(' ');
            } else {
                decoded.push(c);
            }
        }
        if !decoded.trim().is_empty() {
            return decoded;
        }
    }
    "download.bin".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_tls_dns_and_timeout_without_exposing_urls() {
        assert!(network_message("invalid peer certificate", false).contains("certificado HTTPS"));
        assert!(network_message("dns error", false).contains("dominio"));
        assert!(network_message("", true).contains("tiempo"));
    }

    #[test]
    fn imports_aliases_all_sources_and_legacy_without_jobs() {
        let manifest = r#"{"downloads":[{"gameId":"normalization-fixture", "title":"Fixture", "year":2024,
            "genres":["Action","Puzzle"], "descriptionHtml":"<p>Real &amp; plain</p>", "developer":"null",
            "sourceId":"shared", "uris":["https://megadb.net/fixture", "https://torrent.example.test/game.zip", "magnet:?xt=fixture"]}]}"#;
        let first = DownloadService::import_from_json(manifest).unwrap();
        let second = DownloadService::import_from_json(manifest).unwrap();
        assert_eq!(first.len(), 3);
        let options = DownloadService::list_sources("normalization-fixture").unwrap();
        assert!(!options[0].downloadable);
        assert!(options[1].downloadable);
        assert!(!options[2].downloadable);
        assert!(DownloadService::download_game("normalization-fixture".into()).is_err());
        assert!(DownloadService::download_game_from_source("normalization-fixture".into(), Some(options[0].source.id.clone())).is_err());
        assert!(DownloadService::download_game_from_source("normalization-fixture".into(), Some("another-game".into())).is_err());
        assert_eq!(first.iter().map(|source| &source.id).collect::<Vec<_>>(), second.iter().map(|source| &source.id).collect::<Vec<_>>());
        assert!(matches!(first[1].source_type, DownloadSourceType::Http));
        let game = GameService::get_game_by_id("normalization-fixture".into()).unwrap().unwrap();
        assert_eq!(game.release_year, 2024);
        assert_eq!(game.genre, "Action, Puzzle");
        assert_eq!(game.description, "Real & plain");
        assert_eq!(game.developer, "");
        assert!(!game.installed);
        assert!(DownloadService::list_jobs().unwrap().iter().all(|job| job.game_id != game.id));
        let legacy = DownloadService::import_from_json(r#"{"games":[{"gameId":"legacy-normalization-fixture", "name":"Legacy", "platform":"ps1", "url":"https://example.test/game.torrent?key=fixture"}]}"#).unwrap();
        assert_eq!(legacy.len(), 1);
        assert!(matches!(legacy[0].source_type, DownloadSourceType::Torrent));
    }

    #[test]
    fn destination_is_confined_to_platform_directory() {
        let destination = DownloadService::destination_path("ps3", "https://example.test/files/game.pkg").unwrap();
        assert_eq!(destination, Path::new(&paths::games_dir()).join("ps3/game.pkg"));
        assert!(DownloadService::destination_path("../ps3", "https://example.test/game.pkg").is_err());
    }

    #[test]
    fn destination_extracts_dn_from_magnet() {
        let destination = DownloadService::destination_path("pc", "magnet:?xt=urn:btih:123&dn=Dead.Rising.3.zip&tr=udp").unwrap();
        assert_eq!(destination, Path::new(&paths::games_dir()).join("pc/Dead.Rising.3.zip"));
    }

    #[test]
    fn parses_hydra_format_json_with_downloads_and_file_sizes() {
        let json = r#"{
            "name": "Hydra Test Source",
            "downloads": [
                {
                    "title": "Crash Bandicoot [PS1]",
                    "uris": ["https://example.test/crash.chd"],
                    "fileSize": "450 MB",
                    "uploadDate": "2023-01-15T12:00:00Z"
                },
                {
                    "title": "Dead Rising 3",
                    "uris": ["magnet:?xt=urn:btih:abc&dn=Dead.Rising.3.zip"],
                    "fileSize": "13.58 GB",
                    "uploadDate": "2014-09-30T10:53:57.000Z"
                }
            ]
        }"#;
        let sources = DownloadService::import_from_json(json).unwrap();
        assert_eq!(sources.len(), 2);
        let first = GameService::get_game_by_id(sources[0].game_id.clone()).unwrap().unwrap();
        assert_eq!(first.platform, "ps1");
        assert!(!first.installed);
        assert_ne!(first.release_year, 2023);
        assert_eq!(sources[0].size_bytes, Some(450 * 1024 * 1024));
        assert!(DownloadService::list_jobs().unwrap().iter().all(|job| !sources.iter().any(|source| source.game_id == job.game_id)));
        assert_eq!(DownloadService::import_from_json(json).unwrap().len(), 2);
        assert!(matches!(DownloadService::download_game(sources[1].game_id.clone()), Err(EmuBoxError::InvalidConfiguration(_))));
        assert!(DownloadService::list_jobs().unwrap().iter().all(|job| !sources.iter().any(|source| source.game_id == job.game_id)));
    }
}

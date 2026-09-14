use super::GameService;
use super::{PlatformSpec, PLATFORM_SPECS};
use crate::models::{ScanGamesRequest, ScanGamesResult};
use crate::{errors::EmuBoxError, services::db_service::DatabaseService};
use rusqlite::{params, OptionalExtension};
use std::{
    fs,
    path::{Path, PathBuf},
};

fn io_error(error: impl std::fmt::Display) -> EmuBoxError {
    EmuBoxError::StorageUnavailable(error.to_string())
}

impl GameService {
    pub fn get_canonical_games_dir() -> PathBuf {
        let base = PathBuf::from(crate::services::paths::games_dir());
        if base.exists() || fs::create_dir_all(&base).is_ok() {
            base
        } else {
            let fallback = PathBuf::from("/tmp/emubox/games");
            let _ = fs::create_dir_all(&fallback);
            fallback
        }
    }

    pub fn clean_title_from_filename(stem: &str) -> String {
        let mut title = stem.to_string();
        // Eliminar tags en paréntesis como (USA), (Europe), (v1.0), (Disc 1)
        while let Some(start) = title.find('(') {
            if let Some(end) = title[start..].find(')') {
                title.replace_range(start..start + end + 1, "");
            } else {
                break;
            }
        }
        // Eliminar tags en corchetes como [!], [b1], [En,Es]
        while let Some(start) = title.find('[') {
            if let Some(end) = title[start..].find(']') {
                title.replace_range(start..start + end + 1, "");
            } else {
                break;
            }
        }
        title.replace('_', " ").trim().to_string()
    }

    pub fn scan_games(request: Option<ScanGamesRequest>) -> Result<ScanGamesResult, EmuBoxError> {
        let conn = DatabaseService::get_connection()?;
        let _ = Self::get_platforms()?;

        let mut scanned_count = 0;
        let mut added_count = 0;
        let mut updated_count = 0;
        let mut removed_count = 0;
        let mut errors = Vec::new();

        let platforms_to_scan: Vec<&PlatformSpec> = if let Some(req) = &request {
            if let Some(target_plats) = &req.platforms {
                PLATFORM_SPECS
                    .iter()
                    .filter(|p| target_plats.iter().any(|target| target == p.id))
                    .collect()
            } else {
                PLATFORM_SPECS.iter().collect()
            }
        } else {
            PLATFORM_SPECS.iter().collect()
        };

        let custom_dir = request
            .as_ref()
            .and_then(|r| r.roms_directory.as_ref().map(PathBuf::from));
        let base_dirs: Vec<PathBuf> = if let Some(dir) = custom_dir {
            vec![dir]
        } else {
            vec![Self::get_canonical_games_dir()]
        };

        let mut scanned_roots = Vec::new();

        for plat in platforms_to_scan {
            for base_dir in &base_dirs {
                let plat_dir = base_dir.join(plat.id);
                match fs::symlink_metadata(&plat_dir) {
                    Ok(metadata) if metadata.is_dir() => {}
                    Ok(_) => {
                        errors.push(format!(
                            "Raiz de escaneo no es un directorio: {}",
                            plat_dir.display()
                        ));
                        continue;
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                    Err(error) => {
                        errors.push(format!("{}: {error}", plat_dir.display()));
                        continue;
                    }
                }

                match Self::scan_directory_recursive(
                    &plat_dir,
                    plat,
                    &conn,
                    &mut scanned_count,
                    &mut added_count,
                    &mut updated_count,
                ) {
                    Ok(()) => scanned_roots.push(plat_dir),
                    Err(error) => errors.push(format!("{}: {error}", plat_dir.display())),
                }
            }
        }

        let mut statement = conn
            .prepare("SELECT id, rom_path FROM games WHERE rom_path IS NOT NULL")
            .map_err(io_error)?;
        let rows = statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(io_error)?;
        for row in rows {
            let (id, filename) = row.map_err(io_error)?;
            let path = Path::new(&filename);
            if !scanned_roots.iter().any(|root| path.starts_with(root)) {
                continue;
            }
            match fs::metadata(path) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    removed_count += conn
                        .execute("DELETE FROM games WHERE id=?1", params![id])
                        .map_err(io_error)?;
                }
                Err(error) => errors.push(format!("{}: {error}", path.display())),
                Ok(_) => {}
            }
        }

        let total_count: usize = conn
            .query_row("SELECT COUNT(*) FROM games;", [], |row| row.get(0))
            .map_err(io_error)?;

        Ok(ScanGamesResult {
            scanned_count,
            added_count,
            updated_count,
            removed_count,
            total_count,
            errors,
        })
    }

    fn scan_directory_recursive(
        dir: &Path,
        plat: &PlatformSpec,
        conn: &rusqlite::Connection,
        scanned: &mut usize,
        added: &mut usize,
        updated: &mut usize,
    ) -> Result<(), EmuBoxError> {
        if dir.join(".emubox-managed").is_file()
            || dir
                .file_name()
                .is_some_and(|name| name == ".emubox-staging")
        {
            return Ok(());
        }
        let ps4_folder = plat.id == "ps4"
            && dir.join("eboot.bin").is_file()
            && dir.join("sce_sys/param.sfo").is_file();
        if (plat.id == "ps3" && dir.join("PS3_GAME").is_dir()) || ps4_folder {
            *scanned += 1;
            let title = dir
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Unknown");
            let rom_path = if ps4_folder {
                dir.join("eboot.bin")
            } else {
                dir.to_path_buf()
            };
            if Self::register_scanned_game(conn, plat, &rom_path, title, 0)? {
                *updated += 1;
            } else {
                *added += 1;
            }
            return Ok(());
        }

        for entry in fs::read_dir(dir).map_err(io_error)? {
            let entry = entry.map_err(io_error)?;
            let path = entry.path();
            let file_type = entry.file_type().map_err(io_error)?;
            if file_type.is_dir() {
                Self::scan_directory_recursive(&path, plat, conn, scanned, added, updated)?;
            } else if file_type.is_file() {
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if plat.extensions.contains(&ext.as_str()) {
                    *scanned += 1;
                    let stem = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("Unknown");
                    let file_size = entry.metadata().map_err(io_error)?.len();
                    if Self::register_scanned_game(conn, plat, &path, stem, file_size)? {
                        *updated += 1;
                    } else {
                        *added += 1;
                    }
                }
            }
        }
        Ok(())
    }

    fn register_scanned_game(
        conn: &rusqlite::Connection,
        platform: &PlatformSpec,
        path: &Path,
        title: &str,
        size: u64,
    ) -> Result<bool, EmuBoxError> {
        let filename = path.to_string_lossy();
        let exists = conn
            .query_row(
                "SELECT 1 FROM games WHERE rom_path=?1 LIMIT 1",
                params![filename],
                |_| Ok(true),
            )
            .optional()
            .map_err(io_error)?
            .unwrap_or(false);
        let id = format!("{}-{}", platform.id, title.replace(' ', "-").to_lowercase());
        conn.execute(
            "INSERT INTO games (id,title,platform_id,platform_name,rom_path,file_size_bytes)
             VALUES (?1,?2,?3,?4,?5,?6)
             ON CONFLICT(rom_path) DO UPDATE SET file_size_bytes=excluded.file_size_bytes",
            params![
                id,
                Self::clean_title_from_filename(title),
                platform.id,
                platform.name,
                filename,
                size
            ],
        )
        .map_err(io_error)?;
        Ok(exists)
    }
}

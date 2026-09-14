use super::GameService;
use super::{PlatformSpec, PLATFORM_SPECS};
use crate::models::{ScanGamesRequest, ScanGamesResult};
use crate::{errors::EmuBoxError, services::db_service::DatabaseService};
use rusqlite::params;
use std::{
    fs,
    path::{Path, PathBuf},
};

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
        let errors = Vec::new();

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

        let scanned_roots: Vec<PathBuf> = platforms_to_scan
            .iter()
            .flat_map(|plat| base_dirs.iter().map(move |base_dir| base_dir.join(plat.id)))
            .collect();

        for plat in platforms_to_scan {
            for base_dir in &base_dirs {
                let plat_dir = base_dir.join(plat.id);
                if !plat_dir.is_dir() {
                    continue;
                }

                Self::scan_directory_recursive(
                    &plat_dir,
                    plat,
                    &conn,
                    &mut scanned_count,
                    &mut added_count,
                    &mut updated_count,
                );
            }
        }

        // Purgar juegos cuyos archivos ya no existan en disco
        if let Ok(mut stmt) =
            conn.prepare("SELECT id, rom_path FROM games WHERE rom_path IS NOT NULL;")
        {
            let rows = stmt.query_map([], |row| {
                let id: String = row.get(0)?;
                let rom_path: String = row.get(1)?;
                Ok((id, rom_path))
            });

            if let Ok(mapped) = rows {
                for item in mapped.flatten() {
                    let path = Path::new(&item.1);
                    let belongs_to_scan = scanned_roots.iter().any(|root| path.starts_with(root));
                    if belongs_to_scan && !path.exists() {
                        if let Ok(aff) =
                            conn.execute("DELETE FROM games WHERE id = ?1;", params![item.0])
                        {
                            if aff > 0 {
                                removed_count += aff;
                            }
                        }
                    }
                }
            }
        }

        let total_count: usize = conn
            .query_row("SELECT COUNT(*) FROM games;", [], |row| row.get(0))
            .unwrap_or(0);

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
    ) {
        if dir.join(".emubox-managed").is_file()
            || dir
                .file_name()
                .is_some_and(|name| name == ".emubox-staging")
        {
            return;
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
            let clean_title = Self::clean_title_from_filename(title);
            let rom_path = if ps4_folder {
                dir.join("eboot.bin")
            } else {
                dir.to_path_buf()
            }
            .to_string_lossy()
            .to_string();
            let game_id = format!("{}-{}", plat.id, title.replace(' ', "-").to_lowercase());
            let exists_before: bool = conn
                .query_row(
                    "SELECT 1 FROM games WHERE rom_path = ?1 LIMIT 1;",
                    params![rom_path],
                    |_| Ok(true),
                )
                .unwrap_or(false);
            if conn.execute(
                "INSERT INTO games (id, title, platform_id, platform_name, release_year, genre, developer, publisher, rating, rom_path, file_size_bytes, description)
                 VALUES (?1, ?2, ?3, ?4, ?5, 'Classic', ?6, ?6, 4.5, ?7, 0, ?8)
                 ON CONFLICT(rom_path) DO UPDATE SET file_size_bytes = excluded.file_size_bytes;",
                params![game_id, clean_title, plat.id, plat.name, plat.release_year, plat.manufacturer, rom_path, format!("Juego oficial de {}", plat.name)],
            ).is_ok() {
                if exists_before { *updated += 1; } else { *added += 1; }
            }
            return;
        }

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    Self::scan_directory_recursive(&path, plat, conn, scanned, added, updated);
                } else if path.is_file() {
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
                        let clean_title = Self::clean_title_from_filename(stem);
                        let rom_path_str = path.to_string_lossy().to_string();
                        let file_size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                        let game_id =
                            format!("{}-{}", plat.id, stem.replace(' ', "-").to_lowercase());

                        // Comprobar si ya existía antes del insert
                        let exists_before: bool = conn
                            .query_row(
                                "SELECT 1 FROM games WHERE rom_path = ?1 LIMIT 1;",
                                params![rom_path_str],
                                |_| Ok(true),
                            )
                            .unwrap_or(false);

                        let res = conn.execute(
                            "INSERT INTO games (id, title, platform_id, platform_name, release_year, genre, developer, publisher, rating, rom_path, file_size_bytes, description)
                             VALUES (?1, ?2, ?3, ?4, ?5, 'Classic', ?6, ?6, 4.5, ?7, ?8, ?9)
                             ON CONFLICT(rom_path) DO UPDATE SET
                               file_size_bytes = excluded.file_size_bytes;",
                            params![
                                game_id,
                                clean_title,
                                plat.id,
                                plat.name,
                                plat.release_year,
                                plat.manufacturer,
                                rom_path_str,
                                file_size,
                                format!("Juego oficial de {}", plat.name)
                            ]
                        );

                        if res.is_ok() {
                            if !exists_before {
                                *added += 1;
                            } else {
                                *updated += 1;
                            }
                        }
                    }
                }
            }
        }
    }
}

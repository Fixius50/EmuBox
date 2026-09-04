use std::path::{Path, PathBuf};
use std::fs;
use rusqlite::params;
use crate::models::{Game, Platform, GameFilter, ScanGamesRequest, ScanGamesResult};
use crate::errors::EmuBoxError;
use crate::services::db_service::DatabaseService;

pub struct PlatformSpec {
    pub id: &'static str,
    pub name: &'static str,
    pub short_name: &'static str,
    pub manufacturer: &'static str,
    pub generation: u32,
    pub release_year: u32,
    pub color: &'static str,
    pub icon: &'static str,
    pub default_emulator_id: &'static str,
    pub extensions: &'static [&'static str],
}

pub const PLATFORM_SPECS: &[PlatformSpec] = &[
    PlatformSpec {
        id: "ps2",
        name: "PlayStation 2",
        short_name: "PS2",
        manufacturer: "Sony",
        generation: 6,
        release_year: 2000,
        color: "#003791",
        icon: "ps2",
        default_emulator_id: "pcsx2",
        extensions: &["iso", "chd", "cso", "bin", "gz"],
    },
    PlatformSpec {
        id: "ps3",
        name: "PlayStation 3",
        short_name: "PS3",
        manufacturer: "Sony",
        generation: 7,
        release_year: 2006,
        color: "#2f2f2f",
        icon: "ps3",
        default_emulator_id: "rpcs3",
        extensions: &["iso", "pkg", "zip", "7z"],
    },
    PlatformSpec {
        id: "ps1",
        name: "PlayStation",
        short_name: "PS1",
        manufacturer: "Sony",
        generation: 5,
        release_year: 1994,
        color: "#003791",
        icon: "ps1",
        default_emulator_id: "duckstation",
        extensions: &["chd", "cue", "iso", "pbp", "bin", "m3u"],
    },
    PlatformSpec {
        id: "gamecube",
        name: "Nintendo GameCube",
        short_name: "GCN",
        manufacturer: "Nintendo",
        generation: 6,
        release_year: 2001,
        color: "#6a5acd",
        icon: "gamecube",
        default_emulator_id: "dolphin",
        extensions: &["rvz", "iso", "gcm", "ciso"],
    },
    PlatformSpec {
        id: "snes",
        name: "Super Nintendo Entertainment System",
        short_name: "SNES",
        manufacturer: "Nintendo",
        generation: 4,
        release_year: 1990,
        color: "#800080",
        icon: "snes",
        default_emulator_id: "retroarch",
        extensions: &["sfc", "smc", "zip", "7z"],
    },
    PlatformSpec {
        id: "gba",
        name: "Game Boy Advance",
        short_name: "GBA",
        manufacturer: "Nintendo",
        generation: 6,
        release_year: 2001,
        color: "#800080",
        icon: "gba",
        default_emulator_id: "mgba",
        extensions: &["gba", "zip", "7z"],
    },
    PlatformSpec {
        id: "n64",
        name: "Nintendo 64",
        short_name: "N64",
        manufacturer: "Nintendo",
        generation: 5,
        release_year: 1996,
        color: "#e60012",
        icon: "n64",
        default_emulator_id: "retroarch",
        extensions: &["z64", "n64", "v64", "zip"],
    },
    PlatformSpec {
        id: "genesis",
        name: "Sega Genesis / Mega Drive",
        short_name: "Genesis",
        manufacturer: "Sega",
        generation: 4,
        release_year: 1988,
        color: "#000000",
        icon: "genesis",
        default_emulator_id: "retroarch",
        extensions: &["md", "gen", "bin", "smd", "zip"],
    },
    PlatformSpec {
        id: "dreamcast",
        name: "Sega Dreamcast",
        short_name: "DC",
        manufacturer: "Sega",
        generation: 6,
        release_year: 1998,
        color: "#ff6600",
        icon: "dreamcast",
        default_emulator_id: "flycast",
        extensions: &["cdi", "gdi", "chd", "cue"],
    },
    PlatformSpec {
        id: "psp",
        name: "PlayStation Portable",
        short_name: "PSP",
        manufacturer: "Sony",
        generation: 7,
        release_year: 2004,
        color: "#003791",
        icon: "psp",
        default_emulator_id: "ppsspp",
        extensions: &["iso", "cso", "pbp"],
    },
    PlatformSpec {
        id: "nds",
        name: "Nintendo DS",
        short_name: "NDS",
        manufacturer: "Nintendo",
        generation: 7,
        release_year: 2004,
        color: "#808080",
        icon: "nds",
        default_emulator_id: "melonds",
        extensions: &["nds", "zip", "7z"],
    },
    PlatformSpec {
        id: "arcade",
        name: "Arcade Machines",
        short_name: "Arcade",
        manufacturer: "Various",
        generation: 3,
        release_year: 1985,
        color: "#ffcc00",
        icon: "arcade",
        default_emulator_id: "retroarch",
        extensions: &["zip", "7z", "chd"],
    },
];

pub struct GameService;

impl GameService {
    pub fn get_canonical_games_dir() -> PathBuf {
        let base = Path::new("/var/lib/emubox/games");
        if base.exists() || fs::create_dir_all(base).is_ok() {
            base.to_path_buf()
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

    pub fn get_platforms() -> Result<Vec<Platform>, EmuBoxError> {
        let conn = DatabaseService::get_connection()?;

        for p in PLATFORM_SPECS {
            let exts_json = serde_json::to_string(&p.extensions).unwrap_or_else(|_| "[]".to_string());
            conn.execute(
                "INSERT INTO systems (id, name, short_name, manufacturer, generation, release_year, color, icon, default_emulator_id, extensions_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                 ON CONFLICT(id) DO UPDATE SET
                   name = excluded.name,
                   short_name = excluded.short_name,
                   default_emulator_id = excluded.default_emulator_id,
                   extensions_json = excluded.extensions_json;",
                params![p.id, p.name, p.short_name, p.manufacturer, p.generation, p.release_year, p.color, p.icon, p.default_emulator_id, exts_json]
            ).map_err(|e| EmuBoxError::StorageUnavailable(format!("Error guardando plataforma en SQLite: {}", e)))?;
        }

        let list = PLATFORM_SPECS.iter().map(|p| Platform {
            id: p.id.to_string(),
            name: p.name.to_string(),
            short_name: p.short_name.to_string(),
            manufacturer: p.manufacturer.to_string(),
            generation: p.generation,
            release_year: p.release_year,
            color: p.color.to_string(),
            icon: p.icon.to_string(),
            default_emulator_id: p.default_emulator_id.to_string(),
        }).collect();

        Ok(list)
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
                PLATFORM_SPECS.iter().filter(|p| target_plats.iter().any(|target| target == p.id)).collect()
            } else {
                PLATFORM_SPECS.iter().collect()
            }
        } else {
            PLATFORM_SPECS.iter().collect()
        };

        let custom_dir = request.as_ref().and_then(|r| r.roms_directory.as_ref().map(PathBuf::from));
        let base_dirs: Vec<PathBuf> = if let Some(dir) = custom_dir {
            vec![dir]
        } else {
            vec![Self::get_canonical_games_dir()]
        };

        let scanned_roots: Vec<PathBuf> = platforms_to_scan.iter()
            .flat_map(|plat| base_dirs.iter().map(move |base_dir| base_dir.join(plat.id)))
            .collect();

        for plat in platforms_to_scan {
            for base_dir in &base_dirs {
                let plat_dir = base_dir.join(plat.id);
                if !plat_dir.is_dir() {
                    continue;
                }

                Self::scan_directory_recursive(&plat_dir, plat, &conn, &mut scanned_count, &mut added_count, &mut updated_count);
            }
        }

        // Purgar juegos cuyos archivos ya no existan en disco
        if let Ok(mut stmt) = conn.prepare("SELECT id, rom_path FROM games WHERE rom_path IS NOT NULL;") {
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
                        if let Ok(aff) = conn.execute("DELETE FROM games WHERE id = ?1;", params![item.0]) {
                            if aff > 0 {
                                removed_count += aff;
                            }
                        }
                    }
                }
            }
        }

        let total_count: usize = conn.query_row(
            "SELECT COUNT(*) FROM games;",
            [],
            |row| row.get(0)
        ).unwrap_or(0);

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
        if plat.id == "ps3" && dir.join("PS3_GAME").is_dir() {
            *scanned += 1;
            let title = dir.file_name().and_then(|name| name.to_str()).unwrap_or("Unknown");
            let clean_title = Self::clean_title_from_filename(title);
            let rom_path = dir.to_string_lossy().to_string();
            let game_id = format!("{}-{}", plat.id, title.replace(' ', "-").to_lowercase());
            let exists_before: bool = conn.query_row(
                "SELECT 1 FROM games WHERE rom_path = ?1 LIMIT 1;",
                params![rom_path],
                |_| Ok(true),
            ).unwrap_or(false);
            if conn.execute(
                "INSERT INTO games (id, title, platform_id, platform_name, release_year, genre, developer, publisher, rating, rom_path, file_size_bytes, description)
                 VALUES (?1, ?2, ?3, ?4, ?5, 'Classic', ?6, ?6, 4.5, ?7, 0, ?8)
                 ON CONFLICT(rom_path) DO UPDATE SET title = excluded.title, platform_id = excluded.platform_id;",
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
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                    if plat.extensions.contains(&ext.as_str()) {
                        *scanned += 1;
                        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Unknown");
                        let clean_title = Self::clean_title_from_filename(stem);
                        let rom_path_str = path.to_string_lossy().to_string();
                        let file_size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                        let game_id = format!("{}-{}", plat.id, stem.replace(' ', "-").to_lowercase());

                        // Comprobar si ya existía antes del insert
                        let exists_before: bool = conn.query_row(
                            "SELECT 1 FROM games WHERE rom_path = ?1 LIMIT 1;",
                            params![rom_path_str],
                            |_| Ok(true)
                        ).unwrap_or(false);

                        let res = conn.execute(
                            "INSERT INTO games (id, title, platform_id, platform_name, release_year, genre, developer, publisher, rating, rom_path, file_size_bytes, description)
                             VALUES (?1, ?2, ?3, ?4, ?5, 'Classic', ?6, ?6, 4.5, ?7, ?8, ?9)
                             ON CONFLICT(rom_path) DO UPDATE SET
                               title = excluded.title,
                               platform_id = excluded.platform_id,
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

    pub fn get_games(filter: Option<GameFilter>) -> Result<Vec<Game>, EmuBoxError> {
        let conn = DatabaseService::get_connection()?;
        let mut query = "SELECT id, title, platform_id, platform_name, release_year, genre, developer, publisher, rating, play_time_minutes, favorite, cover_image, backdrop_image, description, rom_path FROM games WHERE 1=1".to_string();
        let mut param_values: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(f) = filter {
            if let Some(target_plat) = f.platform {
                if target_plat != "all" {
                    query.push_str(" AND platform_id = ?");
                    param_values.push(Box::new(target_plat));
                }
            }
            if let Some(q) = f.search {
                if !q.trim().is_empty() {
                    query.push_str(" AND title LIKE ?");
                    param_values.push(Box::new(format!("%{}%", q.trim())));
                }
            }
            if let Some(fav_only) = f.favorite {
                if fav_only {
                    query.push_str(" AND favorite = 1");
                }
            }
        }

        query.push_str(" ORDER BY title ASC;");

        let mut stmt = conn.prepare(&query).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
        let params_slice: Vec<&dyn rusqlite::ToSql> = param_values.iter().map(|b| b.as_ref()).collect();

        let rows = stmt.query_map(params_slice.as_slice(), |row| {
            let id: String = row.get(0)?;
            let title: String = row.get(1)?;
            let platform: String = row.get(2)?;
            let platform_name: String = row.get(3)?;
            let release_year: u32 = row.get(4).unwrap_or(2000);
            let genre: String = row.get(5).unwrap_or_else(|_| "Classic".to_string());
            let developer: String = row.get(6).unwrap_or_default();
            let publisher: String = row.get(7).unwrap_or_default();
            let rating: f32 = row.get(8).unwrap_or(4.0);
            let play_time_minutes: u32 = row.get(9).unwrap_or(0);
            let favorite_int: i32 = row.get(10).unwrap_or(0);
            let cover_image: String = row.get(11).unwrap_or_default();
            let backdrop_image: Option<String> = row.get(12).unwrap_or(None);
            let description: String = row.get(13).unwrap_or_default();
            let rom_path: Option<String> = row.get(14).unwrap_or(None);

            Ok(Game {
                id,
                title,
                platform,
                platform_name,
                release_year,
                genre,
                developer,
                publisher,
                rating,
                play_time_minutes,
                favorite: favorite_int == 1,
                cover_image,
                backdrop_image,
                description,
                rom_path,
                file_size_mb: None,
                last_played: None,
                emulator_id: None,
            })
        }).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;

        let mut list = Vec::new();
        for game in rows.flatten() {
            list.push(game);
        }

        Ok(list)
    }

    pub fn get_game_by_id(id: String) -> Result<Option<Game>, EmuBoxError> {
        let conn = DatabaseService::get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, platform_id, platform_name, release_year, genre, developer, publisher, rating, play_time_minutes, favorite, cover_image, backdrop_image, description, rom_path
             FROM games WHERE id = ?1 LIMIT 1;"
        ).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;

        let mut rows = stmt.query_map(params![id], |row| {
            let id: String = row.get(0)?;
            let title: String = row.get(1)?;
            let platform: String = row.get(2)?;
            let platform_name: String = row.get(3)?;
            let release_year: u32 = row.get(4).unwrap_or(2000);
            let genre: String = row.get(5).unwrap_or_else(|_| "Classic".to_string());
            let developer: String = row.get(6).unwrap_or_default();
            let publisher: String = row.get(7).unwrap_or_default();
            let rating: f32 = row.get(8).unwrap_or(4.0);
            let play_time_minutes: u32 = row.get(9).unwrap_or(0);
            let favorite_int: i32 = row.get(10).unwrap_or(0);
            let cover_image: String = row.get(11).unwrap_or_default();
            let backdrop_image: Option<String> = row.get(12).unwrap_or(None);
            let description: String = row.get(13).unwrap_or_default();
            let rom_path: Option<String> = row.get(14).unwrap_or(None);

            Ok(Game {
                id,
                title,
                platform,
                platform_name,
                release_year,
                genre,
                developer,
                publisher,
                rating,
                play_time_minutes,
                favorite: favorite_int == 1,
                cover_image,
                backdrop_image,
                description,
                rom_path,
                file_size_mb: None,
                last_played: None,
                emulator_id: None,
            })
        }).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;

        if let Some(first) = rows.next() {
            return Ok(first.ok());
        }

        Ok(None)
    }

    pub fn toggle_favorite(game_id: String) -> Result<bool, EmuBoxError> {
        let conn = DatabaseService::get_connection()?;
        
        let current_fav: i32 = conn.query_row(
            "SELECT favorite FROM games WHERE id = ?1;",
            params![game_id],
            |row| row.get(0)
        ).unwrap_or(0);

        let new_fav = if current_fav == 1 { 0 } else { 1 };

        conn.execute(
            "UPDATE games SET favorite = ?1 WHERE id = ?2;",
            params![new_fav, game_id]
        ).map_err(|e| EmuBoxError::StorageUnavailable(format!("Error al alternar favorito en SQLite: {}", e)))?;

        Ok(new_fav == 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_title_from_filename() {
        assert_eq!(GameService::clean_title_from_filename("2048 (World) (Homebrew)"), "2048");
        assert_eq!(GameService::clean_title_from_filename("Gran_Turismo_4_[v1.0]_(USA)"), "Gran Turismo 4");
        assert_eq!(GameService::clean_title_from_filename("Super_Mario_World"), "Super Mario World");
    }

    #[test]
    fn test_scan_games_directory() {
        let temp_dir = std::env::temp_dir().join(format!("emubox-game-test-{}", std::process::id()));
        let snes_dir = temp_dir.join("snes");
        fs::create_dir_all(&snes_dir).unwrap();

        let rom_file = snes_dir.join("Test_Game_(USA)_(v1.0).sfc");
        fs::write(&rom_file, "TEST_ROM_CONTENT").unwrap();

        let scan_req = ScanGamesRequest {
            platforms: Some(vec!["snes".to_string()]),
            roms_directory: Some(temp_dir.to_string_lossy().to_string()),
            deep_scan: None,
        };

        let res = GameService::scan_games(Some(scan_req)).unwrap();
        assert!(res.scanned_count >= 1);

        let games = GameService::get_games(Some(GameFilter {
            platform: Some("snes".to_string()),
            search: Some("Test Game".to_string()),
            favorite: None,
            limit: None,
            offset: None,
        })).unwrap();

        assert!(!games.is_empty());
        assert_eq!(games[0].title, "Test Game");

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_scan_ps3_installed_folder() {
        let temp_dir = std::env::temp_dir().join(format!("emubox-ps3-test-{}", std::process::id()));
        let game_dir = temp_dir.join("ps3").join("Gran Turismo 6 (Europe)").join("PS3_GAME");
        fs::create_dir_all(&game_dir).unwrap();

        let scan_req = ScanGamesRequest {
            platforms: Some(vec!["ps3".to_string()]),
            roms_directory: Some(temp_dir.to_string_lossy().to_string()),
            deep_scan: Some(true),
        };
        let result = GameService::scan_games(Some(scan_req)).unwrap();
        assert_eq!(result.scanned_count, 1);

        let games = GameService::get_games(Some(GameFilter {
            platform: Some("ps3".to_string()),
            search: Some("Gran Turismo 6".to_string()),
            favorite: None,
            limit: None,
            offset: None,
        })).unwrap();
        assert!(games.iter().any(|game| game.title == "Gran Turismo 6"));

        fs::remove_dir_all(&temp_dir).unwrap();
    }
}

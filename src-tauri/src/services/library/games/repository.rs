use super::GameService;
use crate::models::{Game, GameFilter};
use crate::{errors::EmuBoxError, services::db_service::DatabaseService};
use rusqlite::params;

impl GameService {
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

        let mut stmt = conn
            .prepare(&query)
            .map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
        let params_slice: Vec<&dyn rusqlite::ToSql> =
            param_values.iter().map(|b| b.as_ref()).collect();

        let rows = stmt
            .query_map(params_slice.as_slice(), |row| {
                let id: String = row.get(0)?;
                let title: String = row.get(1)?;
                let platform: String = row.get(2)?;
                let platform_name: String = row.get(3)?;
                let release_year: u32 = row.get(4).unwrap_or_default();
                let genre: String = row.get(5).unwrap_or_default();
                let developer: String = row.get(6).unwrap_or_default();
                let publisher: String = row.get(7).unwrap_or_default();
                let rating: f32 = row.get(8).unwrap_or_default();
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
                    installed: rom_path.is_some(),
                    rom_path,
                    file_size_mb: None,
                    last_played: None,
                    emulator_id: None,
                })
            })
            .map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;

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

        let mut rows = stmt
            .query_map(params![id], |row| {
                let id: String = row.get(0)?;
                let title: String = row.get(1)?;
                let platform: String = row.get(2)?;
                let platform_name: String = row.get(3)?;
                let release_year: u32 = row.get(4).unwrap_or_default();
                let genre: String = row.get(5).unwrap_or_default();
                let developer: String = row.get(6).unwrap_or_default();
                let publisher: String = row.get(7).unwrap_or_default();
                let rating: f32 = row.get(8).unwrap_or_default();
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
                    installed: rom_path.is_some(),
                    rom_path,
                    file_size_mb: None,
                    last_played: None,
                    emulator_id: None,
                })
            })
            .map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;

        if let Some(first) = rows.next() {
            return Ok(first.ok());
        }

        Ok(None)
    }

    pub fn toggle_favorite(game_id: String) -> Result<bool, EmuBoxError> {
        let conn = DatabaseService::get_connection()?;

        let current_fav: i32 = conn
            .query_row(
                "SELECT favorite FROM games WHERE id = ?1;",
                params![game_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let new_fav = if current_fav == 1 { 0 } else { 1 };

        conn.execute(
            "UPDATE games SET favorite = ?1 WHERE id = ?2;",
            params![new_fav, game_id],
        )
        .map_err(|e| {
            EmuBoxError::StorageUnavailable(format!("Error al alternar favorito en SQLite: {}", e))
        })?;

        Ok(new_fav == 1)
    }
}

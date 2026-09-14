use super::GameService;
use crate::models::{Game, GameFilter};
use crate::{errors::EmuBoxError, services::db_service::DatabaseService};
use rusqlite::{params, OptionalExtension};

const GAME_SELECT: &str = "SELECT id, title, platform_id, platform_name, release_year, genre, developer, publisher, rating, play_time_minutes, favorite, cover_image, backdrop_image, description, rom_path FROM games";

fn row_to_game(row: &rusqlite::Row<'_>) -> rusqlite::Result<Game> {
    let rom_path: Option<String> = row.get(14)?;
    Ok(Game {
        id: row.get(0)?,
        title: row.get(1)?,
        platform: row.get(2)?,
        platform_name: row.get(3)?,
        release_year: row.get::<_, Option<u32>>(4)?.unwrap_or_default(),
        genre: row.get::<_, Option<String>>(5)?.unwrap_or_default(),
        developer: row.get::<_, Option<String>>(6)?.unwrap_or_default(),
        publisher: row.get::<_, Option<String>>(7)?.unwrap_or_default(),
        rating: row.get::<_, Option<f32>>(8)?.unwrap_or_default(),
        play_time_minutes: row.get::<_, Option<u32>>(9)?.unwrap_or_default(),
        favorite: row.get::<_, Option<i32>>(10)?.unwrap_or_default() == 1,
        cover_image: row.get::<_, Option<String>>(11)?.unwrap_or_default(),
        backdrop_image: row.get(12)?,
        description: row.get::<_, Option<String>>(13)?.unwrap_or_default(),
        installed: rom_path.is_some(),
        rom_path,
        file_size_mb: None,
        last_played: None,
        emulator_id: None,
    })
}

impl GameService {
    pub fn get_games(filter: Option<GameFilter>) -> Result<Vec<Game>, EmuBoxError> {
        let conn = DatabaseService::get_connection()?;
        let mut query = format!("{GAME_SELECT} WHERE 1=1");
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
            .query_map(params_slice.as_slice(), row_to_game)
            .map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))
    }

    pub fn get_game_by_id(id: String) -> Result<Option<Game>, EmuBoxError> {
        let conn = DatabaseService::get_connection()?;
        conn.query_row(
            &format!("{GAME_SELECT} WHERE id = ?1"),
            params![id],
            row_to_game,
        )
        .optional()
        .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))
    }

    pub fn toggle_favorite(game_id: String) -> Result<bool, EmuBoxError> {
        let conn = DatabaseService::get_connection()?;

        let favorite: Option<i32> = conn
            .query_row(
                "UPDATE games SET favorite = CASE WHEN favorite = 1 THEN 0 ELSE 1 END WHERE id = ?1 RETURNING favorite",
                params![game_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        favorite
            .map(|value| value == 1)
            .ok_or_else(|| EmuBoxError::NotFound(format!("Juego inexistente: {game_id}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_mapping_preserves_nulls_but_rejects_invalid_types() {
        let connection = rusqlite::Connection::open_in_memory().unwrap();
        let query =
            "SELECT 'id','title','ps2','PS2',?1,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL";
        let game = connection
            .query_row(query, [None::<u32>], row_to_game)
            .unwrap();
        assert_eq!(game.release_year, 0);
        assert!(!game.installed);
        assert!(connection
            .query_row(query, ["invalid year"], row_to_game)
            .is_err());
    }

    #[test]
    fn missing_game_is_not_a_successful_favorite_update() {
        assert!(matches!(
            GameService::toggle_favorite("missing-favorite-test".into()),
            Err(EmuBoxError::NotFound(_))
        ));
        assert!(GameService::get_game_by_id("missing-favorite-test".into())
            .unwrap()
            .is_none());
    }
}

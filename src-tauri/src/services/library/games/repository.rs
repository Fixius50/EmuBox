use super::GameService;
use crate::models::{Game, GameFilter};
use crate::{errors::EmuBoxError, services::db_service::DatabaseService};
use rusqlite::{params, OptionalExtension};

const GAME_SELECT: &str = "SELECT game.id, game.title, game.platform_id, game.platform_name,
    COALESCE(canonical.release_year, game.release_year), COALESCE(canonical.genre, game.genre),
    COALESCE(canonical.developer, game.developer), COALESCE(canonical.publisher, game.publisher),
    game.rating, game.play_time_minutes, CASE WHEN canonical.favorite=1 OR game.favorite=1 THEN 1 ELSE 0 END,
    COALESCE(canonical.cover_image, game.cover_image), game.backdrop_image,
    COALESCE(canonical.description, game.description), game.rom_path,
    canonical.id, canonical.title, release.id, release.title, match.method
    FROM games AS game
    LEFT JOIN catalog_game_matches AS match ON match.catalog_game_id=game.id
    LEFT JOIN canonical_games AS canonical ON canonical.id=match.canonical_game_id
    LEFT JOIN game_releases AS release ON release.id=match.release_id";

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
        canonical_id: row.get(15)?,
        canonical_title: row.get(16)?,
        release_id: row.get(17)?,
        release_title: row.get(18)?,
        match_method: row.get(19)?,
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
                    query.push_str(" AND game.platform_id = ?");
                    param_values.push(Box::new(target_plat));
                }
            }
            if let Some(q) = f.search {
                if !q.trim().is_empty() {
                    query.push_str(" AND game.title LIKE ?");
                    param_values.push(Box::new(format!("%{}%", q.trim())));
                }
            }
            if let Some(fav_only) = f.favorite {
                if fav_only {
                    query.push_str(" AND (game.favorite = 1 OR canonical.favorite = 1)");
                }
            }
        }

        query.push_str(" ORDER BY COALESCE(canonical.title, game.title) ASC, game.title ASC;");

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
            &format!("{GAME_SELECT} WHERE game.id = ?1"),
            params![id],
            row_to_game,
        )
        .optional()
        .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))
    }

    pub fn toggle_favorite(game_id: String) -> Result<bool, EmuBoxError> {
        let mut conn = DatabaseService::get_connection()?;
        let transaction = conn
            .transaction()
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        let canonical_id: Option<String> = transaction
            .query_row(
                "SELECT match.canonical_game_id FROM games AS game LEFT JOIN catalog_game_matches AS match ON match.catalog_game_id=game.id WHERE game.id=?1",
                params![&game_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        let favorite = if let Some(canonical_id) = canonical_id {
            let current: i32 = transaction.query_row(
                "SELECT CASE WHEN canonical.favorite=1 OR EXISTS(SELECT 1 FROM games AS game JOIN catalog_game_matches AS match ON match.catalog_game_id=game.id WHERE match.canonical_game_id=canonical.id AND game.favorite=1) THEN 1 ELSE 0 END FROM canonical_games AS canonical WHERE canonical.id=?1",
                params![&canonical_id], |row| row.get(0))
                .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
            let next = if current == 1 { 0 } else { 1 };
            transaction
                .execute(
                    "UPDATE canonical_games SET favorite=?1 WHERE id=?2",
                    params![next, &canonical_id],
                )
                .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
            transaction.execute("UPDATE games SET favorite=0 WHERE id IN (SELECT catalog_game_id FROM catalog_game_matches WHERE canonical_game_id=?1)", params![canonical_id])
                .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
            next == 1
        } else {
            transaction.query_row(
                "UPDATE games SET favorite = CASE WHEN favorite = 1 THEN 0 ELSE 1 END WHERE id = ?1 RETURNING favorite",
                params![&game_id], |row| row.get::<_, i32>(0))
                .optional()
                .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?
                .map(|value| value == 1)
                .ok_or_else(|| EmuBoxError::NotFound(format!("Juego inexistente: {game_id}")))?
        };
        transaction
            .commit()
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        Ok(favorite)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{models::CatalogEntry, services::game_database};

    #[test]
    fn row_mapping_preserves_nulls_but_rejects_invalid_types() {
        let connection = rusqlite::Connection::open_in_memory().unwrap();
        let query =
            "SELECT 'id','title','ps2','PS2',?1,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL";
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

    #[test]
    fn favorite_is_shared_by_variants_of_a_canonical_game() {
        GameService::get_platforms().unwrap();
        let ids = [
            "favorite-canonical-fixture-a",
            "favorite-canonical-fixture-b",
        ];
        for id in ids {
            GameService::upsert_catalog_entry(CatalogEntry {
                id: id.into(),
                title: "Favorite Canonical Fixture".into(),
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
        game_database::ensure_local_index().unwrap();
        assert!(GameService::toggle_favorite(ids[0].into()).unwrap());
        assert!(ids
            .iter()
            .all(|id| GameService::get_game_by_id((*id).into())
                .unwrap()
                .unwrap()
                .favorite));
        assert!(!GameService::toggle_favorite(ids[1].into()).unwrap());
        assert!(ids
            .iter()
            .all(|id| !GameService::get_game_by_id((*id).into())
                .unwrap()
                .unwrap()
                .favorite));
        let connection = DatabaseService::get_connection().unwrap();
        for id in ids {
            connection
                .execute("DELETE FROM games WHERE id=?1", [id])
                .unwrap();
        }
        connection
            .execute(
                "DELETE FROM canonical_games WHERE normalized_title='favorite canonical fixture'",
                [],
            )
            .unwrap();
    }
}

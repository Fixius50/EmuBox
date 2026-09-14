use super::GameService;
use super::{CatalogEntry, PLATFORM_SPECS};
use crate::models::Platform;
use crate::{errors::EmuBoxError, services::db_service::DatabaseService};
use rusqlite::params;

impl GameService {
    pub fn get_platforms() -> Result<Vec<Platform>, EmuBoxError> {
        let conn = DatabaseService::get_connection()?;

        for p in PLATFORM_SPECS {
            let exts_json =
                serde_json::to_string(&p.extensions).unwrap_or_else(|_| "[]".to_string());
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

        let list = PLATFORM_SPECS
            .iter()
            .map(|p| Platform {
                id: p.id.to_string(),
                name: p.name.to_string(),
                short_name: p.short_name.to_string(),
                manufacturer: p.manufacturer.to_string(),
                generation: p.generation,
                release_year: p.release_year,
                color: p.color.to_string(),
                icon: p.icon.to_string(),
                default_emulator_id: p.default_emulator_id.to_string(),
            })
            .collect();

        Ok(list)
    }

    /// Crea o actualiza la entrada de catálogo de un juego (metadatos) sin tocar
    /// `rom_path`, de modo que un juego pendiente de descarga ya aparezca en la
    /// biblioteca y no pierda su estado de instalado en re-importaciones del manifest.
    pub fn upsert_catalog_entry(entry: CatalogEntry) -> Result<(), EmuBoxError> {
        let conn = DatabaseService::get_connection()?;
        Self::upsert_catalog_entry_on(&conn, entry)
    }

    pub(crate) fn upsert_catalog_entry_on(
        conn: &rusqlite::Connection,
        entry: CatalogEntry,
    ) -> Result<(), EmuBoxError> {
        conn.execute(
            "INSERT OR IGNORE INTO systems (id, name, short_name, manufacturer, generation, release_year, color, icon, default_emulator_id, extensions_json)
             VALUES (?1, ?2, ?3, '', 0, 0, '', ?1, '', '[]');",
            params![entry.platform_id, entry.platform_name, entry.platform_id.to_uppercase()],
        ).map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        conn.execute(
            "INSERT INTO games (id, title, platform_id, platform_name, release_year, genre, developer, publisher, rating, cover_image, backdrop_image, description)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
             ON CONFLICT(id) DO UPDATE SET
               title = excluded.title,
               platform_id = excluded.platform_id,
               platform_name = excluded.platform_name,
               release_year = COALESCE(excluded.release_year, games.release_year),
               genre = COALESCE(excluded.genre, CASE WHEN lower(trim(games.genre)) IN ('null', 'undefined') THEN NULL ELSE games.genre END),
               developer = COALESCE(excluded.developer, CASE WHEN lower(trim(games.developer)) IN ('null', 'undefined') THEN NULL ELSE games.developer END),
               publisher = COALESCE(excluded.publisher, CASE WHEN lower(trim(games.publisher)) IN ('null', 'undefined') THEN NULL ELSE games.publisher END),
               rating = COALESCE(excluded.rating, games.rating),
               cover_image = COALESCE(excluded.cover_image, CASE WHEN lower(trim(games.cover_image)) IN ('null', 'undefined') THEN NULL ELSE games.cover_image END),
               backdrop_image = COALESCE(excluded.backdrop_image, CASE WHEN lower(trim(games.backdrop_image)) IN ('null', 'undefined') THEN NULL ELSE games.backdrop_image END),
               description = COALESCE(excluded.description, CASE WHEN lower(trim(games.description)) IN ('null', 'undefined') THEN NULL ELSE games.description END);",
            params![
                entry.id, entry.title, entry.platform_id, entry.platform_name, entry.release_year,
                entry.genre, entry.developer, entry.publisher, entry.rating,
                entry.cover_image, entry.backdrop_image, entry.description
            ],
        ).map_err(|e| EmuBoxError::StorageUnavailable(format!("Error guardando catálogo de juego: {}", e)))?;
        Ok(())
    }

    /// Marca un juego de catálogo como instalado una vez la descarga ha terminado.
    pub fn mark_installed(
        game_id: &str,
        rom_path: &str,
        file_size_bytes: u64,
    ) -> Result<(), EmuBoxError> {
        let file_size = i64::try_from(file_size_bytes)
            .map_err(|error| EmuBoxError::InvalidConfiguration(error.to_string()))?;
        let conn = DatabaseService::get_connection()?;
        let changed = conn
            .execute(
                "UPDATE games SET rom_path = ?1, file_size_bytes = ?2 WHERE id = ?3;",
                params![rom_path, file_size, game_id],
            )
            .map_err(|e| {
                EmuBoxError::StorageUnavailable(format!(
                    "Error marcando juego como instalado: {}",
                    e
                ))
            })?;
        if changed == 0 {
            return Err(EmuBoxError::NotFound(format!(
                "Juego no encontrado: {game_id}"
            )));
        }
        Ok(())
    }

    pub fn platform_name(platform_id: &str) -> String {
        PLATFORM_SPECS
            .iter()
            .find(|p| p.id == platform_id)
            .map(|p| p.name.to_string())
            .unwrap_or_else(|| platform_id.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installation_rejects_missing_game_and_oversized_file() {
        assert!(matches!(
            GameService::mark_installed("missing-installation-fixture", "/tmp/unused", 1),
            Err(EmuBoxError::NotFound(_))
        ));
        assert!(matches!(
            GameService::mark_installed("missing-installation-fixture", "/tmp/unused", u64::MAX),
            Err(EmuBoxError::InvalidConfiguration(_))
        ));
    }
}

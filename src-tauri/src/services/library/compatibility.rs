use crate::errors::EmuBoxError;
use crate::models::GameEmulatorAssociation;
use crate::models::{Emulator, Game};
use crate::services::db_service::DatabaseService;
use crate::services::EmulatorService;
use rusqlite::params;

pub struct CompatibilityService;

fn row_to_association(row: &rusqlite::Row<'_>) -> rusqlite::Result<GameEmulatorAssociation> {
    let args_json: String = row.get(4)?;
    let custom_arguments = serde_json::from_str(&args_json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(error))
    })?;
    Ok(GameEmulatorAssociation {
        game_id: row.get(0)?,
        emulator_id: row.get(1)?,
        is_default: row.get::<_, i32>(2)? == 1,
        priority: row.get(3)?,
        custom_arguments,
        custom_config_path: row.get(5)?,
        enabled: row.get::<_, i32>(6)? == 1,
    })
}

impl CompatibilityService {
    pub fn resolve_for_game(
        game: &Game,
        preferred_emulator_id: Option<&str>,
    ) -> Result<(Emulator, Vec<String>, Option<String>), EmuBoxError> {
        let available = EmulatorService::get_emulators()?
            .into_iter()
            .filter(|emulator| {
                emulator
                    .supported_platforms
                    .iter()
                    .any(|platform| platform == &game.platform)
            })
            .collect::<Vec<_>>();

        if available.is_empty() {
            return Err(EmuBoxError::EmulatorNotInstalled(format!(
                "No hay un emulador activo para {}",
                game.platform
            )));
        }

        let associations = Self::get_game_associations(game.id.clone())?;
        if let Some(id) = preferred_emulator_id {
            if !available.iter().any(|emulator| emulator.id == id) {
                return Err(EmuBoxError::EmulatorNotInstalled(format!(
                    "Emulador no disponible para {}: {id}",
                    game.platform
                )));
            }
        }
        let selected = preferred_emulator_id
            .and_then(|id| available.iter().find(|emulator| emulator.id == id))
            .or_else(|| {
                associations
                    .iter()
                    .filter(|association| association.enabled)
                    .find_map(|association| {
                        available
                            .iter()
                            .find(|emulator| emulator.id == association.emulator_id)
                    })
            })
            .or_else(|| {
                available
                    .iter()
                    .find(|emulator| emulator.compatibility.status == "supported")
            })
            .or_else(|| available.first())
            .cloned()
            .ok_or_else(|| {
                EmuBoxError::EmulatorNotInstalled("No se pudo resolver el emulador".to_string())
            })?;

        if selected.compatibility.status != "supported" {
            return Err(EmuBoxError::GameLaunchFailed(
                selected.compatibility.reason.clone(),
            ));
        }
        let association = associations
            .into_iter()
            .find(|association| association.enabled && association.emulator_id == selected.id);
        let custom_args = association
            .as_ref()
            .map(|association| association.custom_arguments.clone())
            .unwrap_or_default();
        let custom_config = association.and_then(|association| association.custom_config_path);

        Ok((selected, custom_args, custom_config))
    }

    pub fn get_game_associations(
        game_id: String,
    ) -> Result<Vec<GameEmulatorAssociation>, EmuBoxError> {
        let conn = DatabaseService::get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT game_id, emulator_id, is_default, priority, custom_arguments_json, custom_config_path, enabled
             FROM game_emulator_associations
             WHERE game_id = ?1
             ORDER BY is_default DESC, priority DESC;"
        ).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;

        let rows = stmt
            .query_map(params![game_id], row_to_association)
            .map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))
    }

    pub fn set_game_association(association: GameEmulatorAssociation) -> Result<(), EmuBoxError> {
        let mut conn = DatabaseService::get_connection()?;
        Self::set_game_association_on(&mut conn, association)
    }

    fn set_game_association_on(
        connection: &mut rusqlite::Connection,
        association: GameEmulatorAssociation,
    ) -> Result<(), EmuBoxError> {
        let conn = connection
            .transaction()
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        let args_json = serde_json::to_string(&association.custom_arguments)
            .map_err(|error| EmuBoxError::InvalidConfiguration(error.to_string()))?;

        // Si se marca como default, desmarcar cualquier otra asociación previa del mismo juego
        if association.is_default {
            conn.execute(
                "UPDATE game_emulator_associations SET is_default = 0 WHERE game_id = ?1;",
                params![association.game_id],
            )
            .map_err(|e| {
                EmuBoxError::StorageUnavailable(format!(
                    "Error al actualizar default previo: {}",
                    e
                ))
            })?;
        }

        conn.execute(
            "INSERT INTO game_emulator_associations (game_id, emulator_id, is_default, priority, custom_arguments_json, custom_config_path, enabled)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(game_id, emulator_id) DO UPDATE SET
               is_default = excluded.is_default,
               priority = excluded.priority,
               custom_arguments_json = excluded.custom_arguments_json,
               custom_config_path = excluded.custom_config_path,
               enabled = excluded.enabled;",
            params![
                association.game_id,
                association.emulator_id,
                if association.is_default { 1 } else { 0 },
                association.priority,
                args_json,
                association.custom_config_path,
                if association.enabled { 1 } else { 0 }
            ]
        ).map_err(|e| EmuBoxError::StorageUnavailable(format!("Error al guardar asociación juego ↔ emulador en SQLite: {}", e)))?;

        conn.commit()
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))
    }

    pub fn remove_game_association(
        game_id: String,
        emulator_id: String,
    ) -> Result<(), EmuBoxError> {
        let conn = DatabaseService::get_connection()?;
        conn.execute(
            "DELETE FROM game_emulator_associations WHERE game_id = ?1 AND emulator_id = ?2;",
            params![game_id, emulator_id],
        )
        .map_err(|e| {
            EmuBoxError::StorageUnavailable(format!(
                "Error al eliminar asociación de SQLite: {}",
                e
            ))
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_arguments_are_not_silently_discarded() {
        let connection = rusqlite::Connection::open_in_memory().unwrap();
        assert!(connection
            .query_row(
                "SELECT 'game','emulator',1,0,'not json',NULL,1",
                [],
                row_to_association
            )
            .is_err());
    }
    #[test]
    fn failed_preference_change_preserves_previous_default() {
        let mut connection = rusqlite::Connection::open_in_memory().unwrap();
        connection.execute_batch("CREATE TABLE game_emulator_associations (game_id TEXT, emulator_id TEXT CHECK(emulator_id != 'invalid'), is_default INTEGER, priority INTEGER, custom_arguments_json TEXT, custom_config_path TEXT, enabled INTEGER, PRIMARY KEY(game_id,emulator_id)); INSERT INTO game_emulator_associations VALUES ('game','previous',1,0,'[]',NULL,1);").unwrap();
        let association = GameEmulatorAssociation {
            game_id: "game".into(),
            emulator_id: "invalid".into(),
            is_default: true,
            priority: 0,
            custom_arguments: vec![],
            custom_config_path: None,
            enabled: true,
        };
        assert!(
            CompatibilityService::set_game_association_on(&mut connection, association).is_err()
        );
        let default: i32 = connection
            .query_row(
                "SELECT is_default FROM game_emulator_associations WHERE emulator_id='previous'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(default, 1);
    }
}

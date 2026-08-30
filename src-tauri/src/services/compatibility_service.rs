use rusqlite::params;
use crate::models::GameEmulatorAssociation;
use crate::errors::EmuBoxError;
use crate::services::db_service::DatabaseService;

pub struct CompatibilityService;

impl CompatibilityService {
    pub fn get_game_associations(game_id: String) -> Result<Vec<GameEmulatorAssociation>, EmuBoxError> {
        let conn = DatabaseService::get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT game_id, emulator_id, is_default, priority, custom_arguments_json, custom_config_path, enabled
             FROM game_emulator_associations
             WHERE game_id = ?1
             ORDER BY is_default DESC, priority DESC;"
        ).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;

        let rows = stmt.query_map(params![game_id], |row| {
            let game_id: String = row.get(0)?;
            let emulator_id: String = row.get(1)?;
            let is_default_int: i32 = row.get(2)?;
            let priority: i32 = row.get(3)?;
            let args_json: String = row.get(4)?;
            let custom_config_path: Option<String> = row.get(5)?;
            let enabled_int: i32 = row.get(6)?;

            let custom_arguments: Vec<String> = serde_json::from_str(&args_json).unwrap_or_default();

            Ok(GameEmulatorAssociation {
                game_id,
                emulator_id,
                is_default: is_default_int == 1,
                priority,
                custom_arguments,
                custom_config_path,
                enabled: enabled_int == 1,
            })
        }).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            if let Ok(assoc) = r {
                list.push(assoc);
            }
        }

        Ok(list)
    }

    pub fn set_game_association(association: GameEmulatorAssociation) -> Result<(), EmuBoxError> {
        let conn = DatabaseService::get_connection()?;
        let args_json = serde_json::to_string(&association.custom_arguments)
            .unwrap_or_else(|_| "[]".to_string());

        // Si se marca como default, desmarcar cualquier otra asociación previa del mismo juego
        if association.is_default {
            conn.execute(
                "UPDATE game_emulator_associations SET is_default = 0 WHERE game_id = ?1;",
                params![association.game_id]
            ).map_err(|e| EmuBoxError::StorageUnavailable(format!("Error al actualizar default previo: {}", e)))?;
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

        Ok(())
    }

    pub fn remove_game_association(game_id: String, emulator_id: String) -> Result<(), EmuBoxError> {
        let conn = DatabaseService::get_connection()?;
        conn.execute(
            "DELETE FROM game_emulator_associations WHERE game_id = ?1 AND emulator_id = ?2;",
            params![game_id, emulator_id]
        ).map_err(|e| EmuBoxError::StorageUnavailable(format!("Error al eliminar asociación de SQLite: {}", e)))?;

        Ok(())
    }
}

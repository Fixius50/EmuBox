use rusqlite::params;
use crate::models::GameEmulatorAssociation;
use crate::models::{Emulator, Game};
use crate::errors::EmuBoxError;
use crate::services::db_service::DatabaseService;
use crate::services::EmulatorService;

pub struct CompatibilityService;

impl CompatibilityService {
    pub fn resolve_for_game(
        game: &Game,
        preferred_emulator_id: Option<&str>,
    ) -> Result<(Emulator, Vec<String>, Option<String>), EmuBoxError> {
        let available = EmulatorService::get_emulators()?
            .into_iter()
            .filter(|emulator| {
                emulator.supported_platforms.iter().any(|platform| platform == &game.platform)
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
                return Err(EmuBoxError::EmulatorNotInstalled(format!("Emulador no disponible para {}: {id}", game.platform)));
            }
        }
        let selected = preferred_emulator_id
            .and_then(|id| available.iter().find(|emulator| emulator.id == id))
            .or_else(|| {
                associations.iter()
                    .filter(|association| association.enabled)
                    .find_map(|association| available.iter().find(|emulator| emulator.id == association.emulator_id))
            })
            .or_else(|| available.iter().find(|emulator| emulator.compatibility.status == "supported"))
            .or_else(|| available.first())
            .cloned()
            .ok_or_else(|| EmuBoxError::EmulatorNotInstalled("No se pudo resolver el emulador".to_string()))?;

        if selected.compatibility.status != "supported" {
            return Err(EmuBoxError::GameLaunchFailed(selected.compatibility.reason.clone()));
        }
        let association = associations.into_iter()
            .find(|association| association.enabled && association.emulator_id == selected.id);
        let custom_args = association.as_ref().map(|association| association.custom_arguments.clone()).unwrap_or_default();
        let custom_config = association.and_then(|association| association.custom_config_path);

        Ok((selected, custom_args, custom_config))
    }

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
        for association in rows.flatten() {
            list.push(association);
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

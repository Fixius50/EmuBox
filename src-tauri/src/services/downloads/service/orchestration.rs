use super::DownloadService;
use crate::errors::EmuBoxError;
use crate::{
    models::{CreateDownloadRequest, DownloadJob},
    services::db_service::DatabaseService,
};
use rusqlite::params;

impl DownloadService {
    pub fn import_and_start() -> Result<Vec<DownloadJob>, EmuBoxError> {
        Self::import_link_file()?;
        Self::list_jobs()
    }

    /// Descarga un juego de catálogo a partir de su `gameId`, reutilizando la fuente
    /// importada desde el manifiesto o creada manualmente para ese juego.
    pub fn download_game(game_id: String) -> Result<DownloadJob, EmuBoxError> {
        Self::download_game_from_source(game_id, None)
    }

    pub fn download_game_from_source(
        game_id: String,
        source_id: Option<String>,
    ) -> Result<DownloadJob, EmuBoxError> {
        let conn = DatabaseService::get_connection()?;
        let platform: String = conn
            .query_row(
                "SELECT platform_id FROM games WHERE id = ?1",
                params![game_id],
                |row| row.get(0),
            )
            .map_err(|_| {
                EmuBoxError::NotFound(format!("Juego no encontrado en el catálogo: {}", game_id))
            })?;

        let sources = Self::list_sources(&game_id)?;
        let chosen = if let Some(source_id) = source_id {
            sources
                .into_iter()
                .find(|option| option.source.id == source_id)
                .ok_or_else(|| {
                    EmuBoxError::NotFound("La fuente no pertenece a este juego".into())
                })?
        } else {
            if sources.len() > 1 {
                return Err(EmuBoxError::InvalidConfiguration(
                    "Selecciona una fuente; las URLs pueden ser partes o versiones distintas"
                        .into(),
                ));
            }
            sources.into_iter().next().ok_or_else(|| {
                EmuBoxError::NotFound("No hay fuentes registradas para este juego".into())
            })?
        };
        if !chosen.downloadable {
            return Err(EmuBoxError::InvalidConfiguration(
                chosen
                    .reason
                    .unwrap_or_else(|| "Fuente no disponible".into()),
            ));
        }
        let source = chosen.source;

        let job = Self::create_job(CreateDownloadRequest {
            game_id,
            platform,
            source,
        })?;
        Self::start(job.id)
    }

    pub fn start(id: String) -> Result<DownloadJob, EmuBoxError> {
        crate::services::download_manager::start(id)
    }

    pub fn pause(id: &str) -> Result<DownloadJob, EmuBoxError> {
        crate::services::download_manager::pause(id)
    }

    pub fn resume(id: String) -> Result<DownloadJob, EmuBoxError> {
        crate::services::download_manager::resume(id)
    }

    pub fn cancel(id: &str) -> Result<DownloadJob, EmuBoxError> {
        crate::services::download_manager::cancel(id)
    }
}

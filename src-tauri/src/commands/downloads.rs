use crate::errors::EmuBoxError;
use crate::models::{CreateDownloadRequest, DownloadJob, DownloadSource};
use crate::services::DownloadService;

#[tauri::command]
pub fn import_download_links() -> Result<Vec<DownloadJob>, EmuBoxError> {
    DownloadService::import_link_file()
}

#[tauri::command]
pub fn import_and_start_downloads() -> Result<Vec<DownloadJob>, EmuBoxError> {
    DownloadService::import_and_start()
}

#[tauri::command]
pub fn create_download_source(source: DownloadSource) -> Result<DownloadSource, EmuBoxError> {
    DownloadService::create_source(source)
}

#[tauri::command]
pub fn create_download_job(request: CreateDownloadRequest) -> Result<DownloadJob, EmuBoxError> {
    DownloadService::create_job(request)
}

#[tauri::command]
pub fn get_download_jobs() -> Result<Vec<DownloadJob>, EmuBoxError> {
    DownloadService::list_jobs()
}

#[tauri::command]
pub fn start_download(id: String) -> Result<DownloadJob, EmuBoxError> {
    DownloadService::start(id)
}

#[tauri::command]
pub fn pause_download(id: String) -> Result<DownloadJob, EmuBoxError> {
    DownloadService::pause(&id)
}

#[tauri::command]
pub fn resume_download(id: String) -> Result<DownloadJob, EmuBoxError> {
    DownloadService::resume(id)
}

#[tauri::command]
pub fn cancel_download(id: String) -> Result<DownloadJob, EmuBoxError> {
    DownloadService::cancel(&id)
}

#[tauri::command]
pub fn download_game(game_id: String) -> Result<DownloadJob, EmuBoxError> {
    DownloadService::download_game(game_id)
}
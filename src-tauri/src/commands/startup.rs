use crate::{
    errors::EmuBoxError,
    services::runtime::startup::{Report, Startup, StartupData},
};

#[tauri::command]
pub fn get_startup_status(startup: tauri::State<'_, Startup>) -> Report {
    startup.report()
}

#[tauri::command]
pub async fn get_startup_data(
    startup: tauri::State<'_, Startup>,
) -> Result<StartupData, EmuBoxError> {
    let startup = startup.inner().clone();
    super::blocking(move || startup.data()).await
}

#[tauri::command]
pub fn startup_frontend_ready(
    startup: tauri::State<'_, Startup>,
    app: tauri::AppHandle,
) -> Result<Report, EmuBoxError> {
    let report = startup.frontend_ready()?;
    eprintln!(
        "[Startup] interfaz preparada: {:?} en {} ms",
        report.phase, report.elapsed_ms
    );
    crate::api::events::startup_status(&app, &report);
    Ok(report)
}

use crate::models::{DiagnosticReport, LogEntry};
use crate::services::DiagnosticsService;
use crate::errors::EmuBoxError;

/// Sonda de un solo uso: confirma que el puente IPC del webview llega a Rust,
/// para descartar fallos de detección de entorno Tauri en el frontend.
#[tauri::command]
pub fn frontend_probe(message: String) -> Result<(), EmuBoxError> {
    eprintln!("[PROBE] {}", message);
    Ok(())
}

#[tauri::command]
pub fn get_system_logs(limit: Option<usize>) -> Result<Vec<LogEntry>, EmuBoxError> {
    DiagnosticsService::get_system_logs(limit)
}

#[tauri::command]
pub fn get_emubox_logs(limit: Option<usize>) -> Result<Vec<LogEntry>, EmuBoxError> {
    DiagnosticsService::get_emubox_logs(limit)
}

#[tauri::command]
pub fn get_diagnostics() -> Result<DiagnosticReport, EmuBoxError> {
    DiagnosticsService::get_diagnostics()
}

#[tauri::command]
pub fn execute_command(command: String) -> Result<String, EmuBoxError> {
    DiagnosticsService::execute_command(&command)
}

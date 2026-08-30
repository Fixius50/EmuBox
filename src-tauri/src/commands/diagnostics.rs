use crate::models::{DiagnosticReport, LogEntry};
use crate::services::DiagnosticsService;
use crate::errors::EmuBoxError;

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

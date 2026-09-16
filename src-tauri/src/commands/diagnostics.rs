use crate::errors::EmuBoxError;
use crate::models::{DiagnosticReport, LogEntry};
use crate::services::DiagnosticsService;

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

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FrontendLogEvent {
    event: String,
    level: String,
    timestamp_ms: f64,
    elapsed_ms: f64,
    message: String,
    data: serde_json::Value,
}

#[tauri::command]
pub fn record_frontend_events(entries: Vec<FrontendLogEvent>) -> Result<(), EmuBoxError> {
    if entries.len() > 32 {
        return Err(EmuBoxError::InvalidConfiguration(
            "Lote de logs excesivo".into(),
        ));
    }
    for entry in &entries {
        if entry.message.len() > 8192
            || entry.data.to_string().len() > 4096
            || !entry.timestamp_ms.is_finite()
            || !entry.elapsed_ms.is_finite()
            || ![
                "console.message",
                "input.click",
                "input.action",
                "window.focus",
                "window.visibility",
                "window.resize",
                "window.error",
                "promise.rejection",
                "graphics.context-lost",
                "incident.mark",
                "render.sample",
                "session.start",
                "ipc.complete",
            ]
            .contains(&entry.event.as_str())
            || entry.level.parse::<log::Level>().is_err()
        {
            return Err(EmuBoxError::InvalidConfiguration(
                "Evento de UI invalido".into(),
            ));
        }
    }
    for entry in entries {
        crate::services::infrastructure::telemetry::event(
            entry.level.parse().unwrap_or(log::Level::Info),
            match entry.event.as_str() {
                "input.click" | "input.action" => "ui.input",
                "render.sample" | "graphics.context-lost" => "ui.render",
                "ipc.complete" => "ui.ipc",
                "console.message" => "ui.console",
                _ => "ui",
            },
            &entry.event,
            &entry.message,
            serde_json::json!({"clientTimestampMs": entry.timestamp_ms, "clientElapsedMs": entry.elapsed_ms, "context": entry.data}),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontend_logs_reject_unbounded_batches_and_unknown_events() {
        let fixture = |name: &str| FrontendLogEvent {
            event: name.into(),
            level: "info".into(),
            timestamp_ms: 1000.0,
            elapsed_ms: 1.0,
            message: String::new(),
            data: serde_json::json!({"button": 0}),
        };
        assert!(record_frontend_events(vec![fixture("input.click")]).is_ok());
        assert!(record_frontend_events(vec![fixture("os.fake")]).is_err());
        assert!(record_frontend_events((0..33).map(|_| fixture("input.click")).collect()).is_err());
        let mut oversized = fixture("window.error");
        oversized.message = "x".repeat(8193);
        assert!(record_frontend_events(vec![oversized]).is_err());
    }
}

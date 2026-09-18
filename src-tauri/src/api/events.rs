use crate::services::runtime::startup::Report;
use serde::Serialize;
use tauri::Emitter;

pub const STARTUP_STATUS: &str = "startup-status";
pub const LIBRARY_UPDATED: &str = "library-updated";
pub const WINDOW_RESIZED: &str = "emubox://window-resized";

pub fn startup_status(target: &impl Emitter, report: &Report) {
    let _ = target.emit(STARTUP_STATUS, report);
}

pub fn library_updated(target: &impl Emitter, payload: impl Serialize) {
    let _ = target.emit(LIBRARY_UPDATED, payload);
}

pub fn window_resized(target: &impl Emitter, width: u32, height: u32) {
    let _ = target.emit(
        WINDOW_RESIZED,
        serde_json::json!({ "width": width, "height": height }),
    );
}

use crate::services::runtime::startup::Report;
use serde::Serialize;
use tauri::{Emitter, Runtime};

pub const STARTUP_STATUS: &str = "startup-status";
pub const LIBRARY_UPDATED: &str = "library-updated";
pub const WINDOW_RESIZED: &str = "emubox://window-resized";

pub fn startup_status<R: Runtime>(target: &impl Emitter<R>, report: &Report) {
    let _ = target.emit(STARTUP_STATUS, report);
}

pub fn library_updated<R: Runtime>(target: &impl Emitter<R>, payload: impl Serialize + Clone) {
    let _ = target.emit(LIBRARY_UPDATED, payload);
}

pub fn window_resized<R: Runtime>(target: &impl Emitter<R>, width: u32, height: u32) {
    let _ = target.emit(
        WINDOW_RESIZED,
        serde_json::json!({ "width": width, "height": height }),
    );
}

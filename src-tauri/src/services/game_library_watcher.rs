use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;
use std::thread;
use notify::{Watcher, RecommendedWatcher, RecursiveMode, Event, EventKind};
use tauri::Emitter;
use crate::services::game_service::GameService;

pub struct GameLibraryWatcher;

impl GameLibraryWatcher {
    /// Inicia el watcher reactivo de eventos del sistema de archivos en segundo plano.
    /// No realiza polling: se bloquea en el canal de eventos de notify (inotify en Linux).
    pub fn start_watching(watch_path: Option<PathBuf>, app_handle: Option<tauri::AppHandle>) {
        thread::spawn(move || {
            let target_dir = watch_path.unwrap_or_else(GameService::get_canonical_games_dir);
            if !target_dir.exists() {
                let _ = std::fs::create_dir_all(&target_dir);
            }

            let (tx, rx) = channel();

            let mut watcher = match RecommendedWatcher::new(tx, notify::Config::default()) {
                Ok(w) => w,
                Err(e) => {
                    log::error!("[GameLibraryWatcher] Error inicializando notify watcher: {}", e);
                    return;
                }
            };

            if let Err(e) = watcher.watch(&target_dir, RecursiveMode::Recursive) {
                log::error!("[GameLibraryWatcher] Error observando directorio {}: {}", target_dir.display(), e);
                return;
            }

            log::info!("[GameLibraryWatcher] Observando eventos reactivos en: {}", target_dir.display());

            while let Ok(res) = rx.recv() {
                match res {
                    Ok(event) => {
                        if Self::is_relevant_event(&event) {
                            log::info!("[GameLibraryWatcher] Evento de sistema de archivos detectado ({:?}), sincronizando biblioteca...", event.kind);
                            thread::sleep(Duration::from_millis(500));
                            while rx.try_recv().is_ok() {}

                            if let Ok(result) = GameService::scan_games(None) {
                                if let Some(handle) = &app_handle {
                                    let _ = handle.emit("library-updated", serde_json::json!({
                                        "scannedCount": result.scanned_count,
                                        "addedCount": result.added_count,
                                        "updatedCount": result.updated_count,
                                        "removedCount": result.removed_count,
                                        "totalCount": result.total_count,
                                        "timestamp": std::time::SystemTime::now()
                                            .duration_since(std::time::UNIX_EPOCH)
                                            .unwrap_or_default()
                                            .as_millis()
                                    }));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("[GameLibraryWatcher] Error en evento de archivo: {}", e);
                    }
                }
            }
        });
    }

    fn is_relevant_event(event: &Event) -> bool {
        match event.kind {
            EventKind::Create(_) | EventKind::Remove(_) | EventKind::Modify(_) => {
                event.paths.iter().any(|p| {
                    if p.is_dir() {
                        return true;
                    }
                    if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                        let lower = ext.to_lowercase();
                        matches!(lower.as_str(), "iso" | "chd" | "cso" | "bin" | "cue" | "rvz" | "gcm" | "sfc" | "smc" | "gba" | "z64" | "n64" | "md" | "gen" | "cdi" | "pbp" | "nds" | "pkg" | "zip" | "7z")
                    } else {
                        false
                    }
                })
            }
            _ => false,
        }
    }
}

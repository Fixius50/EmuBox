pub mod api;
pub mod commands;
pub mod errors;
pub mod models;
pub mod services;
pub mod state;

use services::infrastructure::telemetry;
use state::AppState;
use tauri::Manager;

/// Intervalo entre comprobaciones periódicas de los manifiestos de descarga.
const MANIFEST_POLL_INTERVAL_SECS: u64 = 6 * 60 * 60;

pub fn run() {
    services::infrastructure::telemetry::init();
    tauri::Builder::default()
        .manage(AppState::new())
        .manage(services::runtime::startup::Startup::new())
        .setup(|app| {
            let app_handle = app.handle().clone();
            let notify_handle = app_handle.clone();
            app.state::<services::runtime::startup::Startup>().start(
                std::sync::Arc::new(move |report| {
                    eprintln!("[Startup] {:?}: {} ms", report.phase, report.elapsed_ms);
                    telemetry::event(
                        if report.error.is_some() {
                            log::Level::Error
                        } else {
                            log::Level::Info
                        },
                        "startup",
                        "startup.status",
                        "Estado del coordinador",
                        serde_json::to_value(&report).unwrap_or_default(),
                    );
                    api::events::startup_status(&notify_handle, &report);
                }),
                move || {
                    services::GameLibraryWatcher::start_watching(None, Some(app_handle.clone()));
                    match telemetry::operation("library.scan", "initial-scan", || {
                        services::GameService::scan_games(None)
                    }) {
                        Ok(scan) => {
                            if scan.added_count > 0
                                || scan.updated_count > 0
                                || scan.removed_count > 0
                            {
                                api::events::library_updated(
                                    &app_handle,
                                    serde_json::json!({ "reason": "initial-scan" }),
                                );
                            }
                            if !scan.errors.is_empty() {
                                eprintln!(
                                    "[Library] Escaneo inicial incompleto: {:?}",
                                    scan.errors
                                );
                            }
                        }
                        Err(error) => eprintln!("[Library] Escaneo inicial: {error}"),
                    }
                    match telemetry::operation(
                        "catalog.index",
                        "local-index",
                        services::game_database::ensure_local_index,
                    ) {
                        Ok(count) if count > 0 => {
                            api::events::library_updated(
                                &app_handle,
                                serde_json::json!({
                                    "reason": "local-game-index", "matchedCount": count
                                }),
                            );
                        }
                        Ok(_) => (),
                        Err(error) => {
                            eprintln!("[Game Database] Indice local tras preparar UI: {error}")
                        }
                    }
                    match telemetry::operation("catalog.database", "master-sync", || {
                        services::game_database::sync_all(|platform, count| {
                            api::events::library_updated(&app_handle, serde_json::json!({
                        "reason": "game-database", "platform": platform, "canonicalCount": count
                    }));
                        })
                    }) {
                        Ok(count) => {
                            eprintln!("[Game Database] {count} juegos canonicos actualizados")
                        }
                        Err(error) => eprintln!("[Game Database] {error}"),
                    }
                    if let Err(error) =
                        telemetry::operation("catalog.manifests", "initial-sync", || {
                            services::DownloadService::import_link_file_with_progress(|| {
                                api::events::library_updated(
                                    &app_handle,
                                    serde_json::json!({ "reason": "manifest-import" }),
                                );
                            })
                        })
                    {
                        eprintln!("[Catalog] {error}");
                    }
                    loop {
                        std::thread::sleep(std::time::Duration::from_secs(
                            MANIFEST_POLL_INTERVAL_SECS,
                        ));
                        if let Err(error) =
                            telemetry::operation("catalog.manifests", "periodic-sync", || {
                                services::DownloadService::import_link_file_with_progress(|| {
                                    api::events::library_updated(
                                        &app_handle,
                                        serde_json::json!({ "reason": "manifest-import" }),
                                    );
                                })
                            })
                        {
                            eprintln!("[Catalog] {error}");
                        }
                    }
                },
            );

            Ok(())
        })
        .invoke_handler(crate::emubox_ipc_handler!())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Resized(size) = event {
                api::events::window_resized(window, size.width, size.height);
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running emubox application");
}

pub mod commands;
pub mod errors;
pub mod models;
pub mod services;
pub mod state;

use services::infrastructure::telemetry;
use state::AppState;
use tauri::Emitter;
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
                    let _ = notify_handle.emit("startup-status", report);
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
                                let _ = app_handle.emit(
                                    "library-updated",
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
                            let _ = app_handle.emit(
                                "library-updated",
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
                            let _ = app_handle.emit("library-updated", serde_json::json!({
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
                                let _ = app_handle.emit(
                                    "library-updated",
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
                                    let _ = app_handle.emit(
                                        "library-updated",
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
        .invoke_handler(tauri::generate_handler![
            commands::startup::get_startup_status,
            commands::startup::get_startup_data,
            commands::startup::startup_frontend_ready,
            // System & Environment
            commands::system::get_system_info,
            commands::system::get_hardware_info,
            commands::system::get_display_info,
            commands::system::get_audio_info,
            commands::system::first_run_detection,
            commands::system::get_config,
            commands::system::save_config,
            commands::system::get_settings,
            commands::system::save_settings,
            commands::system::system_shutdown,
            commands::system::system_restart,
            commands::system::system_sleep,
            commands::system::system_logout,
            commands::system::restart_app_session,
            commands::system::exit_to_linux_shell,
            // Games
            commands::games::get_games,
            commands::games::get_game_by_id,
            commands::games::get_canonical_game_options,
            commands::games::scan_games,
            commands::games::get_platforms,
            commands::games::toggle_favorite,
            // Emulators
            commands::emulators::get_emulators,
            commands::emulators::get_emulator_by_id,
            commands::emulators::scan_emulators,
            commands::emulators::apply_hardware_profile,
            commands::emulators::get_emulator_status,
            commands::emulators::save_emulator,
            commands::emulators::delete_emulator,
            // Processes
            commands::processes::launch_game,
            commands::processes::stop_game,
            commands::processes::is_game_running,
            commands::processes::get_running_game,
            commands::processes::get_process_status,
            commands::processes::kill_process,
            // Storage
            commands::storage::get_storage_info,
            commands::storage::get_storage_locations,
            // Input
            commands::input::get_gamepads,
            commands::input::get_gamepad_status,
            // Diagnostics
            commands::diagnostics::get_system_logs,
            commands::diagnostics::get_emubox_logs,
            commands::diagnostics::get_diagnostics,
            commands::diagnostics::record_frontend_events,
            commands::diagnostics::frontend_probe,
            // BIOS
            commands::bios::get_bios_requirements,
            commands::bios::scan_bios,
            // Compatibility (Game <-> Emulator Associations)
            commands::compatibility::get_game_associations,
            commands::compatibility::set_game_association,
            commands::compatibility::remove_game_association,
            // Downloads (solo fuentes autorizadas proporcionadas por el usuario)
            commands::downloads::create_download_source,
            commands::downloads::create_download_job,
            commands::downloads::get_download_jobs,
            commands::downloads::start_download,
            commands::downloads::pause_download,
            commands::downloads::resume_download,
            commands::downloads::cancel_download,
            commands::downloads::get_download_candidates,
            commands::downloads::select_download_candidate,
            commands::downloads::import_download_links,
            commands::downloads::import_downloads_from_json,
            commands::downloads::import_downloads_from_url,
            commands::downloads::import_and_start_downloads,
            commands::downloads::download_game,
            commands::downloads::get_download_sources,
            commands::downloads::search_jackett,
            commands::downloads::select_jackett_result,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Resized(size) = event {
                use tauri::Emitter;
                let _ = window.emit(
                    "emubox://window-resized",
                    serde_json::json!({ "width": size.width, "height": size.height }),
                );
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running emubox application");
}

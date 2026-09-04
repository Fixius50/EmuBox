pub mod errors;
pub mod models;
pub mod services;
pub mod state;
pub mod commands;

use tauri::Emitter;
use state::AppState;

/// Intervalo entre comprobaciones periódicas de los manifiestos de descarga.
const MANIFEST_POLL_INTERVAL_SECS: u64 = 120;

pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .setup(|app| {
            let app_handle = app.handle().clone();

            // 1. Iniciar escaneo inicial en segundo plano sin bloquear arranque de UI
            std::thread::spawn(move || {
                let _ = services::EmulatorService::scan_emulators();
                if let Ok(hardware) = services::SystemService::get_hardware_info() {
                    let _ = services::EmulatorService::apply_hardware_profile(&hardware);
                }
                let _ = services::GameService::scan_games(None);
                let _ = services::DownloadService::import_and_start();
                let _ = app_handle.emit("library-updated", serde_json::json!({ "reason": "initial-scan" }));
            });

            // 2. Iniciar watcher reactivo del sistema de archivos (inotify / notify)
            services::GameLibraryWatcher::start_watching(None, Some(app.handle().clone()));

            // 3. Consultar periódicamente los manifiestos de descarga y añadir en
            // caliente el catálogo nuevo, sin esperar a reiniciar la consola.
            let poll_handle = app.handle().clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_secs(MANIFEST_POLL_INTERVAL_SECS));
                if let Ok(jobs) = services::DownloadService::import_and_start() {
                    let _ = poll_handle.emit("library-updated", serde_json::json!({
                        "reason": "periodic-manifest-check",
                        "jobCount": jobs.len()
                    }));
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
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
            commands::system::exit_to_linux_shell,

            // Games
            commands::games::get_games,
            commands::games::get_game_by_id,
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
            commands::diagnostics::execute_command,
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
            commands::downloads::import_download_links,
            commands::downloads::import_and_start_downloads,
            commands::downloads::download_game,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Resized(size) = event {
                use tauri::Emitter;
                let _ = window.emit(
                    "emubox://window-resized",
                    serde_json::json!({ "width": size.width, "height": size.height })
                );
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running emubox application");
}

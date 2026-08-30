pub mod errors;
pub mod models;
pub mod services;
pub mod state;
pub mod commands;

use state::AppState;

pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .setup(|_app| {
            // 1. Iniciar escaneo inicial en segundo plano sin bloquear arranque de UI
            std::thread::spawn(|| {
                let _ = services::EmulatorService::scan_emulators();
                let _ = services::GameService::scan_games(None);
            });

            // 2. Iniciar watcher reactivo del sistema de archivos (inotify / notify)
            services::GameLibraryWatcher::start_watching(None);

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

            // Diagnostics & Terminal
            commands::diagnostics::get_system_logs,
            commands::diagnostics::get_emubox_logs,
            commands::diagnostics::get_diagnostics,
            commands::diagnostics::execute_command,

            // BIOS
            commands::bios::get_bios_requirements,
            commands::bios::scan_bios,

            // Compatibility (Game <-> Emulator Associations)
            commands::compatibility::get_game_associations,
            commands::compatibility::set_game_association,
            commands::compatibility::remove_game_association,
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

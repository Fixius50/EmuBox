// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("--graphics-info") => {
            match serde_json::to_string_pretty(&emubox_lib::services::graphics_service::detect()) {
                Ok(json) => println!("{json}"),
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            }
            return;
        }
        Some("--graphics-session") => {
            print!(
                "{}",
                emubox_lib::services::graphics_service::session_environment()
            );
            return;
        }
        _ => {}
    }
    if std::env::args().nth(1).as_deref() == Some("--import-catalog") {
        match emubox_lib::services::DownloadService::import_link_file() {
            Ok(sources) => println!("Catalogo sincronizado: {} fuentes modificadas; cache conservada, sin iniciar descargas", sources.len()),
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(1);
            }
        }
        return;
    }
    emubox_lib::run();
}

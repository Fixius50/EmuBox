// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if std::env::args().nth(1).as_deref() == Some("--import-catalog") {
        match emubox_lib::services::DownloadService::import_link_file() {
            Ok(sources) => println!("Catalogo actualizado: {} fuentes; sin iniciar descargas", sources.len()),
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(1);
            }
        }
        return;
    }
    emubox_lib::run();
}

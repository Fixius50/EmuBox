use crate::errors::EmuBoxError;
use crate::models::{CreateDownloadRequest, DownloadJob, DownloadSource};
use crate::services::DownloadService;
use std::{fs, net::{SocketAddr, TcpStream}, os::unix::fs::PermissionsExt, path::Path, sync::atomic::{AtomicBool, Ordering}, time::Duration};
use tauri::Manager;

fn jackett_url_allowed(url: &tauri::Url) -> bool {
    url.scheme() == "http" && url.host_str() == Some("127.0.0.1") && url.port_or_known_default() == Some(9117)
}

fn jackett_password() -> Result<String, EmuBoxError> {
    let path = Path::new("/var/lib/emubox/jackett/admin-password");
    let metadata = fs::symlink_metadata(path)
        .map_err(|_| EmuBoxError::ProcessFailed("No se puede leer la contraseña local de Jackett".into()))?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > 256 || metadata.permissions().mode() & 0o007 != 0 {
        return Err(EmuBoxError::ProcessFailed("Contraseña local de Jackett insegura o demasiado larga".into()));
    }
    let password = fs::read_to_string(path)
        .map_err(|_| EmuBoxError::ProcessFailed("No se puede leer la contraseña local de Jackett".into()))?
        .trim_end_matches(['\r', '\n']).to_owned();
    if password.is_empty() {
        return Err(EmuBoxError::ProcessFailed("Contraseña local de Jackett vacía".into()));
    }
    Ok(password)
}

fn jackett_login_script(password: &str) -> String {
    let value = serde_json::to_string(password).expect("Una cadena válida siempre se serializa");
    format!("(() => {{ const input = document.querySelector('form input[name=password]'); if (input) {{ input.value = {value}; input.form.requestSubmit(); }} }})()")
}

fn should_auto_login(event: tauri::webview::PageLoadEvent, url: &tauri::Url, attempted: &AtomicBool) -> bool {
    event == tauri::webview::PageLoadEvent::Finished
        && jackett_url_allowed(url)
        && url.path() == "/UI/Login"
        && !attempted.swap(true, Ordering::AcqRel)
}

#[tauri::command]
pub fn open_jackett(app: tauri::AppHandle) -> Result<(), EmuBoxError> {
    if !std::path::Path::new("/opt/jackett/jackett").is_file() {
        return Err(EmuBoxError::ProcessFailed("Jackett no esta instalado; configura el servicio local desde el terminal".into()));
    }
    let address: SocketAddr = "127.0.0.1:9117".parse().expect("Direccion Jackett fija");
    TcpStream::connect_timeout(&address, Duration::from_millis(500))
        .map_err(|_| EmuBoxError::ProcessFailed("Jackett local no responde en 127.0.0.1:9117".into()))?;
    if let Some(window) = app.get_webview_window("jackett") {
        return window.set_focus().map_err(|error| EmuBoxError::ProcessFailed(error.to_string()));
    }
    let password = jackett_password()?;
    let attempted = AtomicBool::new(false);
    let url = tauri::Url::parse("http://127.0.0.1:9117")
        .map_err(|error| EmuBoxError::ProcessFailed(error.to_string()))?;
    tauri::WebviewWindowBuilder::new(&app, "jackett", tauri::WebviewUrl::External(url))
        .title("Jackett local")
        .inner_size(1200.0, 800.0)
        .on_navigation(jackett_url_allowed)
        .on_page_load(move |window, payload| {
            if should_auto_login(payload.event(), payload.url(), &attempted) {
                let _ = window.eval(jackett_login_script(&password));
            }
        })
        .on_new_window(|_, _| tauri::webview::NewWindowResponse::Deny)
        .build()
        .map_err(|error| EmuBoxError::ProcessFailed(error.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod jackett_window_tests {
    use super::{jackett_login_script, jackett_url_allowed, should_auto_login};
    use std::sync::atomic::AtomicBool;

    #[test]
    fn window_only_navigates_within_local_jackett() {
        for url in ["http://127.0.0.1:9117", "http://127.0.0.1:9117/UI/Dashboard"] {
            assert!(jackett_url_allowed(&tauri::Url::parse(url).unwrap()));
        }
        for url in ["https://127.0.0.1:9117", "http://127.0.0.1:8080", "http://localhost:9117", "http://127.0.0.1:9117.evil.test", "http://127.0.0.1:9117@evil.test"] {
            assert!(!tauri::Url::parse(url).is_ok_and(|parsed| jackett_url_allowed(&parsed)));
        }
    }

    #[test]
    fn login_script_quotes_password_as_data_not_code() {
        let script = jackett_login_script("fixture\";alert(1);//");
        assert!(script.contains("input.value = \"fixture\\\";alert(1);//\""));
        assert!(script.contains("input.form.requestSubmit()"));
    }

    #[test]
    fn login_is_attempted_once_on_finished_local_form_only() {
        let attempted = AtomicBool::new(false);
        let login = tauri::Url::parse("http://127.0.0.1:9117/UI/Login?cookiesChecked=1").unwrap();
        let dashboard = tauri::Url::parse("http://127.0.0.1:9117/UI/Dashboard").unwrap();
        let external = tauri::Url::parse("http://localhost:9117/UI/Login").unwrap();
        use tauri::webview::PageLoadEvent::{Finished, Started};
        assert!(!should_auto_login(Started, &login, &attempted));
        assert!(!should_auto_login(Finished, &dashboard, &attempted));
        assert!(!should_auto_login(Finished, &external, &attempted));
        assert!(should_auto_login(Finished, &login, &attempted));
        assert!(!should_auto_login(Finished, &login, &attempted));
    }
}

#[tauri::command]
pub async fn search_jackett(
    game_id: String,
) -> Result<Vec<crate::models::JackettResult>, EmuBoxError> {
    super::blocking(move || crate::services::downloads::jackett::search(&game_id)).await
}

#[tauri::command]
pub async fn select_jackett_result(
    game_id: String,
    result_id: String,
) -> Result<DownloadSource, EmuBoxError> {
    super::blocking(move || crate::services::downloads::jackett::select(&game_id, &result_id)).await
}

#[tauri::command]
pub fn import_download_links() -> Result<Vec<DownloadSource>, EmuBoxError> {
    DownloadService::import_link_file()
}

#[tauri::command]
pub fn import_downloads_from_json(
    json_content: String,
) -> Result<Vec<DownloadSource>, EmuBoxError> {
    DownloadService::import_from_json(&json_content)
}

#[tauri::command]
pub fn import_downloads_from_url(url: String) -> Result<Vec<DownloadSource>, EmuBoxError> {
    DownloadService::import_from_url(&url)
}

#[tauri::command]
pub fn import_and_start_downloads() -> Result<Vec<DownloadJob>, EmuBoxError> {
    DownloadService::import_and_start()
}

#[tauri::command]
pub fn create_download_source(source: DownloadSource) -> Result<DownloadSource, EmuBoxError> {
    DownloadService::create_source(source)
}

#[tauri::command]
pub fn create_download_job(request: CreateDownloadRequest) -> Result<DownloadJob, EmuBoxError> {
    DownloadService::create_job(request)
}

#[tauri::command]
pub fn get_download_jobs() -> Result<Vec<DownloadJob>, EmuBoxError> {
    DownloadService::list_jobs()
}

#[tauri::command]
pub fn start_download(id: String) -> Result<DownloadJob, EmuBoxError> {
    DownloadService::start(id)
}

#[tauri::command]
pub fn pause_download(id: String) -> Result<DownloadJob, EmuBoxError> {
    DownloadService::pause(&id)
}

#[tauri::command]
pub fn resume_download(id: String) -> Result<DownloadJob, EmuBoxError> {
    DownloadService::resume(id)
}

#[tauri::command]
pub fn cancel_download(id: String) -> Result<DownloadJob, EmuBoxError> {
    DownloadService::cancel(&id)
}

#[tauri::command]
pub fn get_download_candidates(id: String) -> Result<Vec<String>, EmuBoxError> {
    crate::services::download_manager::candidates(&id)
}

#[tauri::command]
pub fn select_download_candidate(id: String, path: String) -> Result<DownloadJob, EmuBoxError> {
    crate::services::download_manager::select_candidate(&id, &path)
}

#[tauri::command]
pub fn get_download_sources(
    game_id: String,
) -> Result<Vec<crate::services::manifest_service::SourceOption>, EmuBoxError> {
    DownloadService::list_sources(&game_id)
}

#[tauri::command]
pub fn download_game(
    game_id: String,
    source_id: Option<String>,
) -> Result<DownloadJob, EmuBoxError> {
    DownloadService::download_game_from_source(game_id, source_id)
}

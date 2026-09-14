//! Canonical appliance filesystem locations, centralized so every service
//! composes paths from the same base constants instead of repeating literals.

pub const CONFIG_DIR: &str = "/etc/emubox";
pub const DATA_DIR: &str = "/var/lib/emubox";
pub const CACHE_DIR: &str = "/var/cache/emubox";
pub const LOG_DIR: &str = "/var/log/emubox";
pub const RUNTIME_DIR: &str = "/run/emubox";

pub fn games_dir() -> String {
    format!("{DATA_DIR}/games")
}

pub fn emulators_dir() -> String {
    #[cfg(test)]
    return std::env::temp_dir().join(format!("emubox-emulators-test-{}", std::process::id())).to_string_lossy().to_string();
    #[cfg(not(test))]
    format!("{DATA_DIR}/emulators")
}

pub fn emulator_dir(emulator_id: &str) -> String {
    format!("{}/{emulator_id}", emulators_dir())
}

pub fn emulator_config_dir(emulator_id: &str) -> String {
    format!("{}/config", emulator_dir(emulator_id))
}

pub fn bios_dir() -> String {
    format!("{DATA_DIR}/bios")
}

pub fn saves_dir() -> String {
    format!("{DATA_DIR}/saves")
}

pub fn states_dir() -> String {
    format!("{DATA_DIR}/states")
}

pub fn screenshots_dir() -> String {
    format!("{DATA_DIR}/screenshots")
}

pub fn database_path() -> String {
    format!("{DATA_DIR}/emubox.db")
}

pub fn downloads_cache_dir() -> String {
    format!("{CACHE_DIR}/downloads")
}

pub fn download_links_file() -> String {
    let configured = format!("{CONFIG_DIR}/download-links.txt");
    let repository = "/opt/emubox/data/download-links.txt";
    select_download_links_file(&configured, repository)
}

fn select_download_links_file(configured: &str, repository: &str) -> String {
    let has_links = std::fs::read_to_string(configured).ok().is_some_and(|content| {
        content.lines().any(|line| {
            let link = line.split('#').next().unwrap_or("").trim();
            link.starts_with("https://") || link.starts_with("http://")
        })
    });
    if has_links || !std::path::Path::new(repository).is_file() { configured.into() } else { repository.into() }
}

#[cfg(test)]
mod download_link_tests {
    use super::*;

    #[test]
    fn repository_links_replace_only_empty_configuration() {
        let directory = std::env::temp_dir().join(format!("emubox-links-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let configured = directory.join("configured.txt");
        let repository = directory.join("repository.txt");
        std::fs::write(&configured, "# placeholder\n").unwrap();
        std::fs::write(&repository, "https://example.test/catalog.json\n").unwrap();
        assert_eq!(select_download_links_file(configured.to_str().unwrap(), repository.to_str().unwrap()), repository.to_string_lossy());
        std::fs::write(&configured, "https://example.test/custom.json\n").unwrap();
        assert_eq!(select_download_links_file(configured.to_str().unwrap(), repository.to_str().unwrap()), configured.to_string_lossy());
        std::fs::remove_dir_all(directory).unwrap();
    }
}

pub fn config_file() -> String {
    format!("{CONFIG_DIR}/config.json")
}

pub fn settings_file() -> String {
    format!("{CONFIG_DIR}/settings.json")
}

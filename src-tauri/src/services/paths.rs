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
    format!("{CONFIG_DIR}/download-links.txt")
}

pub fn config_file() -> String {
    format!("{CONFIG_DIR}/config.json")
}

pub fn settings_file() -> String {
    format!("{CONFIG_DIR}/settings.json")
}

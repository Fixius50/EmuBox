use std::path::{Path, PathBuf};

pub(crate) const BWRAP_BINARY: &str = "/usr/bin/bwrap";
pub(crate) const PRLIMIT_BINARY: &str = "/usr/bin/prlimit";
pub(crate) const SHELL_BINARY: &str = "/usr/bin/sh";
pub(crate) const GAMESCOPE_BINARY: &str = "/usr/bin/gamescope";
pub(crate) const SYSTEM_USR: &str = "/usr";
pub(crate) const SYSTEM_BIN: &str = "/usr/bin";
pub(crate) const SYSTEM_LOCAL_BIN: &str = "/usr/local/bin";
pub(crate) const OPT_ROOT: &str = "/opt";
pub(crate) const EMUBOX_BIN: &str = "/opt/emubox/bin";
pub(crate) const HOME: &str = "/home/player";
pub(crate) const RUNTIME: &str = "/run/player";
pub(crate) const GAME: &str = "/game";
pub(crate) const BIOS: &str = "/bios";

pub(crate) const HOST_USR: &str = "/usr";
pub(crate) const HOST_BIN: &str = "/bin";
pub(crate) const HOST_LIB: &str = "/lib";
pub(crate) const HOST_LIB64: &str = "/lib64";
pub(crate) const PROC: &str = "/proc";
pub(crate) const DEV: &str = "/dev";
pub(crate) const TMP: &str = "/tmp";
pub(crate) const RUN: &str = "/run";
pub(crate) const DEV_SHM: &str = "/dev/shm";
pub(crate) const SOUND_DEVICES: &str = "/dev/snd";
pub(crate) const DRM_DEVICES: &str = "/dev/dri";
pub(crate) const INPUT_DEVICES: &str = "/dev/input";
pub(crate) const UDEV_DATA: &str = "/run/udev/data";
pub(crate) const WAYLAND_SOCKET: &str = "/run/player/wayland-0";
pub(crate) const RETROARCH_PROFILE_CONFIG: &str = "/run/retroarch-profile.cfg";

pub(crate) const CONFIG_DIRECTORY: &str = ".config";
pub(crate) const DATA_DIRECTORY: &str = ".local/share";
pub(crate) const CACHE_DIRECTORY: &str = ".cache";
pub(crate) const SAVES_DIRECTORY: &str = "saves";
pub(crate) const STATES_DIRECTORY: &str = "states";
pub(crate) const SCREENSHOTS_DIRECTORY: &str = "screenshots";
pub(crate) const WINE_DIRECTORY: &str = "wine";
pub(crate) const RPCS3_DEV_FLASH: &str = ".config/rpcs3/dev_flash";
pub(crate) const RPCS3_INSTALLED_GAMES: &str = ".config/rpcs3/dev_hdd0/game";
pub(crate) const SHADPS4_CONFIG: &str = ".config/shadps4";
pub(crate) const RETROARCH_SAVE: &str = "saves/content.srm";
pub(crate) const RETROARCH_STATE: &str = "states/content.state";

pub(crate) const XDG_CONFIG_HOME: &str = "/home/player/.config";
pub(crate) const XDG_DATA_HOME: &str = "/home/player/.local/share";
pub(crate) const XDG_CACHE_HOME: &str = "/home/player/.cache";
pub(crate) const DOLPHIN_USER_DIRECTORY: &str = "/home/player/.local/share/dolphin-emu";
pub(crate) const DOLPHIN_USER_DIRECTORY_RELATIVE: &str = ".local/share/dolphin-emu";

pub(crate) const OPTIONAL_READ_ONLY_HOST_MOUNTS: [&str; 5] = [
    "/etc/fonts",
    "/etc/ld.so.cache",
    "/etc/localtime",
    "/sys/devices",
    "/sys/class/drm",
];

pub(crate) struct ManagedConfigRoute {
    pub source: &'static str,
    pub target: &'static str,
}

pub(crate) fn managed_config_route(emulator_id: &str) -> Option<ManagedConfigRoute> {
    let (source, target) = match emulator_id {
        "retroarch" => ("retroarch.cfg", ".config/retroarch/retroarch.cfg"),
        "pcsx2" => ("PCSX2.ini", ".config/PCSX2/inis/PCSX2.ini"),
        "duckstation" => ("settings.ini", ".config/duckstation/settings.ini"),
        "dolphin" => ("Dolphin.ini", ".local/share/dolphin-emu/Config/Dolphin.ini"),
        "ppsspp" => (
            "PSP/SYSTEM/ppsspp.ini",
            ".config/ppsspp/PSP/SYSTEM/ppsspp.ini",
        ),
        _ => return None,
    };
    Some(ManagedConfigRoute { source, target })
}

pub(crate) fn home_path(relative: impl AsRef<Path>) -> PathBuf {
    Path::new(HOME).join(relative)
}

use crate::{errors::EmuBoxError, services::runtime::launch_policy::failure};
use std::{
    fs,
    os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt},
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
};

pub(super) const HOME: &str = "/home/player";
pub(super) const RUNTIME: &str = "/run/player";

pub(super) fn private_directory(path: &Path) -> Result<(), EmuBoxError> {
    if !path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir))
    {
        return Err(failure("Directorio privado invalido"));
    }
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {}
            Ok(_) => {
                return Err(failure(
                    "El estado privado contiene enlaces o archivos inesperados",
                ))
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                fs::create_dir(&current).map_err(|error| failure(error.to_string()))?;
                fs::set_permissions(&current, fs::Permissions::from_mode(0o700))
                    .map_err(|error| failure(error.to_string()))?;
            }
            Err(error) => return Err(failure(error.to_string())),
        }
    }
    Ok(())
}

pub(super) fn base_command(bwrap: &Path) -> Command {
    let mut command = Command::new("/usr/bin/prlimit");
    command
        .args(["--core=0", "--fsize=21474836480", "--nofile=2048", "--"])
        .arg(bwrap);
    command
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    command.args([
        "--unshare-all",
        "--unshare-user",
        "--disable-userns",
        "--assert-userns-disabled",
        "--die-with-parent",
        "--new-session",
        "--cap-drop",
        "ALL",
        "--hostname",
        "emubox-game",
        "--ro-bind",
        "/usr",
        "/usr",
        "--symlink",
        "usr/bin",
        "/bin",
        "--symlink",
        "usr/lib",
        "/lib",
        "--symlink",
        "usr/lib",
        "/lib64",
        "--proc",
        "/proc",
        "--dev",
        "/dev",
        "--size",
        "536870912",
        "--tmpfs",
        "/tmp",
        "--size",
        "16777216",
        "--tmpfs",
        "/run",
        "--size",
        "134217728",
        "--tmpfs",
        "/dev/shm",
        "--dir",
        RUNTIME,
        "--dir",
        "/game",
        "--dir",
        HOME,
        "--clearenv",
        "--setenv",
        "PATH",
        "/usr/bin",
        "--setenv",
        "LANG",
        "C.UTF-8",
        "--setenv",
        "HOME",
        HOME,
        "--setenv",
        "XDG_RUNTIME_DIR",
        RUNTIME,
        "--setenv",
        "XDG_CONFIG_HOME",
        "/home/player/.config",
        "--setenv",
        "XDG_DATA_HOME",
        "/home/player/.local/share",
        "--setenv",
        "XDG_CACHE_HOME",
        "/home/player/.cache",
        "--setenv",
        "APPIMAGE_EXTRACT_AND_RUN",
        "1",
        "--setenv",
        "SDL_VIDEODRIVER",
        "wayland",
        "--setenv",
        "QT_QPA_PLATFORM",
        "wayland",
        "--setenv",
        "SDL_AUDIODRIVER",
        "alsa",
        "--setenv",
        "ALSOFT_DRIVERS",
        "alsa",
    ]);
    command
}

fn socket_path(path: &Path) -> Result<PathBuf, EmuBoxError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| failure(error.to_string()))?;
    if !metadata.file_type().is_socket() {
        return Err(failure("Socket de sesion invalido"));
    }
    Ok(path.into())
}

pub(super) fn session_access(command: &mut Command) -> Result<(), EmuBoxError> {
    let runtime = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .ok_or_else(|| failure("Falta XDG_RUNTIME_DIR"))?;
    let display = std::env::var("WAYLAND_DISPLAY").map_err(|_| {
        failure("El lanzamiento seguro requiere Wayland; no se expone X11 del host")
    })?;
    if Path::new(&display).components().count() != 1
        || !matches!(
            Path::new(&display).components().next(),
            Some(Component::Normal(_))
        )
    {
        return Err(failure("Nombre Wayland invalido"));
    }
    let socket = socket_path(&runtime.join(display))?;
    command
        .arg("--ro-bind")
        .arg(socket)
        .arg("/run/player/wayland-0")
        .args(["--setenv", "WAYLAND_DISPLAY", "wayland-0"]);
    for entry in fs::read_dir("/dev/snd").into_iter().flatten().flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if ((name.starts_with("pcmC") && name.ends_with('p')) || name.starts_with("controlC"))
            && entry.file_type().is_ok_and(|kind| kind.is_char_device())
        {
            command
                .arg("--dev-bind")
                .arg(entry.path())
                .arg(entry.path());
        }
    }
    for entry in fs::read_dir("/dev/dri").into_iter().flatten().flatten() {
        if entry.file_name().to_string_lossy().starts_with("renderD")
            && entry.file_type().is_ok_and(|kind| kind.is_char_device())
        {
            command
                .arg("--dev-bind")
                .arg(entry.path())
                .arg(entry.path());
        }
    }
    for entry in fs::read_dir("/dev/input").into_iter().flatten().flatten() {
        let metadata = entry
            .metadata()
            .map_err(|error| failure(error.to_string()))?;
        let device = metadata.rdev();
        let major = ((device >> 8) & 0xfff) | ((device >> 32) & 0xfffff000);
        let minor = (device & 0xff) | ((device >> 12) & 0xffffff00);
        let properties =
            fs::read_to_string(format!("/run/udev/data/c{major}:{minor}")).unwrap_or_default();
        if metadata.file_type().is_char_device() && gamepad_only(&properties) {
            command
                .arg("--dev-bind")
                .arg(entry.path())
                .arg(entry.path());
        }
    }
    Ok(())
}

pub(super) fn gamepad_only(properties: &str) -> bool {
    properties
        .lines()
        .any(|line| line == "E:ID_INPUT_JOYSTICK=1")
        && !properties
            .lines()
            .any(|line| matches!(line, "E:ID_INPUT_KEYBOARD=1" | "E:ID_INPUT_MOUSE=1"))
}

pub(super) fn retroarch_config(state: &Path) -> Result<PathBuf, EmuBoxError> {
    use std::io::Write;
    let temporary = state.join(format!(".retroarch-profile-{}", std::process::id()));
    let destination = state.join(".retroarch-profile.cfg");
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| failure(error.to_string()))?;
    file.write_all(b"system_directory = \"/bios\"\nscreenshot_directory = \"/home/player/screenshots\"\naudio_driver = \"alsa\"\nconfig_save_on_exit = \"false\"\nnetwork_cmd_enable = \"false\"\nstdin_cmd_enable = \"false\"\n")
        .map_err(|error| failure(error.to_string()))?;
    fs::rename(&temporary, &destination).map_err(|error| failure(error.to_string()))?;
    Ok(destination)
}

pub(super) fn entrypoint(command: &mut Command) {
    command.args([
        "--",
        "/usr/bin/sh",
        "-c",
        "printf '%s\n' EMUBOX_SANDBOX_READY; exec \"$@\"",
        "emubox-launch",
    ]);
}

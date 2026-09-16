use super::launch_policy::{failure, trusted_binary, LaunchPolicy};
use crate::{
    errors::EmuBoxError,
    models::{Game, PublishedDownload, TransferControl},
    services::{download_preparation, installer_preparation, paths},
};
use sha2::{Digest, Sha256};
use std::{
    fs,
    os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt},
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
};

const HOME: &str = "/home/player";
const RUNTIME: &str = "/run/player";

struct Content {
    root: PathBuf,
    target: PathBuf,
    rom: PathBuf,
    ps3_config: Option<PathBuf>,
}

fn canonical(path: &Path) -> Result<PathBuf, EmuBoxError> {
    fs::canonicalize(path).map_err(|error| failure(format!("{}: {error}", path.display())))
}

fn content(rom: &Path, games: &Path, wine: bool) -> Result<Content, EmuBoxError> {
    let base = canonical(games)?;
    if base != games {
        return Err(failure(
            "La raiz de juegos no puede redirigir mediante enlaces",
        ));
    }
    let rom = canonical(rom)?;
    if !rom.starts_with(&base) || rom == base {
        return Err(failure(
            "El contenido no pertenece a la biblioteca gestionada",
        ));
    }
    let package_root = rom
        .ancestors()
        .skip(1)
        .take_while(|path| *path != base)
        .find(|path| path.join(".emubox-managed").is_file());
    if let Some(root) = package_root {
        let marker = root.join(".emubox-managed");
        if fs::symlink_metadata(&marker)
            .map_err(|error| failure(error.to_string()))?
            .file_type()
            .is_symlink()
            || fs::metadata(&marker)
                .map_err(|error| failure(error.to_string()))?
                .len()
                > 16 * 1024 * 1024
        {
            return Err(failure("Marcador de instalacion no seguro"));
        }
        let package: PublishedDownload =
            serde_json::from_slice(&fs::read(marker).map_err(|error| failure(error.to_string()))?)
                .map_err(|error| failure(error.to_string()))?;
        if package.files.iter().any(|path| {
            path.as_os_str().is_empty()
                || path
                    .components()
                    .any(|part| !matches!(part, Component::Normal(_)))
        }) {
            return Err(failure("Instalacion con rutas no confinadas"));
        }
        let files: Vec<_> = package.files.iter().map(|path| root.join(path)).collect();
        download_preparation::verify(&files, root, None, &TransferControl::default())?;
        let relative = rom
            .strip_prefix(root)
            .map_err(|error| failure(error.to_string()))?;
        if wine
            && (package.launch.as_deref() != Some(relative)
                || !installer_preparation::candidates("pc", root, &package)
                    .iter()
                    .any(|candidate| Path::new(candidate) == relative))
        {
            return Err(failure(
                "Wine requiere la seleccion validada de una instalacion preparada",
            ));
        }
        let ps3_config =
            if package.installation.as_ref().is_some_and(|installation| {
                installation.kind == crate::models::InstallationKind::Ps3
            }) {
                if package.launch.as_deref() != Some(relative)
                    || !installer_preparation::candidates("ps3", root, &package)
                        .iter()
                        .any(|candidate| Path::new(candidate) == relative)
                {
                    return Err(failure(
                        "La instalacion PS3 no tiene una seleccion validada",
                    ));
                }
                let installation = package
                    .installation
                    .as_ref()
                    .ok_or_else(|| failure("Falta instalacion PS3"))?;
                Some(root.join(&installation.root).join("home/config/rpcs3"))
            } else {
                None
            };
        return Ok(Content {
            root: root.into(),
            target: "/game".into(),
            rom: Path::new("/game").join(relative),
            ps3_config,
        });
    }
    if wine {
        return Err(failure("Wine requiere un paquete gestionado y preparado"));
    }
    let relative = rom
        .strip_prefix(&base)
        .map_err(|error| failure(error.to_string()))?;
    let root = if rom.is_file() && relative.components().count() <= 2 {
        rom.clone()
    } else if rom.is_dir() {
        rom.clone()
    } else {
        rom.parent()
            .ok_or_else(|| failure("Contenido sin directorio"))?
            .to_path_buf()
    };
    let target = if root.is_file() {
        Path::new("/game").join(root.file_name().ok_or_else(|| failure("ROM sin nombre"))?)
    } else {
        "/game".into()
    };
    let destination = if root == rom {
        target.clone()
    } else {
        target.join(
            rom.strip_prefix(&root)
                .map_err(|error| failure(error.to_string()))?,
        )
    };
    Ok(Content {
        root,
        target,
        rom: destination,
        ps3_config: None,
    })
}

fn private_directory(path: &Path) -> Result<(), EmuBoxError> {
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

fn base_command(bwrap: &Path) -> Command {
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

fn session_access(command: &mut Command) -> Result<(), EmuBoxError> {
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

fn gamepad_only(properties: &str) -> bool {
    properties
        .lines()
        .any(|line| line == "E:ID_INPUT_JOYSTICK=1")
        && !properties
            .lines()
            .any(|line| matches!(line, "E:ID_INPUT_KEYBOARD=1" | "E:ID_INPUT_MOUSE=1"))
}

fn retroarch_config(state: &Path) -> Result<PathBuf, EmuBoxError> {
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

fn entrypoint(command: &mut Command) {
    command.args([
        "--",
        "/usr/bin/sh",
        "-c",
        "printf '%s\\n' EMUBOX_SANDBOX_READY; exec \"$@\"",
        "emubox-launch",
    ]);
}

pub(super) fn command(
    game: &Game,
    emulator_id: &str,
    policy: &LaunchPolicy,
    gamescope: bool,
) -> Result<Command, EmuBoxError> {
    let bwrap = trusted_binary(Path::new("/usr/bin/bwrap"))?;
    trusted_binary(Path::new("/usr/bin/prlimit"))?;
    trusted_binary(Path::new("/usr/bin/sh"))?;
    let rom = game
        .rom_path
        .as_deref()
        .ok_or_else(|| failure("No hay instalacion seleccionada"))?;
    let content = content(Path::new(rom), Path::new(&paths::games_dir()), policy.wine)?;
    let identity = format!(
        "{:x}",
        Sha256::digest(format!("{}\0{}", game.id, emulator_id).as_bytes())
    );
    let state = Path::new(paths::DATA_DIR).join("sandbox").join(identity);
    private_directory(&state)?;
    for directory in [
        ".config",
        ".local/share",
        ".cache",
        "saves",
        "states",
        "screenshots",
        "wine",
    ] {
        private_directory(&state.join(directory))?;
    }
    let mut command = base_command(&bwrap);
    command
        .arg("--bind")
        .arg(&state)
        .arg(HOME)
        .arg("--ro-bind")
        .arg(&content.root)
        .arg(&content.target);
    for path in [
        "/etc/fonts",
        "/etc/ld.so.cache",
        "/etc/localtime",
        "/sys/devices",
        "/sys/class/drm",
    ] {
        if Path::new(path).exists() {
            command.args(["--ro-bind", path, path]);
        }
    }
    let bios = PathBuf::from(paths::bios_dir());
    if bios.is_dir() && canonical(&bios)? == bios {
        command.arg("--ro-bind").arg(bios).arg("/bios");
    }
    if emulator_id == "rpcs3" {
        let config = content
            .ps3_config
            .clone()
            .unwrap_or_else(|| PathBuf::from(paths::emulator_config_dir("rpcs3")).join("rpcs3"));
        let firmware = config.join("dev_flash");
        if !firmware.join("sys/external/liblv2.sprx").is_file() || canonical(&firmware)? != firmware
        {
            return Err(failure(
                "Falta firmware PS3 autorizado en el entorno gestionado",
            ));
        }
        private_directory(&state.join(".config/rpcs3/dev_flash"))?;
        command
            .arg("--ro-bind")
            .arg(firmware)
            .arg("/home/player/.config/rpcs3/dev_flash");
        if content.ps3_config.is_some() {
            let installed = config.join("dev_hdd0/game");
            if canonical(&installed)? != installed {
                return Err(failure("Instalacion PS3 redirigida"));
            }
            private_directory(&state.join(".config/rpcs3/dev_hdd0/game"))?;
            command
                .arg("--ro-bind")
                .arg(installed)
                .arg("/home/player/.config/rpcs3/dev_hdd0/game");
        }
    }
    if !policy.executable.starts_with("/usr") {
        command
            .arg("--ro-bind")
            .arg(&policy.executable)
            .arg(&policy.executable);
    }
    for pair in policy.arguments.windows(2) {
        if pair[0] == "-L" && !Path::new(&pair[1]).starts_with("/usr") {
            command.arg("--ro-bind").arg(&pair[1]).arg(&pair[1]);
        }
    }
    session_access(&mut command)?;
    let libretro = policy.arguments.iter().any(|arg| arg == "-L");
    if libretro {
        command
            .arg("--ro-bind")
            .arg(retroarch_config(&state)?)
            .arg("/run/retroarch-profile.cfg");
    }
    if policy.wine {
        command.args([
            "--setenv",
            "WINEPREFIX",
            "/home/player/wine",
            "--setenv",
            "WINEDLLOVERRIDES",
            "mscoree,mshtml=",
        ]);
    }
    command
        .arg("--chdir")
        .arg(content.rom.parent().unwrap_or(Path::new("/game")));
    entrypoint(&mut command);
    if gamescope || policy.wine {
        let compositor = trusted_binary(Path::new("/usr/bin/gamescope"))?;
        command.arg(compositor).args(["-f", "--"]);
    }
    command.arg(&policy.executable).args(&policy.arguments);
    if libretro {
        command.args([
            "--appendconfig",
            "/run/retroarch-profile.cfg",
            "--save",
            "/home/player/saves/content.srm",
            "--savestate",
            "/home/player/states/content.state",
        ]);
    }
    if emulator_id == "shadps4" {
        command.args(["--override-root", "/home/player/.config/shadps4"]);
    }
    command.arg(&content.rom);
    Ok(command)
}

pub(super) fn spawn(command: &mut Command) -> Result<std::process::Child, EmuBoxError> {
    use std::io::{BufRead, BufReader, Read};
    let mut child = command
        .spawn()
        .map_err(|error| failure(error.to_string()))?;
    let output = child
        .stdout
        .take()
        .ok_or_else(|| failure("No se puede confirmar el sandbox"))?;
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let mut reader = BufReader::new(output);
        let mut confirmed = false;
        for _ in 0..8 {
            let mut status = String::new();
            if !reader
                .by_ref()
                .take(4096)
                .read_line(&mut status)
                .is_ok_and(|count| count > 0 && count < 4096)
            {
                break;
            }
            if status.trim_end() == "EMUBOX_SANDBOX_READY" {
                confirmed = true;
                break;
            }
        }
        let _ = sender.send(confirmed);
        let _ = std::io::copy(&mut reader, &mut std::io::sink());
    });
    if receiver.recv_timeout(std::time::Duration::from_secs(10)) != Ok(true) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(failure("Bubblewrap no confirmo el aislamiento; revisa namespaces, montajes y permisos. No se ejecuta sin sandbox"));
    }
    Ok(child)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "Requires Linux user namespaces and installed Bubblewrap; launches only a fixed isolation probe"]
    fn native_sandbox_hides_host_data_and_keeps_only_private_writes() {
        let server = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let root =
            std::env::temp_dir().join(format!("emubox-isolation-probe-{}", std::process::id()));
        fs::create_dir_all(root.join("state")).unwrap();
        fs::write(root.join("secret"), "private-host-fixture").unwrap();
        fs::write(root.join("content"), "readonly-content").unwrap();
        let mut command = base_command(Path::new("/usr/bin/bwrap"));
        command.env("EMUBOX_PRIVATE_SENTINEL", "must-not-inherit");
        command
            .arg("--ro-bind")
            .arg(root.join("content"))
            .arg("/game/content")
            .arg("--bind")
            .arg(root.join("state"))
            .arg(HOME);
        entrypoint(&mut command);
        command.args(["/usr/bin/sh", "-c",
                "test -z \"$EMUBOX_PRIVATE_SENTINEL\" && test ! -e /opt/emubox && test ! -e /etc/emubox && test ! -e /run/user && test ! -e \"$1\" && test ! -e /home/emubox && ! echo changed > /game/content && ! /usr/bin/bash -c 'exec 3<>/dev/tcp/127.0.0.1/$1' network-probe \"$2\" && echo private > /home/player/result",
                "probe"]).arg(root.join("secret")).arg(server.local_addr().unwrap().port().to_string());
        let mut child = spawn(&mut command).unwrap();
        assert!(child.wait().unwrap().success());
        assert_eq!(
            fs::read_to_string(root.join("state/result")).unwrap(),
            "private\n"
        );
        assert_eq!(
            fs::read_to_string(root.join("content")).unwrap(),
            "readonly-content"
        );
        let mut rejected = base_command(Path::new("/usr/bin/bwrap"));
        rejected
            .arg("--ro-bind")
            .arg(root.join("missing-mount"))
            .arg("/game/missing");
        entrypoint(&mut rejected);
        rejected.arg("/usr/bin/true");
        assert!(spawn(&mut rejected).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn namespace_and_environment_have_no_host_fallback() {
        let command = base_command(Path::new("/usr/bin/bwrap"));
        let args: Vec<_> = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        for flag in [
            "--unshare-all",
            "--unshare-user",
            "--disable-userns",
            "--clearenv",
            "--die-with-parent",
            "--new-session",
        ] {
            assert!(args.iter().any(|arg| arg == flag));
        }
        for forbidden in [
            "--share-net",
            "--unshare-user-try",
            "--not-a-security-boundary",
            "/home",
            "/opt/emubox",
            "/etc",
            "/dev/input",
            "/run/user",
        ] {
            assert!(!args.iter().any(|arg| arg == forbidden));
        }
        assert_eq!(command.get_envs().count(), 0);
        assert!(gamepad_only("E:ID_INPUT_JOYSTICK=1\n"));
        assert!(!gamepad_only(
            "E:ID_INPUT_JOYSTICK=1\nE:ID_INPUT_KEYBOARD=1\n"
        ));
        assert!(!gamepad_only("E:ID_INPUT_JOYSTICK=1\nE:ID_INPUT_MOUSE=1\n"));
    }

    #[test]
    fn content_and_state_reject_symlink_escape() {
        let root = std::env::temp_dir().join(format!("emubox-sandbox-test-{}", std::process::id()));
        fs::create_dir_all(root.join("games/nes")).unwrap();
        fs::write(root.join("secret"), "fixture").unwrap();
        std::os::unix::fs::symlink(root.join("secret"), root.join("games/nes/escape.nes")).unwrap();
        assert!(content(
            &root.join("games/nes/escape.nes"),
            &root.join("games"),
            false
        )
        .is_err());
        fs::write(root.join("games/nes/valid.nes"), "fixture").unwrap();
        assert!(content(
            &root.join("games/nes/valid.nes"),
            &root.join("games"),
            false
        )
        .is_ok());
        assert!(content(&root.join("games/nes/valid.nes"), &root.join("games"), true).is_err());
        std::os::unix::fs::symlink(root.join("games"), root.join("state")).unwrap();
        assert!(private_directory(&root.join("state/home")).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn wine_requires_explicit_managed_selection_and_creates_no_host_prefix() {
        let root = std::env::temp_dir().join(format!("emubox-wine-policy-{}", std::process::id()));
        let package_root = root.join("pc/package");
        fs::create_dir_all(package_root.join("extracted/app")).unwrap();
        let mut pe = vec![0u8; 88];
        pe[..2].copy_from_slice(b"MZ");
        pe[60..64].copy_from_slice(&64u32.to_le_bytes());
        pe[64..68].copy_from_slice(b"PE\0\0");
        pe[68..70].copy_from_slice(&0x8664u16.to_le_bytes());
        let rom = package_root.join("extracted/app/game.exe");
        fs::write(&rom, pe).unwrap();
        let mut package = PublishedDownload {
            job_id: "fixture".into(),
            source_digest: "fixture".into(),
            files: vec!["extracted/app/game.exe".into()],
            launch: None,
            preparation_reason: None,
            installation: Some(crate::models::PreparedInstallation {
                kind: crate::models::InstallationKind::Inno,
                root: "extracted".into(),
            }),
        };
        let marker = package_root.join(".emubox-managed");
        fs::write(&marker, serde_json::to_vec(&package).unwrap()).unwrap();
        assert!(content(&rom, &root, true).is_err());
        package.launch = Some("extracted/app/game.exe".into());
        fs::write(&marker, serde_json::to_vec(&package).unwrap()).unwrap();
        assert_eq!(
            content(&rom, &root, true).unwrap().rom,
            Path::new("/game/extracted/app/game.exe")
        );
        assert!(!package_root.join(".wine-prefix").exists());
        package.files.push("../../secret".into());
        fs::write(&marker, serde_json::to_vec(&package).unwrap()).unwrap();
        assert!(content(&rom, &root, true).is_err());
        fs::remove_dir_all(root).unwrap();
    }
}

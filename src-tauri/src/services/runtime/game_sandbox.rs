use super::launch_policy::{failure, trusted_binary, LaunchPolicy};
use crate::{
    errors::EmuBoxError,
    models::Game,
    services::{emulators, paths},
};
mod bubblewrap;
mod content;
#[cfg(test)]
use bubblewrap::gamepad_only;
use bubblewrap::{
    base_command, entrypoint, private_directory, retroarch_config, session_access, HOME,
};
use content::{canonical, content};
use sha2::{Digest, Sha256};
#[cfg(test)]
use std::fs;
#[cfg(test)]
use std::os::unix::fs::symlink;
use std::{
    fs::OpenOptions,
    path::{Path, PathBuf},
    process::Command,
};

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
    mount_managed_config(&mut command, &state, emulator_id)?;
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
        #[test]
        fn managed_profile_config_is_read_only_and_confined_to_private_home() {
            let (source, target) = emulators::managed_config("pcsx2").unwrap();
            fs::create_dir_all(source.parent().unwrap()).unwrap();
            fs::write(&source, "[EmuCore/GS]\nRenderer = OpenGL\n").unwrap();
            let state = std::env::temp_dir()
                .join(format!("emubox-managed-config-test-{}", std::process::id()));
            fs::create_dir_all(&state).unwrap();
            let mut command = Command::new("/usr/bin/true");
            mount_managed_config(&mut command, &state, "pcsx2").unwrap();
            let args: Vec<_> = command
                .get_args()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect();
            let canonical_source = fs::canonicalize(&source)
                .unwrap()
                .to_string_lossy()
                .into_owned();
            let destination = Path::new(HOME).join(target).to_string_lossy().into_owned();
            assert!(args.windows(3).any(
                |entry| entry == ["--ro-bind", canonical_source.as_str(), destination.as_str()]
            ));
            assert!(state.join(".config/PCSX2/PCSX2.ini").is_file());

            let mut unknown = Command::new("/usr/bin/true");
            mount_managed_config(&mut unknown, &state, "wine").unwrap();
            assert_eq!(unknown.get_args().count(), 0);

            fs::remove_file(&source).unwrap();
            symlink("/etc/passwd", &source).unwrap();
            assert!(mount_managed_config(&mut command, &state, "pcsx2").is_err());
            let _ = fs::remove_file(&source);
            let _ = fs::remove_dir_all(&state);
        }
        command.args(["--override-root", "/home/player/.config/shadps4"]);
    }
    command.arg(&content.rom);
    Ok(command)
}

fn mount_managed_config(
    command: &mut Command,
    state: &Path,
    emulator_id: &str,
) -> Result<(), EmuBoxError> {
    let Some((source, target)) = emulators::managed_config(emulator_id) else {
        return Ok(());
    };
    if !target.starts_with(".config")
        || target
            .components()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        return Err(failure("Destino de configuracion gestionada invalido"));
    }
    if !source.exists() {
        return Ok(());
    }
    let source_metadata =
        fs::symlink_metadata(&source).map_err(|error| failure(error.to_string()))?;
    if !source_metadata.is_file() || source_metadata.file_type().is_symlink() {
        return Err(failure("Configuracion gestionada no es un archivo regular"));
    }
    let config_root = PathBuf::from(paths::emulator_config_dir(emulator_id));
    let root_metadata =
        fs::symlink_metadata(&config_root).map_err(|error| failure(error.to_string()))?;
    if !root_metadata.is_dir() || root_metadata.file_type().is_symlink() {
        return Err(failure("Directorio de configuracion gestionada invalido"));
    }
    let canonical_root = canonical(&config_root)?;
    let canonical_source = canonical(&source)?;
    if !canonical_source.starts_with(&canonical_root) || canonical_source == canonical_root {
        return Err(failure("Configuracion gestionada fuera de su directorio"));
    }
    let destination = Path::new(HOME).join(&target);
    let private_destination = state.join(&target);
    let parent = private_destination
        .parent()
        .ok_or_else(|| failure("Destino de configuracion sin directorio"))?;
    private_directory(parent)?;
    match fs::symlink_metadata(&private_destination) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => (),
        Ok(_) => return Err(failure("Destino privado de configuracion invalido")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&private_destination)
                .map_err(|error| failure(error.to_string()))?;
        }
        Err(error) => return Err(failure(error.to_string())),
    }
    command
        .arg("--ro-bind")
        .arg(canonical_source)
        .arg(destination);
    Ok(())
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
    use crate::models::PublishedDownload;

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

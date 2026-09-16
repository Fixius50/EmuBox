use crate::{
    errors::EmuBoxError,
    models::LaunchGameRequest,
    services::{binary_service, emulators},
};
use std::{
    fs,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

pub(super) struct LaunchPolicy {
    pub executable: PathBuf,
    pub arguments: Vec<String>,
    pub wine: bool,
}

pub(super) fn failure(message: impl Into<String>) -> EmuBoxError {
    EmuBoxError::GameLaunchFailed(message.into())
}

pub(super) fn trusted_binary(path: &Path) -> Result<PathBuf, EmuBoxError> {
    let resolved = fs::canonicalize(path).map_err(|error| failure(error.to_string()))?;
    if !resolved.starts_with("/usr") && !resolved.starts_with("/opt/emubox/bin") {
        return Err(failure(
            "El ejecutable o core no pertenece a una raiz de herramientas autorizada",
        ));
    }
    for ancestor in resolved.ancestors() {
        let metadata = fs::metadata(ancestor).map_err(|error| failure(error.to_string()))?;
        if metadata.uid() != 0 || metadata.mode() & 0o022 != 0 {
            return Err(failure(format!(
                "Herramienta no protegida contra escritura del usuario: {}",
                ancestor.display()
            )));
        }
    }
    binary_service::validate_binary(&resolved, crate::models::Architecture::current(), false)
        .map_err(failure)?;
    Ok(resolved)
}

pub(super) fn validate_request(
    request: &LaunchGameRequest,
    association_args: &[String],
    association_config: Option<&str>,
) -> Result<(), EmuBoxError> {
    if request.rom_path.is_some()
        || request
            .custom_args
            .as_ref()
            .is_some_and(|args| !args.is_empty())
        || !association_args.is_empty()
        || association_config.is_some()
    {
        return Err(failure("El lanzamiento aislado no admite rutas ROM, argumentos ni configuraciones libres; usa la instalacion y el perfil nativo"));
    }
    if request.save_state_slot.is_some() || request.fullscreen == Some(false) {
        return Err(failure(
            "Opcion de lanzamiento aun no admitida por el perfil aislado",
        ));
    }
    Ok(())
}

pub(super) fn resolve(emulator_id: &str, platform: &str) -> Result<LaunchPolicy, EmuBoxError> {
    let profile = emulators::registry()
        .into_iter()
        .find(|profile| profile.id() == emulator_id)
        .ok_or_else(|| failure("Emulador sin perfil de ejecucion autorizado"))?;
    if !profile.supported_platforms().contains(&platform) {
        return Err(failure("El perfil no admite la plataforma instalada"));
    }
    let executable = profile
        .binary_candidates()
        .iter()
        .map(|candidate| {
            let path = Path::new(candidate);
            if path.is_absolute() {
                path.to_path_buf()
            } else {
                Path::new("/usr/bin").join(path)
            }
        })
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| failure("No esta instalado el ejecutable del perfil nativo"))?;
    let executable = trusted_binary(&executable)?;
    let mut arguments: Vec<String> = profile
        .default_arguments()
        .iter()
        .map(|value| (*value).into())
        .collect();
    if emulator_id == "retroarch" {
        let core = match platform {
            "nes" => "nestopia_libretro.so",
            "snes" => "snes9x_libretro.so",
            "genesis" => "genesis_plus_gx_libretro.so",
            "gba" => "mgba_libretro.so",
            "gb" => "gambatte_libretro.so",
            "arcade" => "fbneo_libretro.so",
            "n64" => "mupen64plus_next_libretro.so",
            "ps1" => "pcsx_rearmed_libretro.so",
            _ => return Err(failure("Plataforma sin core autorizado")),
        };
        arguments.extend(["-L".into(), core.into()]);
    }
    for index in 0..arguments.len() {
        if arguments[index] == "-L" {
            let core = arguments
                .get(index + 1)
                .and_then(|name| binary_service::resolve_core(name))
                .ok_or_else(|| failure("Core del perfil no instalado"))?;
            arguments[index + 1] = trusted_binary(&core)?.to_string_lossy().into_owned();
        }
    }
    Ok(LaunchPolicy {
        executable,
        arguments,
        wine: emulator_id == "wine",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> LaunchGameRequest {
        LaunchGameRequest {
            game_id: "fixture".into(),
            emulator_id: "wine".into(),
            rom_path: None,
            save_state_slot: None,
            custom_args: None,
            fullscreen: None,
            use_gamescope: None,
        }
    }

    #[test]
    fn freeform_arguments_paths_and_unknown_profiles_are_rejected() {
        assert!(validate_request(&request(), &[], None).is_ok());
        let mut input = request();
        input.custom_args = Some(vec!["--config=/home/user/.ssh/config".into()]);
        assert!(validate_request(&input, &[], None).is_err());
        input = request();
        input.rom_path = Some("/etc/passwd".into());
        assert!(validate_request(&input, &[], None).is_err());
        assert!(validate_request(&request(), &["--script".into()], None).is_err());
        assert!(validate_request(&request(), &[], Some("/tmp/config")).is_err());
        assert!(resolve("custom-shell", "pc").is_err());
        assert!(resolve("wine", "snes").is_err());
        assert!(trusted_binary(Path::new("/etc/passwd")).is_err());
    }
}

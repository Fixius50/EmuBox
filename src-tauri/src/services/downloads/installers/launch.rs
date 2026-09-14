use super::{detection::candidates, failure, InstallerKind};
use crate::{errors::EmuBoxError, models::TransferControl, services::download_providers::io_error};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

pub fn configure_launch(
    command: &mut Command,
    emulator: &str,
    rom: &Path,
) -> Result<(), EmuBoxError> {
    if matches!(emulator, "rpcs3" | "azahar" | "shadps4") {
        let config = PathBuf::from(crate::services::paths::emulator_config_dir(emulator));
        let data = PathBuf::from(crate::services::paths::emulator_dir(emulator)).join("data");
        fs::create_dir_all(&config).map_err(io_error)?;
        fs::create_dir_all(&data).map_err(io_error)?;
        command
            .env("APPIMAGE_EXTRACT_AND_RUN", "1")
            .env("XDG_CONFIG_HOME", &config)
            .env("XDG_DATA_HOME", &data);
        if emulator == "rpcs3" {
            command.env(
                "XDG_CACHE_HOME",
                PathBuf::from(crate::services::paths::emulator_dir(emulator)).join("cache"),
            );
        }
        if emulator == "shadps4" {
            command.args(["--override-root"]).arg(config);
        }
    }
    if !matches!(emulator, "wine" | "rpcs3") {
        return Ok(());
    }
    let rom = fs::canonicalize(rom).map_err(io_error)?;
    let Some(destination) = rom
        .ancestors()
        .skip(1)
        .find(|parent| parent.join(".emubox-managed").is_file())
    else {
        if emulator == "wine" {
            return Err(failure(
                "Wine requiere un ejecutable seleccionado de un paquete preparado",
            ));
        }
        return Ok(());
    };
    let package: crate::models::PublishedDownload =
        serde_json::from_slice(&fs::read(destination.join(".emubox-managed")).map_err(io_error)?)
            .map_err(io_error)?;
    if package.files.iter().any(|path| {
        path.as_os_str().is_empty()
            || path
                .components()
                .any(|part| !matches!(part, std::path::Component::Normal(_)))
    }) {
        return Err(failure("Marcador de instalacion contiene rutas invalidas"));
    }
    let files: Vec<_> = package
        .files
        .iter()
        .map(|path| destination.join(path))
        .collect();
    crate::services::download_preparation::verify(
        &files,
        destination,
        None,
        &TransferControl::default(),
    )?;
    let Some(installation) = &package.installation else {
        if emulator == "wine" {
            return Err(failure(
                "No hay instalacion Windows preparada para este paquete",
            ));
        }
        return Ok(());
    };
    let relative = rom.strip_prefix(destination).map_err(io_error)?;
    let platform = if emulator == "wine" { "pc" } else { "ps3" };
    if package.launch.as_deref() != Some(relative)
        || !candidates(platform, destination, &package)
            .iter()
            .any(|path| Path::new(path) == relative)
    {
        return Err(failure(
            "El ejecutable no coincide con la seleccion validada del paquete",
        ));
    }
    let installation_root = destination.join(&installation.root);
    if !fs::canonicalize(&installation_root)
        .map_err(io_error)?
        .starts_with(destination)
    {
        return Err(failure("Instalacion fuera del paquete"));
    }
    match installation.kind {
        InstallerKind::Inno => {
            let prefix = destination.join(".wine-prefix");
            if fs::symlink_metadata(&prefix).is_ok_and(|metadata| metadata.file_type().is_symlink())
            {
                return Err(failure("Prefijo Wine no seguro"));
            }
            fs::create_dir_all(&prefix).map_err(io_error)?;
            command
                .env("WINEPREFIX", prefix)
                .env("WINEDLLOVERRIDES", "mscoree,mshtml=")
                .current_dir(
                    rom.parent()
                        .ok_or_else(|| failure("Ejecutable sin directorio"))?,
                );
        }
        InstallerKind::Ps3 => {
            let config = installation_root.join("home/config");
            let firmware = config.join("rpcs3/dev_flash/sys/external/liblv2.sprx");
            if !firmware.is_file() {
                return Err(failure(&format!("Falta firmware PS3 en el entorno de este paquete: {}. Se requiere firmware autorizado antes de jugar", firmware.display())));
            }
            command
                .env("XDG_CONFIG_HOME", config)
                .env("XDG_CACHE_HOME", installation_root.join("home/cache"))
                .env("XDG_DATA_HOME", installation_root.join("home/data"));
        }
    }
    Ok(())
}

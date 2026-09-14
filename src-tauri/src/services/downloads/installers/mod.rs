mod detection;
mod firmware;
mod launch;
mod sandbox;
#[cfg(test)]
mod tests;
mod validation;

pub use crate::models::InstallationKind as InstallerKind;
use crate::{
    errors::EmuBoxError,
    models::TransferControl,
    services::{binary_service::resolve_executable, download_providers::io_error},
};
pub use detection::{candidates, kind, prepared_metadata};
use firmware::firmware_directory;
pub use launch::configure_launch;
use sandbox::{sandbox, Worker};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use validation::{inventory, pkg_succeeded};

fn failure(message: &str) -> EmuBoxError {
    EmuBoxError::ProcessFailed(message.into())
}

pub fn prepare(
    source: &Path,
    root: &Path,
    kind: InstallerKind,
    control: &TransferControl,
) -> Result<Vec<PathBuf>, EmuBoxError> {
    if control.interrupted() {
        return Err(failure("Preparacion interrumpida"));
    }
    let firmware = if kind == InstallerKind::Ps3 {
        Some(firmware_directory(
            std::env::var_os("EMUBOX_PS3_FIRMWARE_DIR").map(PathBuf::from),
            Path::new(&crate::services::paths::emulator_config_dir("rpcs3")),
        )?)
    } else {
        None
    };
    let source = fs::canonicalize(source).map_err(io_error)?;
    let output = root.join("extracted");
    if output.exists() {
        fs::remove_dir_all(&output).map_err(io_error)?;
    }
    fs::create_dir(&output).map_err(io_error)?;
    let output = fs::canonicalize(output).map_err(io_error)?;
    if let Some(firmware) = firmware {
        let target = output.join("home/config/rpcs3/dev_flash");
        for file in inventory(&firmware)? {
            if control.interrupted() {
                return Err(failure("Preparacion interrumpida"));
            }
            let destination = target.join(file.strip_prefix(&firmware).map_err(io_error)?);
            fs::create_dir_all(
                destination
                    .parent()
                    .ok_or_else(|| failure("Ruta de firmware invalida"))?,
            )
            .map_err(io_error)?;
            fs::copy(file, destination).map_err(io_error)?;
        }
    }
    let tool = match kind {
        InstallerKind::Inno => resolve_executable("innoextract"),
        InstallerKind::Ps3 => resolve_executable("rpcs3")
            .or_else(|| resolve_executable("/opt/emubox/bin/RPCS3.AppImage")),
    }
    .ok_or_else(|| failure("No esta instalado el preparador requerido (innoextract o RPCS3)"))?;
    let input = match kind {
        InstallerKind::Inno => "/input/package.exe",
        InstallerKind::Ps3 => "/input/package.pkg",
    };
    let mut command = sandbox(&source, &output, &tool, input)?;
    match kind {
        InstallerKind::Inno => {
            command.args([
                "--extract",
                "--silent",
                "--no-extract-unknown",
                "--exclude-temp",
                "--collisions",
                "error",
                "--output-dir",
                "/output",
                input,
            ]);
        }
        InstallerKind::Ps3 => {
            command.args(["--headless", "--installpkg", input]);
        }
    }
    let mut worker = Worker(command.spawn().map_err(io_error)?);
    let started = Instant::now();
    loop {
        if control.interrupted() {
            return Err(failure("Preparacion interrumpida; original conservado"));
        }
        if started.elapsed() > Duration::from_secs(1800) {
            return Err(failure("Preparador excede 30 minutos; original conservado"));
        }
        inventory(&output)?;
        if let Some(status) = worker.0.try_wait().map_err(io_error)? {
            if !status.success() {
                return Err(failure("El preparador rechazo el paquete o su version. EXE: solo Inno Setup compatible, sin ejecutar instaladores; PKG: debe ser un paquete PS3 valido"));
            }
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    let files = inventory(&output)?;
    if files.is_empty() {
        return Err(failure("El preparador no produjo contenido"));
    }
    if kind == InstallerKind::Ps3
        && (!pkg_succeeded(&output, &files)
            || !files.iter().any(|path| {
                path.starts_with(output.join("home/config/rpcs3/dev_hdd0/game"))
                    && path.file_name().is_some_and(|name| name == "PARAM.SFO")
            }))
    {
        return Err(failure(
            "RPCS3 no confirmo una instalacion PS3 completa; no se marca como instalada",
        ));
    }
    for file in &files {
        fs::File::open(file)
            .and_then(|file| file.sync_all())
            .map_err(io_error)?;
    }
    Ok(files)
}

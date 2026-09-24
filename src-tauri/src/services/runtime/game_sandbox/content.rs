use crate::{
    errors::EmuBoxError,
    models::{PublishedDownload, TransferControl},
    services::{
        download_preparation, installer_preparation,
        runtime::{launch_policy::failure, sandbox_paths},
    },
};
use std::{
    fs,
    path::{Component, Path, PathBuf},
};

pub(super) struct Content {
    pub root: PathBuf,
    pub target: PathBuf,
    pub rom: PathBuf,
    pub ps3_config: Option<PathBuf>,
}

pub(super) fn canonical(path: &Path) -> Result<PathBuf, EmuBoxError> {
    fs::canonicalize(path).map_err(|error| failure(format!("{}: {error}", path.display())))
}

pub(super) fn content(rom: &Path, games: &Path, wine: bool) -> Result<Content, EmuBoxError> {
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
            target: sandbox_paths::GAME.into(),
            rom: Path::new(sandbox_paths::GAME).join(relative),
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
        Path::new(sandbox_paths::GAME)
            .join(root.file_name().ok_or_else(|| failure("ROM sin nombre"))?)
    } else {
        sandbox_paths::GAME.into()
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

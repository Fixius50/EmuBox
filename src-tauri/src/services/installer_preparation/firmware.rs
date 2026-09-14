use super::failure;
use crate::errors::EmuBoxError;
use std::{fs, path::{Path, PathBuf}};

pub(super) fn firmware_directory(explicit: Option<PathBuf>, configured: &Path) -> Result<PathBuf, EmuBoxError> {
    let path = explicit.unwrap_or_else(|| configured.join("rpcs3/dev_flash"));
    let path = fs::canonicalize(&path).map_err(|_| failure(&format!("No se encuentra firmware PS3 instalado en {}", path.display())))?;
    if !path.join("sys/external/liblv2.sprx").is_file() {
        return Err(failure("El firmware PS3 local esta incompleto: falta sys/external/liblv2.sprx"));
    }
    Ok(path)
}


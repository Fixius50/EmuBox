use super::failure;
use crate::{errors::EmuBoxError, services::download_providers::io_error};
use std::{fs, path::{Path, PathBuf}};

const MAX_BYTES: u64 = 100 * 1024 * 1024 * 1024;

pub(super) fn inventory(root: &Path) -> Result<Vec<PathBuf>, EmuBoxError> {
    let mut files = Vec::new();
    let mut bytes = 0u64;
    for (index, entry) in walkdir::WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .enumerate()
    {
        let entry = entry.map_err(io_error)?;
        if index > 100_000 {
            return Err(failure("El instalador supera 100000 entradas"));
        }
        if entry.file_type().is_dir() {
            continue;
        }
        if !entry.file_type().is_file() {
            return Err(failure(
                "El instalador produjo enlaces o archivos especiales",
            ));
        }
        let metadata = entry.metadata().map_err(io_error)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if metadata.nlink() != 1 {
                return Err(failure("El instalador produjo enlaces duros"));
            }
        }
        bytes = bytes
            .checked_add(metadata.len())
            .ok_or_else(|| failure("Tamano de instalacion desbordado"))?;
        if bytes > MAX_BYTES {
            return Err(failure("La instalacion supera 100 GiB"));
        }
        files.push(entry.into_path());
    }
    Ok(files)
}

pub(super) fn pkg_succeeded(root: &Path, files: &[PathBuf]) -> bool {
    use std::io::{BufRead, BufReader, Read};
    files
        .iter()
        .filter(|path| {
            path.file_name().is_some_and(|name| name == "RPCS3.log")
                && (path.starts_with(root.join("home/cache"))
                    || path.starts_with(root.join("home/config")))
        })
        .any(|path| {
            let Ok(file) = fs::File::open(path) else {
                return false;
            };
            BufReader::new(file.take(16 * 1024 * 1024))
                .lines()
                .any(|line| {
                    line.is_ok_and(|line| {
                        line.contains("GUI: Successfully installed /input/package.pkg (title_id=")
                    })
                })
        })
}


use crate::services::download_providers::io_error;
use crate::{errors::EmuBoxError, models::TransferControl};
use compress_tools::{ArchiveContents, ArchiveIteratorBuilder};
use std::{
    collections::HashSet,
    fs,
    io::Write,
    path::{Component, Path, PathBuf},
};

pub fn supported_signature(header: &[u8]) -> bool {
    header.starts_with(b"7z\xbc\xaf\x27\x1c")
        || header.starts_with(b"Rar!\x1a\x07\x00")
        || header.starts_with(b"Rar!\x1a\x07\x01\x00")
}

fn relative_path(name: &str) -> Result<PathBuf, EmuBoxError> {
    let path = Path::new(name);
    if name.is_empty()
        || name.contains(['\\', ':'])
        || name.chars().any(char::is_control)
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(EmuBoxError::InvalidConfiguration(
            "Archivo comprimido contiene una ruta no segura".into(),
        ));
    }
    Ok(path.into())
}

pub fn extract(
    source: &Path,
    root: &Path,
    control: &TransferControl,
) -> Result<Vec<PathBuf>, EmuBoxError> {
    if control.interrupted() {
        return Err(EmuBoxError::ProcessFailed(
            "Preparacion interrumpida".into(),
        ));
    }
    let extraction = root.join("extracted");
    if extraction.exists() {
        fs::remove_dir_all(&extraction).map_err(io_error)?;
    }
    fs::create_dir(&extraction).map_err(io_error)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&extraction, fs::Permissions::from_mode(0o700)).map_err(io_error)?;
    }
    let compressed_size = fs::metadata(source).map_err(io_error)?.len().max(1);
    let mut archive = ArchiveIteratorBuilder::new(fs::File::open(source).map_err(io_error)?)
        .raw_format(false)
        .mtree_format(false)
        .build()
        .map_err(io_error)?;
    let mut result = Vec::new();
    let mut output: Option<fs::File> = None;
    let mut expected = 0u64;
    let mut written = 0u64;
    let mut declared_total = 0u64;
    let mut count = 0usize;
    let mut entry_open = false;
    let mut names = HashSet::new();
    for content in &mut archive {
        if control.interrupted() {
            return Err(EmuBoxError::ProcessFailed(
                "Preparacion interrumpida".into(),
            ));
        }
        match content {
            ArchiveContents::StartOfEntry(name, metadata) => {
                if entry_open { return Err(EmuBoxError::InvalidConfiguration("Entrada comprimida sin cierre".into())); }
                entry_open = true;
                count += 1;
                if count > 100_000 || metadata.st_size < 0 { return Err(EmuBoxError::InvalidConfiguration("Archivo comprimido supera limites de entradas o tamano".into())); }
                let relative = relative_path(&name)?;
                if !names.insert(relative.clone()) { return Err(EmuBoxError::InvalidConfiguration("Entrada comprimida duplicada".into())); }
                let destination = extraction.join(relative);
                let kind = metadata.st_mode & 0o170000;
                if !matches!(kind, 0o100000 | 0o040000) || metadata.st_nlink > 1 {
                    return Err(EmuBoxError::InvalidConfiguration("No se extraen enlaces ni archivos especiales".into()));
                }
                expected = metadata.st_size as u64;
                written = 0;
                declared_total = declared_total.checked_add(expected).ok_or_else(|| EmuBoxError::InvalidConfiguration("Tamano comprimido desbordado".into()))?;
                if declared_total > 100 * 1024 * 1024 * 1024 || (declared_total > 1024 * 1024 && declared_total / compressed_size > 1000) {
                    return Err(EmuBoxError::InvalidConfiguration("Archivo comprimido excede limite de expansion".into()));
                }
                if kind == 0o040000 {
                    if expected != 0 { return Err(EmuBoxError::InvalidConfiguration("Directorio comprimido con datos inesperados".into())); }
                    fs::create_dir_all(destination).map_err(io_error)?;
                    output = None;
                } else {
                    fs::create_dir_all(destination.parent().unwrap()).map_err(io_error)?;
                    output = Some(fs::OpenOptions::new().write(true).create_new(true).open(&destination).map_err(io_error)?);
                    result.push(destination);
                }
            }
            ArchiveContents::DataChunk(bytes) => {
                written = written.checked_add(bytes.len() as u64).ok_or_else(|| EmuBoxError::InvalidConfiguration("Tamano de entrada desbordado".into()))?;
                if !entry_open || written > expected { return Err(EmuBoxError::InvalidConfiguration("Entrada excede tamano declarado".into())); }
                output.as_mut().ok_or_else(|| EmuBoxError::InvalidConfiguration("Datos sin archivo regular".into()))?.write_all(&bytes).map_err(io_error)?;
            }
            ArchiveContents::EndOfEntry => {
                if !entry_open || written != expected { return Err(EmuBoxError::InvalidConfiguration("Entrada comprimida incompleta".into())); }
                if let Some(file) = output.take() { file.sync_all().map_err(io_error)?; }
                entry_open = false;
            }
            ArchiveContents::Err(error) => return Err(EmuBoxError::InvalidConfiguration(format!("No se puede preparar el archivo (cifrado, volumen ausente o formato no compatible): {error}"))),
        }
    }
    archive.close().map_err(io_error)?;
    if entry_open || result.is_empty() {
        return Err(EmuBoxError::InvalidConfiguration(
            "Archivo incompleto o sin contenido".into(),
        ));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_unsafe_paths_and_recognizes_rar_versions() {
        for name in [
            "../escape",
            "/absolute",
            "C:\\escape",
            "folder/../escape",
            "folder\nfile",
        ] {
            assert!(relative_path(name).is_err());
        }
        assert_eq!(
            relative_path("folder/game.chd").unwrap(),
            PathBuf::from("folder/game.chd")
        );
        assert!(supported_signature(b"Rar!\x1a\x07\x00"));
        assert!(supported_signature(b"Rar!\x1a\x07\x01\x00"));
        assert!(!supported_signature(b"MZ executable"));
    }

    #[test]
    fn extracts_real_seven_zip_created_locally() {
        let root = std::env::temp_dir().join(format!("emubox-sevenzip-{}", std::process::id()));
        fs::create_dir_all(root.join("input/folder")).unwrap();
        fs::write(root.join("input/folder/game.chd"), b"local archive data").unwrap();
        let source = root.join("content.7z");
        let status = std::process::Command::new("bsdtar")
            .args(["--format=7zip", "-cf"])
            .arg(&source)
            .arg("-C")
            .arg(root.join("input"))
            .arg("folder")
            .status()
            .unwrap();
        assert!(status.success());
        let files = crate::services::download_preparation::prepare(
            &[source.clone()],
            &root,
            &TransferControl::default(),
        )
        .unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(fs::read(&files[0]).unwrap(), b"local archive data");
        let control = TransferControl::default();
        control
            .cancelled
            .store(true, std::sync::atomic::Ordering::Relaxed);
        assert!(extract(&source, &root, &control).is_err());
        assert!(files[0].exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn seven_zip_rejects_symlinks() {
        let root =
            std::env::temp_dir().join(format!("emubox-sevenzip-link-{}", std::process::id()));
        fs::create_dir_all(root.join("input")).unwrap();
        std::os::unix::fs::symlink("/etc/passwd", root.join("input/link.chd")).unwrap();
        let source = root.join("content.7z");
        assert!(std::process::Command::new("bsdtar")
            .args(["--format=7zip", "-cf"])
            .arg(&source)
            .arg("-C")
            .arg(root.join("input"))
            .arg("link.chd")
            .status()
            .unwrap()
            .success());
        assert!(extract(&source, &root, &TransferControl::default()).is_err());
        assert!(!root.join("extracted/link.chd").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[ignore = "Requires EMUBOX_RAR5_COMPRESSED_FIXTURE pointing to libarchive test_read_format_rar5_compressed.rar; no network in test"]
    fn extracts_compressed_rar5_fixture() {
        let source = PathBuf::from(
            std::env::var_os("EMUBOX_RAR5_COMPRESSED_FIXTURE")
                .expect("Compressed RAR5 fixture required"),
        );
        let root =
            std::env::temp_dir().join(format!("emubox-rar5-compressed-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let files = crate::services::download_preparation::prepare(
            &[source],
            &root,
            &TransferControl::default(),
        )
        .unwrap();
        assert_eq!(files, vec![root.join("extracted/test.bin")]);
        let bytes = fs::read(&files[0]).unwrap();
        assert_eq!(bytes.len(), 1200);
        for (index, chunk) in bytes.chunks_exact(4).enumerate() {
            let ordinal = index as i32 + 1;
            let expected = (ordinal * ordinal - 3 * ordinal + 1).max(0) as u32;
            assert_eq!(u32::from_le_bytes(chunk.try_into().unwrap()), expected);
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn truncated_rar_is_not_reported_as_prepared() {
        let root =
            std::env::temp_dir().join(format!("emubox-rar-truncated-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let source = root.join("content.rar");
        for signature in [b"Rar!\x1a\x07\x00".as_slice(), b"Rar!\x1a\x07\x01\x00"] {
            fs::write(&source, signature).unwrap();
            assert!(crate::services::download_preparation::prepare(
                &[source.clone()],
                &root,
                &TransferControl::default()
            )
            .is_err());
            assert_eq!(fs::read(&source).unwrap(), signature);
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[ignore = "Requires EMUBOX_RAR5_FIXTURE pointing to libarchive test_read_format_rar5_stored.rar; no network in test"]
    fn extracts_valid_rar5_fixture() {
        let source = PathBuf::from(
            std::env::var_os("EMUBOX_RAR5_FIXTURE").expect("RAR5 fixture path required"),
        );
        let root = std::env::temp_dir().join(format!("emubox-rar5-valid-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let files = crate::services::download_preparation::prepare(
            &[source],
            &root,
            &TransferControl::default(),
        )
        .unwrap();
        assert_eq!(files, vec![root.join("extracted/helloworld.txt")]);
        assert_eq!(
            fs::read(&files[0]).unwrap(),
            b"hello libarchive test suite!\n"
        );
        fs::remove_dir_all(root).unwrap();
    }
}

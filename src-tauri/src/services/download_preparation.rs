use crate::{errors::EmuBoxError, models::TransferControl};
use super::download_providers::io_error;
use sha2::{Digest, Sha256};
use std::{fs, io::{Read, Write}, path::{Path, PathBuf}};

pub fn prepare(files: &[PathBuf], root: &Path, control: &TransferControl) -> Result<Vec<PathBuf>, EmuBoxError> {
    if files.len() != 1 { return Ok(files.to_vec()); }
    let mut header = [0u8;4];
    let mut file = fs::File::open(&files[0]).map_err(io_error)?;
    let count = file.read(&mut header).map_err(io_error)?;
    if count != 4 || header != *b"PK\x03\x04" { return Ok(files.to_vec()); }
    let extraction = root.join("extracted");
    if extraction.exists() { fs::remove_dir_all(&extraction).map_err(io_error)?; }
    fs::create_dir(&extraction).map_err(io_error)?;
    let mut archive = zip::ZipArchive::new(fs::File::open(&files[0]).map_err(io_error)?).map_err(io_error)?;
    if archive.len() > 100_000 { return Err(EmuBoxError::InvalidConfiguration("ZIP supera el limite de archivos".into())); }
    let mut total = 0u64; let mut result = Vec::new();
    for index in 0..archive.len() {
        if control.interrupted() { return Err(EmuBoxError::ProcessFailed("Preparacion interrumpida".into())); }
        let mut entry = archive.by_index(index).map_err(io_error)?;
        let relative = entry.enclosed_name().ok_or_else(|| EmuBoxError::InvalidConfiguration("ZIP contiene una ruta fuera del paquete".into()))?;
        if entry.unix_mode().is_some_and(|mode| mode & 0o170000 == 0o120000) { return Err(EmuBoxError::InvalidConfiguration("ZIP contiene enlaces simbolicos".into())); }
        if relative.components().any(|component| !matches!(component,std::path::Component::Normal(_))) { return Err(EmuBoxError::InvalidConfiguration("ZIP contiene una ruta no segura".into())); }
        total = total.checked_add(entry.size()).ok_or_else(|| EmuBoxError::InvalidConfiguration("Tamano ZIP desbordado".into()))?;
        if entry.size() > 1024 * 1024 && entry.size() / entry.compressed_size().max(1) > 1000 { return Err(EmuBoxError::InvalidConfiguration("ZIP excede la relacion de expansion permitida".into())); }
        if total > 100 * 1024 * 1024 * 1024 { return Err(EmuBoxError::InvalidConfiguration("ZIP excede 100 GiB descomprimidos".into())); }
        let destination = extraction.join(relative);
        if entry.is_dir() { fs::create_dir_all(destination).map_err(io_error)?; continue; }
        fs::create_dir_all(destination.parent().unwrap()).map_err(io_error)?;
        let mut output = fs::OpenOptions::new().write(true).create_new(true).open(&destination).map_err(io_error)?;
        let mut buffer = [0u8;64 * 1024]; let mut written = 0;
        loop {
            if control.interrupted() { return Err(EmuBoxError::ProcessFailed("Preparacion interrumpida".into())); }
            let read = entry.read(&mut buffer).map_err(io_error)?; if read == 0 { break; }
            written += read as u64;
            if written > entry.size() { return Err(EmuBoxError::InvalidConfiguration("ZIP excede tamano declarado".into())); }
            output.write_all(&buffer[..read]).map_err(io_error)?;
        }
        output.sync_all().map_err(io_error)?; result.push(destination);
    }
    if result.is_empty() { return Err(EmuBoxError::InvalidConfiguration("ZIP sin contenido".into())); }
    Ok(result)
}

pub fn verify(files: &[PathBuf], root: &Path, checksum: Option<&str>, control: &TransferControl) -> Result<(), EmuBoxError> {
    let root = fs::canonicalize(root).map_err(io_error)?;
    for path in files {
        if !fs::canonicalize(path).map_err(io_error)?.starts_with(&root) || fs::symlink_metadata(path).map_err(io_error)?.file_type().is_symlink() {
            return Err(EmuBoxError::InvalidConfiguration("Artefacto fuera de la zona de descarga".into()));
        }
    }
    if let Some(expected) = checksum {
        let expected = expected.trim().strip_prefix("sha256:").unwrap_or(expected.trim());
        if expected.len() != 64 || !expected.chars().all(|character| character.is_ascii_hexdigit()) {
            return Err(EmuBoxError::InvalidConfiguration("Checksum no compatible: se requiere SHA-256 hexadecimal".into()));
        }
        if files.len() != 1 { return Err(EmuBoxError::InvalidConfiguration("Checksum de paquete ambiguo para multiples archivos".into())); }
        let mut file = fs::File::open(&files[0]).map_err(io_error)?;
        let mut hasher = Sha256::new(); let mut buffer = [0u8; 64 * 1024];
        loop {
            if control.interrupted() { return Err(EmuBoxError::ProcessFailed("Verificacion interrumpida".into())); }
            let size = file.read(&mut buffer).map_err(io_error)?;
            if size == 0 { break; } hasher.update(&buffer[..size]);
        }
        if !format!("{:x}", hasher.finalize()).eq_ignore_ascii_case(expected) {
            return Err(EmuBoxError::InvalidConfiguration("Checksum SHA-256 no coincide; archivo no publicado".into()));
        }
    }
    Ok(())
}

pub fn launch_target(platform: &str, files: &[PathBuf]) -> Option<PathBuf> {
    let specification = super::game_service::PLATFORM_SPECS.iter().find(|specification| specification.id == platform)?;
    let candidates: Vec<_> = files.iter().filter(|path| {
        let extension = path.extension().and_then(|value| value.to_str()).unwrap_or("").to_ascii_lowercase();
        !matches!(extension.as_str(), "zip" | "7z" | "rar" | "pkg" | "exe" | "bin") && specification.extensions.contains(&extension.as_str())
    }).collect();
    let descriptors: Vec<_> = candidates.iter().filter(|path| path.extension().is_some_and(|extension| extension == "cue" || extension == "m3u")).collect();
    if let [path] = descriptors.as_slice() { return Some((***path).clone()); }
    match candidates.as_slice() { [path] => Some((*path).clone()), _ => None }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn archives_and_installers_are_not_marked_playable() {
        assert_eq!(launch_target("ps3", &["game.pkg".into()]), None);
        assert_eq!(launch_target("ps2", &["game.zip".into()]), None);
        assert_eq!(launch_target("ps2", &["game.iso".into()]), Some("game.iso".into()));
        assert_eq!(launch_target("ps2", &["one.iso".into(), "two.iso".into()]), None);
    }

    #[test]
    fn zip_preparation_preserves_relative_paths() {
        let root = std::env::temp_dir().join(format!("emubox-zip-{}",std::process::id()));
        fs::create_dir_all(&root).unwrap(); let archive = root.join("content.zip");
        let mut zip = zip::ZipWriter::new(fs::File::create(&archive).unwrap());
        zip.start_file("folder/game.chd",zip::write::SimpleFileOptions::default()).unwrap();
        zip.write_all(b"local test data").unwrap(); zip.finish().unwrap();
        let files = prepare(&[archive],&root,&TransferControl::default()).unwrap();
        assert_eq!(files[0],root.join("extracted/folder/game.chd"));
        assert_eq!(fs::read(&files[0]).unwrap(),b"local test data");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn zip_traversal_is_rejected_without_writing_outside() {
        let root = std::env::temp_dir().join(format!("emubox-zip-security-{}",std::process::id()));
        fs::create_dir_all(&root).unwrap(); let archive = root.join("content.zip");
        let mut zip = zip::ZipWriter::new(fs::File::create(&archive).unwrap());
        zip.start_file("../escape.bin",zip::write::SimpleFileOptions::default()).unwrap();
        zip.write_all(b"data").unwrap(); zip.finish().unwrap();
        assert!(prepare(&[archive],&root,&TransferControl::default()).is_err());
        assert!(!root.join("escape.bin").exists()); fs::remove_dir_all(root).unwrap();
    }
}
use super::InstallerKind;
use std::{fs, path::{Path, PathBuf}};

pub fn prepared_metadata(
    original: &[PathBuf],
    prepared: &[PathBuf],
    root: &Path,
) -> Option<crate::models::PreparedInstallation> {
    use std::io::Read;
    if original.len() != 1
        || prepared.is_empty()
        || !prepared
            .iter()
            .all(|path| path.starts_with(root.join("extracted")))
    {
        return None;
    }
    let mut header = [0u8; 4];
    fs::File::open(&original[0])
        .ok()?
        .read_exact(&mut header)
        .ok()?;
    Some(crate::models::PreparedInstallation {
        kind: kind(&header)?,
        root: "extracted".into(),
    })
}

pub(super) fn windows_executable(path: &Path) -> bool {
    use std::io::{Read, Seek, SeekFrom};
    let Ok(mut file) = fs::File::open(path) else {
        return false;
    };
    let mut header = [0u8; 64];
    if file.read_exact(&mut header).is_err() || !header.starts_with(b"MZ") {
        return false;
    }
    let offset = u32::from_le_bytes(header[60..64].try_into().unwrap()) as u64;
    if offset < 64 || offset > 1024 * 1024 || file.seek(SeekFrom::Start(offset)).is_err() {
        return false;
    }
    let mut pe = [0u8; 24];
    file.read_exact(&mut pe).is_ok()
        && pe.starts_with(b"PE\0\0")
        && matches!(u16::from_le_bytes([pe[4], pe[5]]), 0x14c | 0x8664)
        && u16::from_le_bytes([pe[22], pe[23]]) & 0x2000 == 0
}

pub fn candidates(
    platform: &str,
    destination: &Path,
    package: &crate::models::PublishedDownload,
) -> Vec<String> {
    let Some(installation) = &package.installation else {
        return Vec::new();
    };
    if installation.root.as_os_str().is_empty()
        || installation
            .root
            .components()
            .any(|part| !matches!(part, std::path::Component::Normal(_)))
    {
        return Vec::new();
    }
    package
        .files
        .iter()
        .filter(|relative| {
            if !relative.starts_with(&installation.root) {
                return false;
            }
            let path = destination.join(relative);
            match installation.kind {
                InstallerKind::Inno if platform == "pc" => {
                    let name = path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("")
                        .to_ascii_lowercase();
                    relative.starts_with(installation.root.join("app"))
                        && name.ends_with(".exe")
                        && !name.starts_with("unins")
                        && !name.starts_with("setup")
                        && windows_executable(&path)
                }
                InstallerKind::Ps3 if platform == "ps3" => {
                    use std::io::Read;
                    let base = installation.root.join("home/config/rpcs3/dev_hdd0/game");
                    if !relative.starts_with(base)
                        || path.file_name().is_none_or(|name| name != "EBOOT.BIN")
                        || path
                            .parent()
                            .and_then(Path::file_name)
                            .is_none_or(|name| name != "USRDIR")
                    {
                        return false;
                    }
                    let Some(title) = relative.parent().and_then(Path::parent) else {
                        return false;
                    };
                    let sfo = title.join("PARAM.SFO");
                    if !package.files.contains(&sfo) {
                        return false;
                    }
                    let mut header = [0u8; 4];
                    fs::File::open(destination.join(sfo))
                        .and_then(|mut file| file.read_exact(&mut header))
                        .is_ok()
                        && header == *b"\0PSF"
                        && fs::File::open(path)
                            .and_then(|mut file| file.read_exact(&mut header))
                            .is_ok()
                        && matches!(&header, b"SCE\0" | b"\x7fELF")
                }
                _ => false,
            }
        })
        .map(|path| path.to_string_lossy().into_owned())
        .collect()
}

pub fn kind(header: &[u8]) -> Option<InstallerKind> {
    if header.starts_with(b"MZ") {
        Some(InstallerKind::Inno)
    } else if header.starts_with(b"\x7fPKG") {
        Some(InstallerKind::Ps3)
    } else {
        None
    }
}


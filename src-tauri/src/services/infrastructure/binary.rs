use std::fs::File;
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use crate::models::Architecture;

pub fn resolve_executable(executable: &str) -> Option<PathBuf> {
    let path = Path::new(executable);
    if executable.is_empty() {
        return None;
    }
    if path.is_absolute() || path.components().count() > 1 {
        return path.is_file().then(|| path.to_path_buf());
    }
    std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
        .map(|directory| directory.join(path))
        .find(|candidate| candidate.is_file() && candidate.metadata()
            .map(|metadata| metadata.permissions().mode() & 0o111 != 0).unwrap_or(false))
}

pub fn elf_architecture(header: &[u8]) -> Architecture {
    if header.len() < 20 || &header[..6] != b"\x7fELF\x02\x01" {
        return Architecture::Unsupported;
    }
    match u16::from_le_bytes([header[18], header[19]]) {
        62 => Architecture::X86_64,
        183 => Architecture::Aarch64,
        _ => Architecture::Unsupported,
    }
}

pub fn validate_binary(path: &Path, host: Architecture, executable: bool) -> Result<Architecture, String> {
    validate_inner(path, host, executable, 0)
}

fn validate_inner(path: &Path, host: Architecture, executable: bool, depth: usize) -> Result<Architecture, String> {
    if depth > 4 {
        return Err(format!("Wrapper interpreter recursion: {}", path.display()));
    }
    let metadata = path.metadata().map_err(|error| format!("{}: {error}", path.display()))?;
    if !metadata.is_file() || (executable && metadata.permissions().mode() & 0o111 == 0) {
        return Err(format!("Not an executable file: {}", path.display()));
    }
    let mut header = [0u8; 256];
    let size = File::open(path).and_then(|mut file| file.read(&mut header))
        .map_err(|error| error.to_string())?;
    if executable && header[..size].starts_with(b"#!") {
        let text = String::from_utf8_lossy(&header[2..size]);
        let words: Vec<_> = text.lines().next().unwrap_or("").split_whitespace().collect();
        let interpreter = words.first().ok_or("Missing wrapper interpreter")?;
        let interpreter = if Path::new(interpreter).file_name().is_some_and(|name| name == "env") {
            words.get(1).filter(|word| !word.starts_with('-')).ok_or("Unsupported env wrapper; configure an explicit interpreter")?
        } else {
            interpreter
        };
        let resolved = resolve_executable(interpreter).ok_or("Wrapper interpreter not installed")?;
        return validate_inner(&resolved, host, true, depth + 1);
    }
    let binary = elf_architecture(&header[..size]);
    if host == Architecture::Unsupported || binary != host {
        return Err(format!("Unsupported architecture: host={}, binary={}, path={}", host.as_str(), binary.as_str(), path.display()));
    }
    Ok(binary)
}

pub fn resolve_core(core: &str) -> Option<PathBuf> {
    let path = Path::new(core);
    if path.is_absolute() {
        return path.is_file().then(|| path.to_path_buf());
    }
    let name = path.file_name()?;
    [PathBuf::from(core), Path::new("/usr/lib/libretro").join(name),
        Path::new("/usr/local/lib/libretro").join(name)]
        .into_iter().find(|candidate| candidate.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binaries_and_cores_matrix() {
        let directory = std::env::temp_dir().join(format!("emubox-binary-test-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        for (machine, architecture) in [(62u16, Architecture::X86_64), (183, Architecture::Aarch64)] {
            let path = directory.join(architecture.as_str());
            let mut header = [0u8; 64];
            header[..6].copy_from_slice(b"\x7fELF\x02\x01");
            header[18..20].copy_from_slice(&machine.to_le_bytes());
            std::fs::write(&path, header).unwrap();
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
            for host in [Architecture::X86_64, Architecture::Aarch64, Architecture::Unsupported] {
                assert_eq!(validate_binary(&path, host, true).is_ok(), host == architecture);
                assert_eq!(validate_binary(&path, host, false).is_ok(), host == architecture);
            }
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
            assert!(validate_binary(&path, architecture, true).is_err());
            assert!(validate_binary(&path, architecture, false).is_ok());
        }
        assert!(validate_binary(&directory.join("missing"), Architecture::current(), true).is_err());
        assert_eq!(elf_architecture(b"not an ELF"), Architecture::Unsupported);
        let wrapper = directory.join("wrapper");
        std::fs::write(&wrapper, "#!/bin/sh\nexit 0\n").unwrap();
        std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(validate_binary(&wrapper, Architecture::current(), true).is_ok());
        assert!(validate_binary(&wrapper, Architecture::current(), false).is_err());
        std::fs::remove_dir_all(directory).unwrap();
    }
}
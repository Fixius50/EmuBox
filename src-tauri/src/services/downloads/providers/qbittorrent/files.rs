use super::failure;
use crate::{errors::EmuBoxError, services::downloads::providers::io_error};
use serde::Deserialize;
use std::{
    fs,
    path::{Component, Path, PathBuf},
};

#[derive(Deserialize)]
pub(super) struct Status {
    pub hash: String,
    pub state: String,
    pub amount_left: i64,
    pub total_size: i64,
    pub completed: u64,
    pub dlspeed: u64,
}

pub(super) fn validated_files(root: &Path, text: &str) -> Result<Vec<PathBuf>, EmuBoxError> {
    #[derive(Deserialize)]
    struct File {
        name: String,
        progress: f64,
    }
    let files: Vec<File> = serde_json::from_str(text)
        .map_err(|_| failure("Lista de archivos qBittorrent invalida"))?;
    if files.is_empty() {
        return Err(failure("Torrent completo sin archivos"));
    }
    files
        .into_iter()
        .map(|file| {
            let relative = Path::new(&file.name);
            if relative.as_os_str().is_empty()
                || relative
                    .components()
                    .any(|component| !matches!(component, Component::Normal(_)))
                || file.progress < 1.0
            {
                return Err(failure("Archivo torrent incompleto o fuera del trabajo"));
            }
            let path = root.join(relative);
            let resolved = fs::canonicalize(&path).map_err(io_error)?;
            if !resolved.starts_with(root)
                || !resolved.is_file()
                || fs::symlink_metadata(&path)
                    .map_err(io_error)?
                    .file_type()
                    .is_symlink()
            {
                return Err(failure(
                    "qBittorrent intento publicar una ruta no confinada",
                ));
            }
            Ok(path)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::downloads::providers::qbittorrent::engine::private_dir;

    #[test]
    fn completed_files_reject_escape_and_incomplete_data() {
        let root = std::env::temp_dir().join(format!("emubox-qbit-files-{}", std::process::id()));
        private_dir(&root).unwrap();
        fs::write(root.join("valid.bin"), b"data").unwrap();
        assert_eq!(
            validated_files(&root, r#"[{"name":"valid.bin","progress":1}]"#)
                .unwrap()
                .len(),
            1
        );
        for text in [
            r#"[{"name":"../escape","progress":1}]"#,
            r#"[{"name":"/etc/passwd","progress":1}]"#,
            r#"[{"name":"valid.bin","progress":0.5}]"#,
            "[]",
        ] {
            assert!(validated_files(&root, text).is_err());
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn metadata_keeps_unknown_sizes_signed_until_interpreted() {
        let metadata: Status = serde_json::from_str(r#"{"hash":"fixture","state":"metaDL","amount_left":-1,"total_size":-1,"completed":0,"dlspeed":0}"#).unwrap();
        assert!(u64::try_from(metadata.total_size).is_err());
    }
}

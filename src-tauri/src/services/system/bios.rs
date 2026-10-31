use crate::errors::EmuBoxError;
use crate::models::{BiosFile, BiosManifest, BiosRequirement, BiosStatus};
use std::collections::HashMap;
use std::path::Path;

pub struct BiosService;

impl BiosService {
    pub fn get_bios_requirements() -> Result<BiosStatus, EmuBoxError> {
        let manifest: BiosManifest = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../data/bios/bios-manifest.json"
        )))
        .map_err(|error| EmuBoxError::InvalidConfiguration(error.to_string()))?;
        Self::inspect(manifest, Path::new(&crate::services::paths::bios_dir()))
    }

    pub fn scan_bios() -> Result<BiosStatus, EmuBoxError> {
        Self::get_bios_requirements()
    }

    fn inspect(manifest: BiosManifest, root: &Path) -> Result<BiosStatus, EmuBoxError> {
        let mut status = BiosStatus {
            total_required: 0,
            total_found: 0,
            missing_required_count: 0,
            platforms: HashMap::new(),
        };
        for definition in manifest.requirements {
            let mut requirement = BiosRequirement {
                platform_id: definition.platform_id,
                platform_name: definition.platform_name,
                emulator_id: definition.emulator_id,
                bios_files: Vec::new(),
                all_required_present: true,
            };
            for bios in definition.bios_files {
                let found = [
                    root.join(&requirement.platform_id).join(&bios.filename),
                    root.join(&bios.filename),
                ]
                .into_iter()
                .find(|path| path.is_file());
                let mut valid = found.is_some();
                if bios.required {
                    status.total_required += 1;
                }
                if let Some(path) = &found {
                    let metadata = std::fs::metadata(path).ok();
                    let file_size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
                    let file_mtime = metadata
                        .as_ref()
                        .and_then(|m| m.modified().ok())
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs())
                        .unwrap_or(0);

                    let mut cached_valid: Option<bool> = None;
                    if let Ok(conn) = crate::services::infrastructure::database::DatabaseService::get_connection() {
                        cached_valid = conn.query_row(
                            "SELECT is_valid FROM bios_cache WHERE filename = ?1 AND platform_id = ?2 AND file_size = ?3 AND file_mtime = ?4",
                            rusqlite::params![bios.filename, requirement.platform_id, file_size, file_mtime],
                            |row| row.get::<_, i32>(0),
                        ).ok().map(|v| v != 0);
                    }

                    if let Some(cached) = cached_valid {
                        valid = cached;
                    } else {
                        for (program, expected) in [
                            ("md5sum", &bios.expected_md5),
                            ("sha1sum", &bios.expected_sha1),
                        ] {
                            if let Some(expected) = expected {
                                let digest = crate::services::host_command::output(
                                    program,
                                    &[path.to_str().ok_or_else(|| {
                                        EmuBoxError::InvalidConfiguration("Ruta BIOS no UTF-8".into())
                                    })?],
                                )?;
                                valid &= digest
                                    .split_whitespace()
                                    .next()
                                    .is_some_and(|value| value.eq_ignore_ascii_case(expected));
                            }
                        }
                        if let Ok(conn) = crate::services::infrastructure::database::DatabaseService::get_connection() {
                            let now = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .map(|d| d.as_secs())
                                .unwrap_or(0);
                            let _ = conn.execute(
                                "INSERT INTO bios_cache (filename, platform_id, file_size, file_mtime, is_valid, checked_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6) ON CONFLICT(filename, platform_id) DO UPDATE SET file_size=excluded.file_size, file_mtime=excluded.file_mtime, is_valid=excluded.is_valid, checked_at=excluded.checked_at",
                                rusqlite::params![bios.filename, requirement.platform_id, file_size, file_mtime, if valid { 1 } else { 0 }, now],
                            );
                        }
                    }
                    status.total_found += 1;
                }
                if bios.required && !valid {
                    status.missing_required_count += 1;
                    requirement.all_required_present = false;
                }
                let state = match (&found, valid, bios.required) {
                    (Some(_), true, _) => "found_valid",
                    (Some(_), false, _) => "found_invalid_checksum",
                    (None, _, true) => "required_missing",
                    (None, _, false) => "optional_missing",
                };
                let file_size_bytes = found
                    .as_ref()
                    .map(|path| std::fs::metadata(path).map(|metadata| metadata.len()))
                    .transpose()
                    .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
                requirement.bios_files.push(BiosFile {
                    filename: bios.filename,
                    description: bios.description,
                    expected_md5: bios.expected_md5,
                    expected_sha1: bios.expected_sha1,
                    found_path: found.map(|path| path.to_string_lossy().into()),
                    state: state.into(),
                    file_size_bytes,
                });
            }
            status
                .platforms
                .insert(requirement.platform_id.clone(), requirement);
        }
        Ok(status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shipped_requirements_are_not_empty() {
        let manifest = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../data/bios/bios-manifest.json"
        )))
        .unwrap();
        let root = std::env::temp_dir().join(format!("emubox-absent-bios-{}", std::process::id()));
        let status = BiosService::inspect(manifest, &root).unwrap();
        assert!(status.total_required > 0);
        assert_eq!(status.missing_required_count, status.total_required);
        assert_eq!(status.total_found, 0);
    }
}

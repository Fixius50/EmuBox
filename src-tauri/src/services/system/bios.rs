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

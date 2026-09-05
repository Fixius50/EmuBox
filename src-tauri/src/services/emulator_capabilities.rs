use std::{collections::HashMap, sync::LazyLock};
use serde::Deserialize;
use crate::models::{Architecture, Emulator, EmulatorCompatibility, EmulatorRequirements, HardwareInfo};
use super::binary_service::{resolve_core, resolve_executable, validate_binary};

#[derive(Default, Deserialize)]
pub struct Definition {
    pub architectures: Vec<Architecture>,
    #[serde(default)]
    pub requirements: EmulatorRequirements,
}

static DEFINITIONS: LazyLock<HashMap<String, Definition>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../../data/emulator-capabilities.json"))
        .expect("Invalid embedded emulator capability manifest")
});

pub fn supports(id: &str, host: Architecture) -> bool {
    DEFINITIONS.get(id).is_some_and(|definition| definition.architectures.contains(&host))
}

pub fn refresh(emulator: &mut Emulator, host: Architecture, hardware: &HardwareInfo) {
    if let Some(definition) = DEFINITIONS.get(&emulator.id) {
        emulator.architectures = definition.architectures.clone();
        emulator.requirements = definition.requirements.clone();
    }
    let mut compatibility = EmulatorCompatibility {
        status: "supported".into(), reason: String::new(),
        host_architecture: host.as_str().into(), binary_architecture: None,
    };
    let checked = (|| -> Result<(), (&str, String)> {
        if host == Architecture::Unsupported || !emulator.architectures.contains(&host) {
            return Err(("unsupported_architecture", format!("{} no admite {} en EmuBox", emulator.name, host.as_str())));
        }
        let path = resolve_executable(&emulator.executable)
            .ok_or_else(|| ("not_installed", format!("{} no esta instalado", emulator.name)))?;
        let binary = validate_binary(&path, host, true).map_err(|reason| (
            if reason.starts_with("Unsupported architecture") { "unsupported_architecture" } else { "invalid_binary" }, reason))?;
        compatibility.binary_architecture = Some(binary.as_str().into());
        for arguments in emulator.arguments.windows(2) {
            if arguments[0] == "-L" || arguments[0] == "--libretro" {
                let core = resolve_core(&arguments[1])
                    .ok_or_else(|| ("not_installed", format!("Core no instalado: {}", arguments[1])))?;
                validate_binary(&core, host, false).map_err(|reason| ("unsupported_architecture", reason))?;
            }
        }
        if hardware.cpu_cores < emulator.requirements.min_cpu_cores
            || hardware.total_memory_mb < emulator.requirements.min_memory_mb
            || (emulator.requirements.vulkan && !hardware.vulkan_supported) {
            return Err(("requirements_not_met", format!("{} requiere {} nucleos, {} MiB y Vulkan={}", emulator.name,
                emulator.requirements.min_cpu_cores, emulator.requirements.min_memory_mb, emulator.requirements.vulkan)));
        }
        Ok(())
    })();
    if let Err((status, reason)) = checked {
        compatibility.status = status.into();
        compatibility.reason = reason;
    }
    emulator.compatibility = compatibility;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn emulator_architecture_matrix() {
        assert!(supports("pcsx2", Architecture::X86_64));
        assert!(!supports("pcsx2", Architecture::Aarch64));
        assert!(supports("rpcs3", Architecture::Aarch64));
        assert!(supports("retroarch", Architecture::Aarch64));
        assert!(!supports("unverified", Architecture::Aarch64));
        assert!(!supports("retroarch", Architecture::Unsupported));
    }
}
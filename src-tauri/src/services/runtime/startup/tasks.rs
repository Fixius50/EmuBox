use crate::{
    errors::EmuBoxError,
    models::{Emulator, HardwareInfo, Platform, SystemSettings},
    services::{download_manager, EmulatorService, GameService, SystemService},
};
use std::{path::PathBuf, sync::Arc};

use super::Task;

pub(super) enum Output {
    Library(SystemSettings, Vec<Platform>, Vec<crate::models::Game>),
    Hardware(HardwareInfo),
    Services,
    Emulators(Vec<Emulator>),
}

pub(super) type Execute = Arc<
    dyn Fn(Task, Option<HardwareInfo>) -> Result<(Output, Vec<String>), EmuBoxError> + Send + Sync,
>;

pub(super) fn execute(
    task: Task,
    hardware: Option<HardwareInfo>,
) -> Result<(Output, Vec<String>), EmuBoxError> {
    let mut warnings = Vec::new();
    let output = match task {
        Task::Library => {
            SystemService::get_config()?;
            let settings = SystemService::get_settings()?;
            download_manager::recover()?;
            Output::Library(
                settings,
                GameService::get_platforms()?,
                GameService::get_games(None)?,
            )
        }
        Task::Hardware => {
            let hardware = SystemService::get_hardware_info()?;
            if hardware.graphics.detection_state
                == crate::models::graphics::DetectionState::Indeterminate
            {
                warnings.push(
                    "Deteccion grafica indeterminada; se conserva el backend operativo".into(),
                );
            }
            Output::Hardware(hardware)
        }
        Task::Services => {
            let runtime = std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from);
            for service in ["bus", "pipewire-0", "pulse/native"] {
                if !runtime
                    .as_ref()
                    .is_some_and(|path| path.join(service).exists())
                {
                    warnings.push(format!("Servicio de sesion no disponible: {service}"));
                }
            }
            Output::Services
        }
        Task::Emulators => {
            let hardware = hardware.ok_or_else(|| {
                EmuBoxError::HardwareUnavailable("Falta instantanea de hardware".into())
            })?;
            let emulators = match EmulatorService::scan_with_hardware(&hardware) {
                Ok(emulators) => emulators,
                Err(error) => {
                    warnings.push(format!("Inventario de emuladores incompleto: {error}"));
                    EmulatorService::cached_with_hardware(&hardware)?
                }
            };
            if emulators
                .iter()
                .any(|emulator| emulator.version == "Instalado (version no disponible)")
            {
                warnings.push(
                    "No se pudo consultar la version de algunos emuladores dentro del plazo".into(),
                );
            }
            if let Err(error) = EmulatorService::apply_hardware_profile(&hardware) {
                warnings.push(format!("Perfiles de emuladores: {error}"));
            }
            Output::Emulators(emulators)
        }
    };
    Ok((output, warnings))
}

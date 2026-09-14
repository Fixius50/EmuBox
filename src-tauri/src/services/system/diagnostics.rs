use crate::errors::EmuBoxError;
use crate::models::{DiagnosticReport, LogEntry};
use std::process::Command;

pub struct DiagnosticsService;

impl DiagnosticsService {
    pub fn get_system_logs(limit: Option<usize>) -> Result<Vec<LogEntry>, EmuBoxError> {
        crate::services::log_service::system(limit)
    }

    pub fn get_emubox_logs(limit: Option<usize>) -> Result<Vec<LogEntry>, EmuBoxError> {
        crate::services::log_service::session(limit)
    }

    pub fn get_diagnostics() -> Result<DiagnosticReport, EmuBoxError> {
        let system = crate::services::SystemService::get_system_info()?;
        let emulators = crate::services::EmulatorService::get_emulators()?;
        let installed = emulators
            .iter()
            .filter(|emulator| emulator.compatibility.status == "supported")
            .count();
        let hardware = &system.hardware;
        let summary = format!("architecture={} kernelArchitecture={} cpu={} cores={} memoryMiB={} gpuVendor={} renderer={} vulkan={} opengl={} openglAccelerated={} openglRenderer={} drm={} gamescope={} compositor={} device={} accelerated={} backend={} gpuKind={} vm={}",
            system.architecture, system.kernel_architecture, hardware.cpu_model, hardware.cpu_cores,
            hardware.total_memory_mb, hardware.gpu_vendor, hardware.gpu_renderer, hardware.vulkan_supported,
            hardware.opengl_supported, hardware.opengl_accelerated, hardware.opengl_renderer.as_deref().unwrap_or("unknown"),
            hardware.drm_available, hardware.gamescope_available, hardware.recommended_compositor, hardware.device_model,
            hardware.graphics_accelerated, hardware.graphics_backend, hardware.gpu_kind, hardware.is_virtual_machine);
        Ok(DiagnosticReport {
            generated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            os_info: system.os_name,
            kernel_version: system.kernel_version,
            architecture: system.architecture,
            gpu_adapter: hardware.gpu_renderer.clone(),
            vulkan_ready: hardware.vulkan_supported,
            gamescope_ready: hardware.gamescope_ready,
            pipewire_ready: Command::new("pgrep")
                .args(["-x", "pipewire"])
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false),
            storage_mounted: std::path::Path::new(&crate::services::paths::games_dir()).is_dir(),
            emulators_installed_count: installed,
            emulators_missing_count: emulators.len() - installed,
            connected_gamepads_count: crate::services::SystemService::detect_gamepads().len(),
            recent_errors: Self::get_system_logs(Some(100)).ok().map(|entries| {
                entries
                    .into_iter()
                    .filter(|entry| entry.level == "error")
                    .collect()
            }),
            raw_summary_text: format!("{summary}\ngraphicsEvidence={}", serde_json::to_string(&hardware.graphics)
                .map_err(|error| EmuBoxError::Unknown(error.to_string()))?),
        })
    }

    pub fn execute_command(cmd: &str) -> Result<String, EmuBoxError> {
        let output = Command::new("bash")
            .arg("-c")
            .arg(cmd)
            .output()
            .map_err(|e| EmuBoxError::ProcessFailed(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if output.status.success() {
            if stdout.is_empty() && !stderr.is_empty() {
                Ok(stderr)
            } else if stdout.is_empty() {
                Ok("(Comando ejecutado con éxito sin salida de texto)".to_string())
            } else {
                Ok(stdout)
            }
        } else {
            Ok(format!(
                "[ERROR - CÓDIGO: {}]\n{}\n{}",
                output.status.code().unwrap_or(-1),
                stdout,
                stderr
            ))
        }
    }
}

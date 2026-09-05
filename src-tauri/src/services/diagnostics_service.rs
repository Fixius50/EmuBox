use crate::models::{DiagnosticReport, LogEntry};
use crate::errors::EmuBoxError;
use std::process::Command;

pub struct DiagnosticsService;

impl DiagnosticsService {
    pub fn get_system_logs(_limit: Option<usize>) -> Result<Vec<LogEntry>, EmuBoxError> {
        Ok(vec![])
    }

    pub fn get_emubox_logs(_limit: Option<usize>) -> Result<Vec<LogEntry>, EmuBoxError> {
        Ok(vec![])
    }

    pub fn get_diagnostics() -> Result<DiagnosticReport, EmuBoxError> {
        let system = super::SystemService::get_system_info()?;
        let emulators = super::EmulatorService::get_emulators()?;
        let installed = emulators.iter().filter(|emulator| emulator.compatibility.status == "supported").count();
        let hardware = &system.hardware;
        let summary = format!("architecture={} kernelArchitecture={} cpu={} cores={} memoryMiB={} gpuVendor={} renderer={} vulkan={} drm={} gamescope={} compositor={} device={}",
            system.architecture, system.kernel_architecture, hardware.cpu_model, hardware.cpu_cores,
            hardware.total_memory_mb, hardware.gpu_vendor, hardware.gpu_renderer, hardware.vulkan_supported,
            hardware.drm_available, hardware.gamescope_available, hardware.recommended_compositor, hardware.device_model);
        Ok(DiagnosticReport {
            generated_at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),
            os_info: system.os_name,
            kernel_version: system.kernel_version,
            architecture: system.architecture,
            gpu_adapter: hardware.gpu_renderer.clone(),
            vulkan_ready: hardware.vulkan_supported,
            gamescope_ready: hardware.gamescope_available && hardware.vulkan_supported && hardware.drm_available,
            pipewire_ready: Command::new("pgrep").args(["-x", "pipewire"]).output().map(|output| output.status.success()).unwrap_or(false),
            storage_mounted: std::path::Path::new(&super::paths::games_dir()).is_dir(),
            emulators_installed_count: installed,
            emulators_missing_count: emulators.len() - installed,
            connected_gamepads_count: super::SystemService::detect_gamepads().len(),
            recent_errors: vec![],
            raw_summary_text: summary,
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
            Ok(format!("[ERROR - CÓDIGO: {}]\n{}\n{}", output.status.code().unwrap_or(-1), stdout, stderr))
        }
    }
}

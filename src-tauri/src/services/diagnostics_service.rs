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
        Ok(DiagnosticReport {
            generated_at: 1700000000,
            os_info: "EmuBox OS 1.0 (Arch Linux)".to_string(),
            kernel_version: "6.8.9-zen".to_string(),
            architecture: "x86_64".to_string(),
            gpu_adapter: "RADV Vulkan".to_string(),
            vulkan_ready: true,
            gamescope_ready: true,
            pipewire_ready: true,
            storage_mounted: true,
            emulators_installed_count: 5,
            emulators_missing_count: 0,
            connected_gamepads_count: 1,
            recent_errors: vec![],
            raw_summary_text: "EmuBox Diagnostic Report: Nominal".to_string(),
        })
    }

    pub fn execute_command(cmd: &str) -> Result<String, EmuBoxError> {
        let output = Command::new("bash")
            .arg("-c")
            .arg(cmd)
            .output()
            .map_err(|e| EmuBoxError::ExecutionFailed(e.to_string()))?;

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

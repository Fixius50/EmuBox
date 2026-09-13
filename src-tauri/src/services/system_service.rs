use crate::errors::EmuBoxError;
use crate::models::{
    AudioInfo, DisplayInfo, EmuBoxConfig, FirstRunDetectionResult, HardwareInfo, SystemInfo,
    SystemSettings,
};
use crate::services::paths;
use crate::services::EmulatorService;
use std::fs;
use std::process::Command;

pub struct SystemService;

impl SystemService {
    pub fn get_system_info() -> Result<SystemInfo, EmuBoxError> {
        Ok(SystemInfo {
            os_name: Self::detect_os_name(),
            kernel_version: Self::run_trim("uname", &["-r"])
                .unwrap_or_else(|| "unknown".to_string()),
            architecture: crate::models::Architecture::current().as_str().to_string(),
            kernel_architecture: Self::run_trim("uname", &["-m"])
                .unwrap_or_else(|| "unknown".into()),
            hostname: Self::detect_hostname(),
            uptime_seconds: Self::detect_uptime_seconds(),
            hardware: Self::get_hardware_info()?,
            display: Self::get_display_info().ok(),
            audio: Self::get_audio_info().ok(),
            battery_level_percent: Self::detect_battery_percent(),
            is_plugged_in: Self::detect_plugged_in(),
        })
    }

    pub fn get_hardware_info() -> Result<HardwareInfo, EmuBoxError> {
        let graphics = super::graphics_service::detect();
        let (mem_total_mb, mem_free_mb) = Self::detect_memory_mb();

        Ok(HardwareInfo {
            graphics: graphics.clone(),
            gpu_vendor: graphics.vendor,
            gpu_renderer: graphics.renderer,
            vulkan_driver_version: graphics.driver_version,
            vulkan_supported: graphics.vulkan,
            opengl_supported: graphics.opengl,
            opengl_accelerated: graphics.probes.iter().any(|probe| probe.api == "opengl" && probe.state == crate::models::graphics::DetectionState::Accelerated),
            opengl_renderer: graphics.opengl_renderer,
            graphics_accelerated: graphics.accelerated,
            graphics_backend: graphics.backend,
            gpu_kind: graphics.gpu_kind,
            is_virtual_machine: graphics.virtual_machine,
            gamescope_ready: graphics.gamescope_ready,
            drm_available: graphics.drm,
            gamescope_available: graphics.gamescope,
            recommended_compositor: graphics.compositor,
            device_model: graphics.device,
            cpu_model: Self::detect_cpu_model(),
            cpu_cores: Self::detect_cpu_cores(),
            cpu_architecture: crate::models::Architecture::current().as_str().to_string(),
            total_memory_mb: mem_total_mb,
            free_memory_mb: mem_free_mb,
        })
    }

    pub fn get_display_info() -> Result<DisplayInfo, EmuBoxError> {
        super::display_service::detect()
    }

    pub fn get_audio_info() -> Result<AudioInfo, EmuBoxError> {
        super::audio_service::detect()
    }

    pub fn first_run_detection() -> Result<FirstRunDetectionResult, EmuBoxError> {
        let hardware = Self::get_hardware_info()?;
        let vulkan_supported = hardware.vulkan_supported;

        // Reutiliza el escaneo oficial de emuladores (rutas reales + versión probada) en vez
        // de una detección propia duplicada, y aplica el renderer óptimo según el hardware real.
        let scanned_emulators = EmulatorService::scan_emulators()?;
        EmulatorService::apply_hardware_profile(&hardware)?;

        let installed_emulators: Vec<String> = scanned_emulators
            .into_iter()
            .filter(|e| e.status == "active")
            .map(|e| e.id)
            .collect();

        Ok(FirstRunDetectionResult {
            gpu_vendor: hardware.gpu_vendor,
            gpu_renderer: hardware.gpu_renderer,
            vulkan_supported,
            gamepads_detected: Self::detect_gamepads(),
            installed_emulators,
            roms_directory_found: std::path::Path::new(&paths::games_dir()).is_dir(),
            config_generated: std::path::Path::new(&paths::config_file()).is_file(),
        })
    }

    pub fn exit_to_linux_shell() -> Result<(), EmuBoxError> {
        let _ = std::fs::write("/tmp/emubox-drop-shell", "1");
        std::process::exit(0);
    }

    // --- Helpers de detección real (sin invocar shell, sin interpolar comandos) ---

    fn run_trim(bin: &str, args: &[&str]) -> Option<String> {
        let output = Command::new(bin).args(args).output().ok()?;
        if !output.status.success() {
            return None;
        }
        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if text.is_empty() {
            None
        } else {
            Some(text)
        }
    }

    fn detect_os_name() -> String {
        if let Ok(content) = fs::read_to_string("/etc/os-release") {
            for line in content.lines() {
                if let Some(value) = line.strip_prefix("PRETTY_NAME=") {
                    return value.trim_matches('"').to_string();
                }
            }
        }
        "Linux".to_string()
    }

    fn detect_hostname() -> String {
        fs::read_to_string("/proc/sys/kernel/hostname")
            .ok()
            .map(|s| s.trim().to_string())
            .or_else(|| Self::run_trim("hostname", &[]))
            .unwrap_or_else(|| "unknown".to_string())
    }

    fn detect_uptime_seconds() -> u64 {
        fs::read_to_string("/proc/uptime")
            .ok()
            .and_then(|content| content.split_whitespace().next().map(str::to_string))
            .and_then(|s| s.parse::<f64>().ok())
            .map(|secs| secs as u64)
            .unwrap_or(0)
    }

    fn detect_battery_percent() -> Option<u32> {
        fs::read_to_string("/sys/class/power_supply/BAT0/capacity")
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
    }

    fn detect_plugged_in() -> Option<bool> {
        for supply in ["AC", "ADP1", "AC0"] {
            if let Ok(content) =
                fs::read_to_string(format!("/sys/class/power_supply/{supply}/online"))
            {
                return content.trim().parse::<u8>().ok().map(|v| v == 1);
            }
        }
        None
    }

    fn detect_cpu_model() -> String {
        if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
            for line in content.lines() {
                if let Some((key, value)) = line.split_once(':') {
                    if matches!(key.trim(), "model name" | "Hardware" | "Model") {
                        return value.trim().to_string();
                    }
                }
            }
        }
        "Unknown CPU".to_string()
    }

    fn detect_cpu_cores() -> usize {
        if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
            let count = content
                .lines()
                .filter(|l| l.starts_with("processor"))
                .count();
            if count > 0 {
                return count;
            }
        }
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    }

    fn detect_memory_mb() -> (u64, u64) {
        let mut total_kb: u64 = 0;
        let mut available_kb: u64 = 0;
        if let Ok(content) = fs::read_to_string("/proc/meminfo") {
            for line in content.lines() {
                if let Some(value) = line.strip_prefix("MemTotal:") {
                    total_kb = Self::parse_kb(value);
                } else if let Some(value) = line.strip_prefix("MemAvailable:") {
                    available_kb = Self::parse_kb(value);
                }
            }
        }
        (total_kb / 1024, available_kb / 1024)
    }

    fn parse_kb(value: &str) -> u64 {
        value
            .trim()
            .trim_end_matches("kB")
            .trim()
            .parse::<u64>()
            .unwrap_or(0)
    }

    /// Lista gamepads reales conectados vía `/proc/bus/input/devices` (entradas con handler `js*`).
    pub(crate) fn detect_gamepads() -> Vec<String> {
        let Ok(content) = fs::read_to_string("/proc/bus/input/devices") else {
            return vec![];
        };

        let mut gamepads = Vec::new();
        let mut current_name: Option<String> = None;

        for line in content.lines() {
            if let Some(rest) = line.strip_prefix("N: Name=") {
                current_name = Some(rest.trim_matches('"').to_string());
            } else if line.starts_with("H: Handlers=") && line.contains("js") {
                if let Some(name) = current_name.take() {
                    gamepads.push(name);
                }
            } else if line.is_empty() {
                current_name = None;
            }
        }

        gamepads
    }

    /// Lee `/etc/emubox/config.json`; si no existe todavía (primer arranque antes
    /// de que el instalador lo copie), cae al `config.json` empaquetado con la app.
    pub fn get_config() -> Result<EmuBoxConfig, EmuBoxError> {
        super::config_service::get_config()
    }

    pub fn save_config(config: EmuBoxConfig) -> Result<(), EmuBoxError> {
        super::config_service::save_config(config)
    }

    /// Lee `/etc/emubox/settings.json`; cae al empaquetado con la app y, si falta
    /// alguna sección opcional heredada, la completa desde la configuracion de fabrica.
    pub fn get_settings() -> Result<SystemSettings, EmuBoxError> {
        super::config_service::get_settings()
    }

    pub fn save_settings(settings: SystemSettings) -> Result<bool, EmuBoxError> {
        super::config_service::save_settings(settings)
    }
}

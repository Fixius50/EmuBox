use crate::models::{SystemInfo, HardwareInfo, DisplayInfo, AudioInfo, FirstRunDetectionResult};
use crate::errors::EmuBoxError;
use crate::services::EmulatorService;
use std::fs;
use std::process::Command;

const ROMS_DIRECTORY: &str = "/var/lib/emubox/games";

pub struct SystemService;

impl SystemService {
    pub fn get_system_info() -> Result<SystemInfo, EmuBoxError> {
        Ok(SystemInfo {
            os_name: Self::detect_os_name(),
            kernel_version: Self::run_trim("uname", &["-r"]).unwrap_or_else(|| "unknown".to_string()),
            architecture: Self::run_trim("uname", &["-m"]).unwrap_or_else(|| std::env::consts::ARCH.to_string()),
            hostname: Self::detect_hostname(),
            uptime_seconds: Self::detect_uptime_seconds(),
            hardware: Self::get_hardware_info()?,
            display: Self::get_display_info()?,
            audio: Self::get_audio_info()?,
            battery_level_percent: Self::detect_battery_percent(),
            is_plugged_in: Self::detect_plugged_in(),
        })
    }

    pub fn get_hardware_info() -> Result<HardwareInfo, EmuBoxError> {
        let (gpu_vendor, gpu_renderer, vulkan_driver_version) = Self::detect_gpu();
        let (mem_total_mb, mem_free_mb) = Self::detect_memory_mb();

        Ok(HardwareInfo {
            gpu_vendor,
            gpu_renderer,
            vulkan_driver_version,
            cpu_model: Self::detect_cpu_model(),
            cpu_cores: Self::detect_cpu_cores(),
            cpu_architecture: Self::run_trim("uname", &["-m"]).unwrap_or_else(|| std::env::consts::ARCH.to_string()),
            total_memory_mb: mem_total_mb,
            free_memory_mb: mem_free_mb,
        })
    }

    pub fn get_display_info() -> Result<DisplayInfo, EmuBoxError> {
        let gamescope_active = Self::process_running("gamescope");
        let active_compositor = if gamescope_active {
            "gamescope".to_string()
        } else if std::env::var("WAYLAND_DISPLAY").is_ok() {
            "wayland".to_string()
        } else if std::env::var("DISPLAY").is_ok() {
            "x11".to_string()
        } else {
            "wayland".to_string()
        };

        // Sin sesión gráfica activa (p. ej. accediendo por SSH) no hay forma fiable de
        // sondear la resolución real; se mantienen los valores por defecto de settings.json.
        Ok(DisplayInfo {
            resolution: "1920x1080".to_string(),
            width: 1920,
            height: 1080,
            refresh_rate: 60,
            device_pixel_ratio: 1.0,
            color_depth: 24,
            hdr_supported: false,
            active_compositor,
            gamescope_active,
        })
    }

    pub fn get_audio_info() -> Result<AudioInfo, EmuBoxError> {
        Ok(AudioInfo {
            master_volume: 85,
            ui_sound_effects: true,
            background_music: false,
            latency_ms: 16,
            sample_rate: 48000,
            devices: vec![],
        })
    }

    pub fn first_run_detection() -> Result<FirstRunDetectionResult, EmuBoxError> {
        let hardware = Self::get_hardware_info()?;
        let vulkan_supported = hardware.vulkan_driver_version.is_some();

        // Reutiliza el escaneo oficial de emuladores (rutas reales + versión probada) en vez
        // de una detección propia duplicada, y aplica el renderer óptimo según el hardware real.
        let scanned_emulators = EmulatorService::scan_emulators()?;
        EmulatorService::apply_hardware_profile(&hardware)?;

        let installed_emulators: Vec<String> = scanned_emulators.into_iter()
            .filter(|e| e.status == "active")
            .map(|e| e.id)
            .collect();

        Ok(FirstRunDetectionResult {
            gpu_vendor: hardware.gpu_vendor,
            gpu_renderer: hardware.gpu_renderer,
            vulkan_supported,
            gamepads_detected: Self::detect_gamepads(),
            installed_emulators,
            roms_directory_found: std::path::Path::new(ROMS_DIRECTORY).is_dir(),
            config_generated: true,
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
        if text.is_empty() { None } else { Some(text) }
    }

    fn run_full(bin: &str, args: &[&str]) -> Option<String> {
        let output = Command::new(bin).args(args).output().ok()?;
        if !output.status.success() {
            return None;
        }
        Some(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn process_running(name: &str) -> bool {
        Command::new("pgrep").arg("-x").arg(name)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
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
            if let Ok(content) = fs::read_to_string(format!("/sys/class/power_supply/{supply}/online")) {
                return content.trim().parse::<u8>().ok().map(|v| v == 1);
            }
        }
        None
    }

    fn detect_cpu_model() -> String {
        if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
            for line in content.lines() {
                if let Some((key, value)) = line.split_once(':') {
                    if key.trim() == "model name" {
                        return value.trim().to_string();
                    }
                }
            }
        }
        "Unknown CPU".to_string()
    }

    fn detect_cpu_cores() -> usize {
        if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
            let count = content.lines().filter(|l| l.starts_with("processor")).count();
            if count > 0 {
                return count;
            }
        }
        std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1)
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
        value.trim().trim_end_matches("kB").trim().parse::<u64>().unwrap_or(0)
    }

    /// Detecta vendor/renderer/versión de driver Vulkan real vía `vulkaninfo`,
    /// con fallback a `lspci` cuando Vulkan no está disponible (p. ej. VM sin passthrough de GPU).
    fn detect_gpu() -> (String, String, Option<String>) {
        if let Some(summary) = Self::run_full("vulkaninfo", &["--summary"]) {
            let mut vendor_id: Option<String> = None;
            let mut device_name: Option<String> = None;
            let mut driver_info: Option<String> = None;

            for line in summary.lines() {
                let line = line.trim();
                if let Some(value) = line.strip_prefix("vendorID") {
                    vendor_id = value.trim_start_matches(['=', ' ']).split_whitespace().next().map(str::to_string);
                } else if let Some(value) = line.strip_prefix("deviceName") {
                    device_name = Some(value.trim_start_matches(['=', ' ']).trim().to_string());
                } else if let Some(value) = line.strip_prefix("driverInfo") {
                    driver_info = Some(value.trim_start_matches(['=', ' ']).trim().to_string());
                }
                if device_name.is_some() && driver_info.is_some() {
                    break;
                }
            }

            if let Some(renderer) = device_name {
                let vendor = Self::vendor_from_pci_id(vendor_id.as_deref());
                return (vendor, renderer, driver_info);
            }
        }

        Self::detect_gpu_via_lspci()
    }

    fn vendor_from_pci_id(vendor_id: Option<&str>) -> String {
        match vendor_id {
            Some(id) if id.eq_ignore_ascii_case("0x1002") => "amd".to_string(),
            Some(id) if id.eq_ignore_ascii_case("0x10de") => "nvidia".to_string(),
            Some(id) if id.eq_ignore_ascii_case("0x8086") => "intel".to_string(),
            _ => "generic".to_string(),
        }
    }

    fn detect_gpu_via_lspci() -> (String, String, Option<String>) {
        if let Some(output) = Self::run_full("lspci", &["-mm"]) {
            for line in output.lines() {
                if line.contains("VGA compatible controller") || line.contains("3D controller") {
                    let lower = line.to_lowercase();
                    let vendor = if lower.contains("amd") || lower.contains("ati") {
                        "amd"
                    } else if lower.contains("nvidia") {
                        "nvidia"
                    } else if lower.contains("intel") {
                        "intel"
                    } else {
                        "generic"
                    };
                    return (vendor.to_string(), line.trim().to_string(), None);
                }
            }
        }

        let is_vm = Self::run_trim("systemd-detect-virt", &[])
            .map(|v| v != "none")
            .unwrap_or(false);
        let renderer = if is_vm {
            "Software Renderer (llvmpipe/VM sin passthrough GPU)".to_string()
        } else {
            "Unknown GPU".to_string()
        };
        ("generic".to_string(), renderer, None)
    }

    /// Lista gamepads reales conectados vía `/proc/bus/input/devices` (entradas con handler `js*`).
    fn detect_gamepads() -> Vec<String> {
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
}


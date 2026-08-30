use crate::models::{SystemInfo, HardwareInfo, DisplayInfo, AudioInfo, FirstRunDetectionResult};
use crate::errors::EmuBoxError;

pub struct SystemService;

impl SystemService {
    pub fn get_system_info() -> Result<SystemInfo, EmuBoxError> {
        Ok(SystemInfo {
            os_name: "EmuBox OS (Arch Linux)".to_string(),
            kernel_version: "6.8.9-zen1-1-zen".to_string(),
            architecture: "x86_64".to_string(),
            hostname: "emubox".to_string(),
            uptime_seconds: 3600,
            hardware: Self::get_hardware_info()?,
            display: Self::get_display_info()?,
            audio: Self::get_audio_info()?,
            battery_level_percent: None,
            is_plugged_in: Some(true),
        })
    }

    pub fn get_hardware_info() -> Result<HardwareInfo, EmuBoxError> {
        Ok(HardwareInfo {
            gpu_vendor: "amd".to_string(),
            gpu_renderer: "AMD Radeon Graphics (RADV Vulkan 1.3)".to_string(),
            vulkan_driver_version: Some("24.0.4".to_string()),
            cpu_model: "AMD Processor".to_string(),
            cpu_cores: 8,
            cpu_architecture: "x86_64".to_string(),
            total_memory_mb: 16384,
            free_memory_mb: 12288,
        })
    }

    pub fn get_display_info() -> Result<DisplayInfo, EmuBoxError> {
        Ok(DisplayInfo {
            resolution: "1920x1080".to_string(),
            width: 1920,
            height: 1080,
            refresh_rate: 60,
            device_pixel_ratio: 1.0,
            color_depth: 24,
            hdr_supported: false,
            active_compositor: "gamescope".to_string(),
            gamescope_active: true,
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
        Ok(FirstRunDetectionResult {
            gpu_vendor: "amd".to_string(),
            gpu_renderer: "AMD Radeon Graphics".to_string(),
            vulkan_supported: true,
            gamepads_detected: vec!["Xbox Controller".to_string()],
            installed_emulators: vec!["retroarch".to_string(), "duckstation-qt".to_string()],
            roms_directory_found: true,
            config_generated: true,
        })
    }

    pub fn exit_to_linux_shell() -> Result<(), EmuBoxError> {
        let _ = std::fs::write("/tmp/emubox-drop-shell", "1");
        std::process::exit(0);
    }
}

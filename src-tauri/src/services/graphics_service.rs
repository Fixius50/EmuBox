use std::{fs, path::Path, process::Command};

#[derive(Default)]
pub struct GraphicsCapabilities {
    pub vendor: String,
    pub renderer: String,
    pub driver_version: Option<String>,
    pub vulkan: bool,
    pub opengl: bool,
    pub opengl_renderer: Option<String>,
    pub drm: bool,
    pub gamescope: bool,
    pub cage: bool,
    pub accelerated: bool,
    pub backend: String,
    pub gpu_kind: String,
    pub virtual_machine: bool,
    pub gamescope_ready: bool,
    pub compositor: String,
    pub device: String,
}

pub fn vendor_from_text(text: &str) -> &'static str {
    let lower = text.to_lowercase();
    let words: Vec<_> = lower.split(|character: char| !character.is_alphanumeric()).collect();
    if ["vmware", "vmwgfx", "virtualbox", "virtio", "qxl", "virgl", "svga3d", "venus"].iter().any(|word| words.contains(word)) { "virtual" }
    else if ["amd", "ati", "radeon", "amdgpu", "1002"].iter().any(|word| words.contains(word)) { "amd" }
    else if ["nvidia", "nouveau", "10de"].iter().any(|word| words.contains(word)) { "nvidia" }
    else if ["intel", "i915", "xe", "8086"].iter().any(|word| words.contains(word)) { "intel" }
    else if ["broadcom", "v3d", "vc4", "14e4"].iter().any(|word| words.contains(word)) { "broadcom" }
    else if ["mali", "panfrost", "panthor", "lima", "13b5"].iter().any(|word| words.contains(word)) { "mali" }
    else if ["qualcomm", "adreno", "msm", "freedreno", "5143"].iter().any(|word| words.contains(word)) { "qualcomm" }
    else if ["apple", "asahi", "agx", "106b"].iter().any(|word| words.contains(word)) { "apple" }
    else { "unknown" }
}

pub fn hardware_vulkan(summary: &str) -> Option<(String, Option<String>)> {
    let mut name = None;
    let mut version = None;
    let mut hardware = false;
    for line in summary.lines().chain(std::iter::once("GPU999:")) {
        let line = line.trim();
        if line.starts_with("GPU") && line.ends_with(':') {
            if hardware && name.as_deref().is_some_and(|renderer| !is_software_renderer(renderer)) { return name.map(|name| (name, version)); }
            name = None;
            version = None;
            hardware = false;
        }
        if let Some((key, value)) = line.split_once('=') {
            let value = value.trim();
            match key.trim() {
                "deviceType" => hardware = value.contains("DISCRETE_GPU") || value.contains("INTEGRATED_GPU") || value.contains("VIRTUAL_GPU"),
                "deviceName" => name = Some(value.to_string()),
                "driverInfo" => version = Some(value.to_string()),
                _ => {}
            }
        }
    }
    None
}

pub fn select_backend(vulkan: bool, opengl_accelerated: bool) -> &'static str {
    if opengl_accelerated { "opengl" } else if vulkan { "vulkan" } else { "software" }
}

pub fn gamescope_supported(vulkan: bool, executable: bool, display_available: bool) -> bool {
    vulkan && executable && display_available
}

pub fn select_session(preference: &str, backend: &str, drm: bool, cage: bool, gamescope_ready: bool) -> &'static str {
    if !drm { return "unavailable"; }
    if preference == "gamescope" && backend != "software" && gamescope_ready { return "gamescope"; }
    if cage && matches!(backend, "opengl" | "software") { return "cage"; }
    if backend != "software" && gamescope_ready { return "gamescope"; }
    "unavailable"
}

pub fn is_software_renderer(renderer: &str) -> bool {
    let lower = renderer.to_lowercase();
    ["llvmpipe", "softpipe", "swrast", "software rasterizer", "swiftshader", "lavapipe", "microsoft basic render", "gdi generic", "mesa software"]
        .iter().any(|software| lower.contains(software))
}

pub fn opengl_renderer(summary: &str) -> Option<String> {
    let renderers: Vec<_> = summary.lines().filter_map(|line| {
        let (key, value) = line.trim().split_once(':')?;
        if matches!(key, "OpenGL core profile renderer" | "OpenGL compatibility profile renderer"
            | "OpenGL ES profile renderer" | "OpenGL renderer string") && !value.trim().is_empty() {
            Some(value.trim().to_string())
        } else {
            None
        }
    }).collect();
    renderers.iter().find(|renderer| !is_software_renderer(renderer))
        .or_else(|| renderers.first()).cloned()
}

pub fn detect() -> GraphicsCapabilities {
    let mut result = GraphicsCapabilities {
        vendor: "unknown".into(), renderer: "Unknown GPU".into(),
        device: fs::read_to_string("/sys/firmware/devicetree/base/model")
            .or_else(|_| fs::read_to_string("/sys/class/dmi/id/product_name"))
            .unwrap_or_else(|_| "unknown".into()).trim_matches(['\0', '\n']).to_string(),
        gamescope: super::binary_service::resolve_executable("gamescope").is_some(),
        cage: super::binary_service::resolve_executable("cage").is_some(),
        ..Default::default()
    };
    if let Ok(entries) = fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with("card") || name.contains('-') { continue; }
            result.drm |= Path::new("/dev/dri").join(&name).exists();
            let device = entry.path().join("device");
            let driver = fs::read_link(device.join("driver")).ok()
                .and_then(|path| path.file_name().map(|name| name.to_string_lossy().to_string())).unwrap_or_default();
            let uevent = fs::read_to_string(device.join("uevent")).unwrap_or_default();
            let vendor = vendor_from_text(&format!("{driver} {uevent}"));
            if result.vendor == "unknown" {
                result.vendor = vendor.into();
                result.renderer = if driver.is_empty() { "Unknown DRM renderer".into() } else { driver };
            }
        }
    }
    if let Ok(output) = Command::new("timeout").args(["10s", "vulkaninfo", "--summary"]).env("LC_ALL", "C").output() {
        if output.status.success() {
            if let Some((renderer, version)) = hardware_vulkan(&String::from_utf8_lossy(&output.stdout)) {
                let vendor = vendor_from_text(&renderer);
                if vendor != "unknown" { result.vendor = vendor.into(); }
                result.renderer = renderer;
                result.driver_version = version;
                result.vulkan = true;
            }
        }
    }
    if let Ok(output) = Command::new("timeout").args(["10s", "eglinfo", "-B"]).env("LC_ALL", "C").output() {
        if let Some(renderer) = opengl_renderer(&String::from_utf8_lossy(&output.stdout)) {
            result.opengl = true;
            result.opengl_renderer = Some(renderer.clone());
            if !is_software_renderer(&renderer) || !result.vulkan {
                result.renderer = renderer;
            }
        }
    }
    if result.vendor == "unknown" {
        if let Ok(output) = Command::new("lspci").arg("-mm").output() {
            for line in String::from_utf8_lossy(&output.stdout).lines() {
                if ["VGA compatible controller", "3D controller", "Display controller"].iter().any(|class| line.contains(class)) {
                    result.vendor = vendor_from_text(line).into();
                    if !result.vulkan && !result.opengl { result.renderer = line.to_string(); }
                    break;
                }
            }
        }
    }
    let opengl_accelerated = result.opengl_renderer.as_deref().is_some_and(|renderer| !is_software_renderer(renderer));
    result.backend = select_backend(result.vulkan, opengl_accelerated).into();
    result.accelerated = result.backend != "software";
    let renderer_vendor = vendor_from_text(&result.renderer);
    if renderer_vendor != "unknown" { result.vendor = renderer_vendor.into(); }
    result.gpu_kind = if !result.accelerated { "software" } else if renderer_vendor == "virtual" || result.vendor == "virtual" { "virtual" } else { "physical" }.into();
    result.virtual_machine = Command::new("systemd-detect-virt").args(["--vm", "--quiet"]).status().map(|status| status.success()).unwrap_or(false);
    result.gamescope_ready = gamescope_supported(result.vulkan, result.gamescope, result.drm);
    result.compositor = select_session(&std::env::var("EMUBOX_COMPOSITOR_PREFERENCE").unwrap_or_else(|_| "auto".into()),
        &result.backend, result.drm, result.cage, result.gamescope_ready).into();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn graphics_matrix() {
        for (text, vendor) in [("AMD Radeon", "amd"), ("Intel i915", "intel"), ("NVIDIA", "nvidia"),
            ("Mali panfrost", "mali"), ("v3d", "broadcom"), ("Adreno", "qualcomm"),
            ("Apple AGX", "apple"), ("VGA compatible controller", "unknown"), ("VMware", "virtual")] {
            assert_eq!(vendor_from_text(text), vendor);
        }
        let software = "GPU0:\n deviceType = PHYSICAL_DEVICE_TYPE_CPU\n deviceName = llvmpipe\n";
        assert!(hardware_vulkan(software).is_none());
        let mixed = format!("{software}GPU1:\n deviceType = PHYSICAL_DEVICE_TYPE_INTEGRATED_GPU\n deviceName = Mali\n driverInfo = Mesa\n");
        assert_eq!(hardware_vulkan(&mixed).unwrap().0, "Mali");
    }

    #[test]
    fn opengl_virtual_acceleration_is_not_software() {
        let svga = "SVGA3D; build: RELEASE; LLVM;";
        let mixed = format!("OpenGL core profile renderer: llvmpipe (LLVM 22)\nOpenGL ES profile renderer: {svga}\n");
        assert_eq!(opengl_renderer(&mixed).as_deref(), Some(svga));
        assert!(!is_software_renderer(svga));
        assert!(is_software_renderer("llvmpipe (LLVM 22)"));
        assert!(is_software_renderer("softpipe"));
        assert!(opengl_renderer("eglinfo: eglInitialize failed").is_none());
        assert_eq!(opengl_renderer("OpenGL renderer string: Mali-G610").as_deref(), Some("Mali-G610"));
        assert_eq!(select_session("auto", "opengl", true, true, false), "cage");
    }

    #[test]
    fn backend_and_session_are_independent() {
        for (vulkan, opengl, backend) in [(true, true, "opengl"), (false, true, "opengl"), (true, false, "vulkan"), (false, false, "software")] {
            assert_eq!(select_backend(vulkan, opengl), backend);
            assert_eq!(select_session("auto", backend, true, true, vulkan), if backend == "vulkan" { "gamescope" } else { "cage" });
        }
        assert_eq!(select_session("gamescope", "opengl", true, true, false), "cage");
        assert_eq!(select_session("gamescope", "opengl", true, true, true), "gamescope");
        assert_eq!(select_session("auto", "vulkan", true, true, false), "unavailable");
        assert_eq!(select_session("auto", "software", true, false, true), "unavailable");
        assert_eq!(vendor_from_text("aarch64 ARM CPU"), "unknown");
        for renderer in ["virgl", "SVGA3D", "VirtualBox 3D", "venus"] {
            assert_eq!(vendor_from_text(renderer), "virtual");
            assert!(!is_software_renderer(renderer));
        }
        assert!(hardware_vulkan("GPU0:\n deviceType = PHYSICAL_DEVICE_TYPE_VIRTUAL_GPU\n deviceName = SwiftShader").is_none());
        assert!(!gamescope_supported(false, true, true));
        assert!(!gamescope_supported(true, false, true));
        assert!(!gamescope_supported(true, true, false));
        assert!(gamescope_supported(true, true, true));
    }

    #[test]
    #[ignore = "Read-only host probe; requires graphics tools and device access"]
    fn host_graphics_probe() {
        let graphics = detect();
        eprintln!("accelerated={} backend={} gpuKind={} vm={} renderer={} vulkan={} compositor={}",
            graphics.accelerated, graphics.backend, graphics.gpu_kind, graphics.virtual_machine,
            graphics.renderer, graphics.vulkan, graphics.compositor);
        assert_eq!(graphics.accelerated, graphics.backend != "software");
        assert_eq!(graphics.backend, select_backend(graphics.vulkan,
            graphics.opengl_renderer.as_deref().is_some_and(|renderer| !is_software_renderer(renderer))));
    }
}
use crate::models::graphics::{
    DetectionState, GraphicsCapabilities, GraphicsDevice, GraphicsProbe,
};
pub use crate::services::graphics_policy::{
    gamescope_supported, hardware_vulkan, is_software_renderer, opengl_renderer, select_session,
    vendor_from_text,
};
use std::{fs, path::Path, process::Command};

pub fn detect() -> GraphicsCapabilities {
    let inventory = crate::services::graphics_probe::inventory();
    let inventory_complete = inventory.is_ok();
    let inventory_reason = inventory.as_ref().err().map(|error| match error.kind() {
        std::io::ErrorKind::PermissionDenied => "permission_denied".into(),
        _ => "inventory_unavailable".into(),
    });
    let mut result = summarize(
        inventory.unwrap_or_default(),
        inventory_complete,
        crate::services::graphics_probe::run("opengl", "eglinfo", &["-B"]),
        crate::services::graphics_probe::run("vulkan", "vulkaninfo", &[]),
    );
    result.inventory_reason = inventory_reason;
    result.device = fs::read_to_string("/sys/firmware/devicetree/base/model")
        .or_else(|_| fs::read_to_string("/sys/class/dmi/id/product_name"))
        .unwrap_or_else(|_| "unknown".into())
        .trim_matches(['\0', '\n'])
        .into();
    result.gamescope = crate::services::binary_service::resolve_executable("gamescope").is_some();
    result.cage = crate::services::binary_service::resolve_executable("cage").is_some();
    result.drm = result
        .devices
        .iter()
        .filter(|device| {
            result
                .selected_device_id
                .as_ref()
                .is_none_or(|id| &device.id == id)
        })
        .any(|device| {
            device
                .card_nodes
                .iter()
                .any(|node| Path::new(node).exists())
        });
    result.virtual_machine = Command::new("systemd-detect-virt")
        .args(["--vm", "--quiet"])
        .status()
        .map(|status| status.success())
        .unwrap_or(false);
    result.gamescope_ready = gamescope_for_selection(&result, result.drm);
    result.operational_backend = crate::services::graphics_policy::effective_backend(
        &result.backend,
        result.detection_state,
        &std::env::var("EMUBOX_RENDER_MODE").unwrap_or_default(),
    );
    if result.operational_backend == "software"
        && result.detection_state == DetectionState::Indeterminate
    {
        result.fallback_reason = Some("explicit_software_fallback".into());
    }
    result.compositor = select_session(
        &std::env::var("EMUBOX_COMPOSITOR_PREFERENCE").unwrap_or_else(|_| "auto".into()),
        &result.operational_backend,
        result.drm,
        result.cage,
        result.gamescope_ready,
    )
    .into();
    result
}

pub fn gamescope_for_selection(graphics: &GraphicsCapabilities, display_available: bool) -> bool {
    let selected_vulkan = graphics
        .probes
        .iter()
        .find(|probe| probe.api == "vulkan")
        .is_some_and(|probe| {
            probe.observations.iter().any(|observation| {
                observation.state == DetectionState::Accelerated
                    && observation.device_id.is_some()
                    && observation.device_id == graphics.selected_device_id
            })
        });
    gamescope_supported(selected_vulkan, graphics.gamescope, display_available)
}

pub fn summarize(
    devices: Vec<GraphicsDevice>,
    inventory_complete: bool,
    mut opengl: GraphicsProbe,
    mut vulkan: GraphicsProbe,
) -> GraphicsCapabilities {
    crate::services::graphics_probe::correlate(
        &mut opengl.observations,
        &devices,
        inventory_complete,
    );
    crate::services::graphics_probe::correlate(
        &mut vulkan.observations,
        &devices,
        inventory_complete,
    );
    let mut state = crate::services::graphics_policy::detection_state(opengl.state, vulkan.state);
    let mut backend =
        crate::services::graphics_policy::operational_backend(opengl.state, vulkan.state)
            .to_string();
    if state == DetectionState::Software && (!inventory_complete || devices.len() > 1) {
        state = DetectionState::Indeterminate;
        backend = "auto".into();
    }
    let preferred = if backend == "vulkan" {
        &vulkan
    } else {
        &opengl
    };
    let observations: Vec<_> = preferred
        .observations
        .iter()
        .filter(|observation| observation.state == DetectionState::Accelerated)
        .collect();
    let candidate = observations
        .first()
        .and_then(|observation| observation.device_id.clone());
    let selected = candidate.filter(|id| {
        observations
            .iter()
            .all(|observation| observation.device_id.as_ref() == Some(id))
    });
    let renderer = observations
        .first()
        .map(|observation| observation.renderer.clone())
        .or_else(|| {
            preferred
                .observations
                .first()
                .map(|observation| observation.renderer.clone())
        })
        .unwrap_or_else(|| "Unknown GPU".into());
    let vendor = selected
        .as_ref()
        .and_then(|id| devices.iter().find(|device| &device.id == id))
        .map(|device| device.vendor.clone())
        .unwrap_or_else(|| vendor_from_text(&renderer).into());
    let kind = match state {
        DetectionState::Software => "software",
        DetectionState::Indeterminate => "unknown",
        DetectionState::Accelerated if vendor == "virtual" => "virtual",
        DetectionState::Accelerated if selected.is_some() && vendor != "unknown" => "physical",
        _ => "unknown",
    };
    GraphicsCapabilities {
        detection_state: state,
        inventory_complete,
        selected_device_id: selected,
        active_device_id: None,
        accelerated: state == DetectionState::Accelerated,
        vendor,
        renderer,
        gpu_kind: kind.into(),
        opengl: opengl.state != DetectionState::Indeterminate,
        opengl_renderer: opengl
            .observations
            .iter()
            .find(|observation| observation.state == DetectionState::Accelerated)
            .or_else(|| opengl.observations.first())
            .map(|observation| observation.renderer.clone()),
        vulkan: vulkan.state == DetectionState::Accelerated,
        operational_backend: backend.clone(),
        backend,
        devices,
        probes: vec![opengl, vulkan],
        ..Default::default()
    }
}

pub fn session_environment() -> String {
    let graphics = detect();
    let selected = graphics
        .selected_device_id
        .as_ref()
        .and_then(|id| graphics.devices.iter().find(|device| &device.id == id));
    let status = |value: bool| if value { "1" } else { "0" }.to_string();
    let state = match graphics.detection_state {
        DetectionState::Accelerated => "accelerated",
        DetectionState::Software => "software",
        DetectionState::Indeterminate => "indeterminate",
    };
    let reason = format!(
        "inventory:{};{}",
        graphics.inventory_reason.as_deref().unwrap_or("complete"),
        graphics
            .probes
            .iter()
            .map(|probe| format!("{}:{}", probe.api, probe.reason))
            .collect::<Vec<_>>()
            .join(";")
    );
    let opengl_accelerated = graphics
        .probes
        .iter()
        .any(|probe| probe.api == "opengl" && probe.state == DetectionState::Accelerated);
    [
        ("GRAPHICS_DETECTION_STATE", state.into()),
        ("GRAPHICS_PROBE_REASONS", reason),
        ("GRAPHICS_BACKEND", graphics.backend.clone()),
        (
            "GRAPHICS_OPERATIONAL_BACKEND",
            graphics.operational_backend.clone(),
        ),
        (
            "GRAPHICS_FALLBACK_REASON",
            graphics.fallback_reason.clone().unwrap_or_default(),
        ),
        ("GPU_VENDOR", graphics.vendor.clone()),
        (
            "GPU_DRIVER",
            selected
                .map(|device| device.driver.clone())
                .unwrap_or_else(|| "unknown".into()),
        ),
        (
            "GPU_DEVICE",
            graphics
                .selected_device_id
                .clone()
                .unwrap_or_else(|| "unknown".into()),
        ),
        ("GPU_ACTIVE_DEVICE", "unknown".into()),
        ("GPU_KIND", graphics.gpu_kind.clone()),
        ("RENDERER_DESC", graphics.renderer.clone()),
        ("DEVICE_MODEL", graphics.device.clone()),
        ("HAS_HW_VULKAN", status(graphics.vulkan)),
        ("HAS_OPENGL", status(graphics.opengl)),
        ("HAS_HW_OPENGL", status(opengl_accelerated)),
        (
            "OPENGL_RENDERER",
            graphics
                .opengl_renderer
                .clone()
                .unwrap_or_else(|| "unknown".into()),
        ),
        ("HAS_ACCELERATION", status(graphics.accelerated)),
        ("HAS_DRM", status(graphics.drm)),
        ("HAS_CAGE", status(graphics.cage)),
        ("HAS_GAMESCOPE", status(graphics.gamescope)),
        ("GAMESCOPE_READY", status(graphics.gamescope_ready)),
        ("IS_VIRTUAL_MACHINE", status(graphics.virtual_machine)),
        ("EMUBOX_COMPOSITOR", graphics.compositor.clone()),
    ]
    .into_iter()
    .map(|(key, value)| format!("{key}={}\n", value.replace(['\n', '\r'], " ")))
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn graphics_matrix() {
        for (text, vendor) in [
            ("AMD Radeon", "amd"),
            ("Intel i915", "intel"),
            ("NVIDIA", "nvidia"),
            ("Mali panfrost", "mali"),
            ("v3d", "broadcom"),
            ("Adreno", "qualcomm"),
            ("Apple AGX", "apple"),
            ("VGA compatible controller", "unknown"),
            ("VMware", "virtual"),
        ] {
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
        assert_eq!(
            opengl_renderer("OpenGL renderer string: Mali-G610").as_deref(),
            Some("Mali-G610")
        );
        assert_eq!(select_session("auto", "opengl", true, true, false), "cage");
    }

    #[test]
    fn backend_and_session_are_independent() {
        for (vulkan, opengl, backend) in [
            (true, true, "opengl"),
            (false, true, "opengl"),
            (true, false, "vulkan"),
            (false, false, "software"),
        ] {
            assert_eq!(
                crate::services::graphics_policy::operational_backend(
                    if opengl {
                        DetectionState::Accelerated
                    } else {
                        DetectionState::Software
                    },
                    if vulkan {
                        DetectionState::Accelerated
                    } else {
                        DetectionState::Software
                    }
                ),
                backend
            );
            assert_eq!(
                select_session("auto", backend, true, true, vulkan),
                if backend == "vulkan" {
                    "gamescope"
                } else {
                    "cage"
                }
            );
        }
        assert_eq!(
            select_session("gamescope", "opengl", true, true, false),
            "cage"
        );
        assert_eq!(
            select_session("gamescope", "opengl", true, true, true),
            "gamescope"
        );
        assert_eq!(
            select_session("auto", "vulkan", true, true, false),
            "unavailable"
        );
        assert_eq!(
            select_session("auto", "software", true, false, true),
            "unavailable"
        );
        assert_eq!(vendor_from_text("aarch64 ARM CPU"), "unknown");
        for renderer in ["virgl", "SVGA3D", "VirtualBox 3D", "venus"] {
            assert_eq!(vendor_from_text(renderer), "virtual");
            assert!(!is_software_renderer(renderer));
        }
        assert!(hardware_vulkan(
            "GPU0:\n deviceType = PHYSICAL_DEVICE_TYPE_VIRTUAL_GPU\n deviceName = SwiftShader"
        )
        .is_none());
        assert!(!gamescope_supported(false, true, true));
        assert!(!gamescope_supported(true, false, true));
        assert!(!gamescope_supported(true, true, false));
        assert!(gamescope_supported(true, true, true));
    }

    #[test]
    fn different_gpus_never_merge_compositor_capabilities() {
        let devices = ["128", "129"]
            .map(|number| GraphicsDevice {
                id: format!("gpu-{number}"),
                card_nodes: vec![],
                render_nodes: vec![format!("/dev/dri/renderD{number}")],
                render_identifiers: vec![format!("drm:226:{number}")],
                pci_address: None,
                driver: String::new(),
                vendor: "unknown".into(),
            })
            .to_vec();
        let opengl = crate::services::graphics_probe::classify(
            "opengl",
            Some(0),
            "EGL DRM render node: /dev/dri/renderD128\nOpenGL renderer string: GPU OpenGL",
            "",
        );
        let vulkan = crate::services::graphics_probe::classify("vulkan", Some(0), "GPU0:\n deviceType = PHYSICAL_DEVICE_TYPE_DISCRETE_GPU\n deviceName = GPU Vulkan\n drmHasRender = true\n renderMajor = 226\n renderMinor = 129", "");
        let mut graphics = summarize(devices.clone(), true, opengl, vulkan);
        graphics.gamescope = true;
        assert_eq!(graphics.backend, "opengl");
        assert_eq!(graphics.selected_device_id.as_deref(), Some("gpu-128"));
        assert!(graphics.vulkan);
        assert!(!gamescope_for_selection(&graphics, true));
        assert_eq!(graphics.active_device_id, None);
        let uncorrelated = crate::services::graphics_probe::classify(
            "opengl",
            Some(0),
            "OpenGL renderer string: GPU",
            "",
        );
        let unavailable = crate::services::graphics_probe::classify(
            "vulkan",
            Some(1),
            "",
            "initialization failed",
        );
        let unknown_device = summarize(devices, true, uncorrelated, unavailable);
        assert!(unknown_device.accelerated);
        assert_eq!(unknown_device.backend, "opengl");
        assert_eq!(unknown_device.selected_device_id, None);
        assert_eq!(
            select_session("auto", &unknown_device.backend, true, true, false),
            "cage"
        );
    }

    #[test]
    fn software_probe_is_not_complete_hardware_coverage() {
        let opengl = crate::services::graphics_probe::classify(
            "opengl",
            Some(0),
            "OpenGL renderer string: llvmpipe",
            "",
        );
        let vulkan = crate::services::graphics_probe::classify(
            "vulkan",
            Some(0),
            "GPU0:\n deviceType = PHYSICAL_DEVICE_TYPE_CPU\n deviceName = llvmpipe",
            "",
        );
        let inconclusive = summarize(vec![], false, opengl.clone(), vulkan.clone());
        assert_eq!(inconclusive.detection_state, DetectionState::Indeterminate);
        assert_eq!(inconclusive.backend, "auto");
        let software = summarize(vec![], true, opengl, vulkan);
        assert_eq!(software.detection_state, DetectionState::Software);
        assert_eq!(software.backend, "software");
    }

    #[test]
    #[ignore = "Read-only host probe; requires graphics tools and device access"]
    fn host_graphics_probe() {
        let graphics = detect();
        eprintln!(
            "accelerated={} backend={} gpuKind={} vm={} renderer={} vulkan={} compositor={}",
            graphics.accelerated,
            graphics.backend,
            graphics.gpu_kind,
            graphics.virtual_machine,
            graphics.renderer,
            graphics.vulkan,
            graphics.compositor
        );
        assert_eq!(
            graphics.accelerated,
            graphics.detection_state == DetectionState::Accelerated
        );
        assert_eq!(
            graphics.backend,
            crate::services::graphics_policy::operational_backend(
                graphics.probes[0].state,
                graphics.probes[1].state
            )
        );
    }
}

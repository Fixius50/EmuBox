use crate::models::graphics::{DetectionState, GraphicsDevice, GraphicsObservation, GraphicsProbe};
use crate::services::graphics_policy::{is_software_renderer, vendor_from_text};
use std::{collections::BTreeMap, fs, process::Command};

pub fn inventory() -> Result<Vec<GraphicsDevice>, std::io::Error> {
    let mut devices = BTreeMap::<String, GraphicsDevice>::new();
    for entry in fs::read_dir("/sys/class/drm")? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        let card = name.strip_prefix("card").is_some_and(|suffix| {
            !suffix.is_empty() && suffix.chars().all(|character| character.is_ascii_digit())
        });
        let render = name.strip_prefix("renderD").is_some_and(|suffix| {
            !suffix.is_empty() && suffix.chars().all(|character| character.is_ascii_digit())
        });
        if !card && !render {
            continue;
        }
        let device_path = fs::canonicalize(entry.path().join("device"))?;
        let id = device_path.to_string_lossy().to_string();
        let device = devices.entry(id.clone()).or_insert_with(|| {
            let driver = fs::read_link(device_path.join("driver"))
                .ok()
                .and_then(|path| {
                    path.file_name()
                        .map(|name| name.to_string_lossy().to_string())
                })
                .unwrap_or_default();
            let uevent = fs::read_to_string(device_path.join("uevent")).unwrap_or_default();
            let pci_address = uevent
                .lines()
                .find_map(|line| line.strip_prefix("PCI_SLOT_NAME=").map(str::to_string));
            GraphicsDevice {
                id,
                card_nodes: vec![],
                render_nodes: vec![],
                render_identifiers: vec![],
                pci_address,
                vendor: vendor_from_text(&format!("{driver} {uevent}")).into(),
                driver,
            }
        });
        let node = format!("/dev/dri/{name}");
        if card {
            device.card_nodes.push(node);
        } else {
            device.render_nodes.push(node);
            if let Ok(number) = fs::read_to_string(entry.path().join("dev")) {
                device
                    .render_identifiers
                    .push(format!("drm:{}", number.trim()));
            }
        }
    }
    for device in devices.values_mut() {
        device.card_nodes.sort();
        device.render_nodes.sort();
    }
    Ok(devices.into_values().collect())
}

pub fn correlate(
    observations: &mut [GraphicsObservation],
    devices: &[GraphicsDevice],
    inventory_complete: bool,
) {
    let accelerated_names: std::collections::BTreeSet<_> = observations
        .iter()
        .filter(|observation| observation.state == DetectionState::Accelerated)
        .map(|observation| observation.renderer.clone())
        .collect();
    for observation in observations {
        if observation.state != DetectionState::Accelerated {
            continue;
        }
        if let Some(reported) = &observation.reported_device {
            let matches: Vec<_> = devices
                .iter()
                .filter(|device| {
                    device.render_nodes.contains(reported)
                        || device.card_nodes.contains(reported)
                        || device.render_identifiers.contains(reported)
                        || &device.id == reported
                })
                .collect();
            if let [device] = matches.as_slice() {
                observation.device_id = Some(device.id.clone());
                observation.correlation = "device_identifier".into();
            }
        } else if inventory_complete && devices.len() == 1 && accelerated_names.len() == 1 {
            observation.device_id = Some(devices[0].id.clone());
            observation.correlation = "single_device_inference".into();
        }
    }
}

fn observation(
    api: &str,
    renderer: String,
    state: DetectionState,
    reported_device: Option<String>,
) -> GraphicsObservation {
    GraphicsObservation {
        api: api.into(),
        state,
        renderer,
        device_id: None,
        correlation: "unknown".into(),
        reported_device,
    }
}

pub fn parse_opengl(text: &str) -> Vec<GraphicsObservation> {
    let mut observations = Vec::new();
    let mut reported = None;
    for line in text.lines().map(str::trim) {
        if line.ends_with("platform:") || line.starts_with("Device #") {
            reported = None;
        }
        if let Some((key, value)) = line.split_once(':') {
            if matches!(
                key,
                "EGL DRM render node" | "EGL DRM device" | "DRM render node" | "DRM device"
            ) && value.trim().starts_with("/dev/dri/")
            {
                reported = Some(value.trim().to_string());
            }
            if matches!(
                key,
                "OpenGL core profile renderer"
                    | "OpenGL compatibility profile renderer"
                    | "OpenGL ES profile renderer"
                    | "OpenGL renderer string"
            ) && !value.trim().is_empty()
            {
                let renderer = value.trim().to_string();
                let state = if is_software_renderer(&renderer) {
                    DetectionState::Software
                } else {
                    DetectionState::Accelerated
                };
                observations.push(observation("opengl", renderer, state, reported.clone()));
            }
        }
    }
    observations
}

pub fn parse_vulkan(text: &str) -> Vec<GraphicsObservation> {
    let mut observations = Vec::new();
    let mut name = String::new();
    let mut device_type = String::new();
    let mut major = None;
    let mut minor = None;
    let mut has_render = false;
    for line in text
        .lines()
        .map(str::trim)
        .chain(std::iter::once("GPU999:"))
    {
        if line.starts_with("GPU") && line.ends_with(':') {
            if !name.is_empty() {
                let state = if is_software_renderer(&name) || device_type.contains("TYPE_CPU") {
                    DetectionState::Software
                } else if ["DISCRETE_GPU", "INTEGRATED_GPU", "VIRTUAL_GPU"]
                    .iter()
                    .any(|kind| device_type.contains(kind))
                {
                    DetectionState::Accelerated
                } else {
                    DetectionState::Indeterminate
                };
                let reported = if has_render {
                    Some(
                        major
                            .zip(minor)
                            .map(|(major, minor)| format!("drm:{major}:{minor}"))
                            .unwrap_or_else(|| "drm:unresolved".into()),
                    )
                } else {
                    None
                };
                observations.push(observation(
                    "vulkan",
                    std::mem::take(&mut name),
                    state,
                    reported,
                ));
            }
            device_type.clear();
            major = None;
            minor = None;
            has_render = false;
        }
        if let Some((key, value)) = line.split_once('=') {
            match key.trim() {
                "deviceName" => name = value.trim().into(),
                "deviceType" => device_type = value.trim().into(),
                "drmHasRender" => has_render = value.trim() == "true",
                "renderMajor" => major = value.trim().parse::<u32>().ok(),
                "renderMinor" => minor = value.trim().parse::<u32>().ok(),
                _ => {}
            }
        }
    }
    observations
}

pub fn classify(api: &str, exit_code: Option<i32>, stdout: &str, stderr: &str) -> GraphicsProbe {
    let mut observations = if api == "opengl" {
        parse_opengl(stdout)
    } else {
        parse_vulkan(stdout)
    };
    let diagnostic = stderr.to_lowercase();
    let complete = exit_code == Some(0);
    let accelerated = observations
        .iter()
        .any(|observation| observation.state == DetectionState::Accelerated);
    let software_only = !observations.is_empty()
        && observations
            .iter()
            .all(|observation| observation.state == DetectionState::Software);
    let (state, reason) = if exit_code == Some(124) || exit_code == Some(137) {
        (DetectionState::Indeterminate, "timeout")
    } else if api == "vulkan" && !complete {
        (
            DetectionState::Indeterminate,
            if diagnostic.contains("permission denied") {
                "permission_denied"
            } else {
                "initialization_failed"
            },
        )
    } else if accelerated {
        (
            DetectionState::Accelerated,
            if complete {
                "renderer_observed"
            } else {
                "renderer_observed_partial_probe"
            },
        )
    } else if complete && software_only {
        (DetectionState::Software, "software_renderers_observed")
    } else {
        (
            DetectionState::Indeterminate,
            if diagnostic.contains("permission denied") {
                "permission_denied"
            } else if complete {
                "no_renderer_reported"
            } else {
                "initialization_failed"
            },
        )
    };
    if state == DetectionState::Indeterminate {
        observations
            .iter_mut()
            .for_each(|observation| observation.state = DetectionState::Indeterminate);
    }
    GraphicsProbe {
        api: api.into(),
        state,
        reason: reason.into(),
        exit_code,
        observations,
    }
}

pub fn run(api: &str, program: &str, arguments: &[&str]) -> GraphicsProbe {
    let failure = |reason: &str| GraphicsProbe {
        api: api.into(),
        state: DetectionState::Indeterminate,
        reason: reason.into(),
        exit_code: None,
        observations: vec![],
    };
    if crate::services::binary_service::resolve_executable(program).is_none() {
        return failure("tool_missing");
    }
    let mut command = Command::new("timeout");
    command
        .arg("--kill-after=1s")
        .arg("10s")
        .arg(program)
        .args(arguments)
        .env("LC_ALL", "C")
        .env_remove("LIBGL_ALWAYS_SOFTWARE");
    for key in ["GALLIUM_DRIVER", "MESA_LOADER_DRIVER_OVERRIDE"] {
        if std::env::var(key).is_ok_and(|value| is_software_renderer(&value)) {
            command.env_remove(key);
        }
    }
    match command.output() {
        Ok(output) => classify(
            api,
            output.status.code(),
            &String::from_utf8_lossy(&output.stdout),
            &String::from_utf8_lossy(&output.stderr),
        ),
        Err(error) => failure(if error.kind() == std::io::ErrorKind::PermissionDenied {
            "permission_denied"
        } else {
            "probe_unavailable"
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn device(id: &str, node: &str) -> GraphicsDevice {
        GraphicsDevice {
            id: id.into(),
            card_nodes: vec![],
            render_nodes: vec![node.into()],
            render_identifiers: vec![],
            pci_address: None,
            driver: String::new(),
            vendor: "unknown".into(),
        }
    }
    #[test]
    fn multigpu_never_correlates_by_renderer_name() {
        let devices = vec![
            device("intel", "/dev/dri/renderD128"),
            device("nvidia", "/dev/dri/renderD129"),
        ];
        let mut observations = parse_opengl("OpenGL renderer string: Intel\nEGL DRM render node: /dev/dri/renderD129\nOpenGL renderer string: NVIDIA");
        correlate(&mut observations, &devices, true);
        assert_eq!(observations[0].device_id, None);
        assert_eq!(observations[1].device_id.as_deref(), Some("nvidia"));
    }
    #[test]
    fn single_virtual_gpu_does_not_require_full_correlation() {
        let mut observations =
            parse_opengl("OpenGL renderer string: SVGA3D; LLVM;\nOpenGL renderer string: llvmpipe");
        correlate(
            &mut observations,
            &[device("virtual", "/dev/dri/renderD128")],
            true,
        );
        assert_eq!(observations[0].device_id.as_deref(), Some("virtual"));
        assert_eq!(observations[0].correlation, "single_device_inference");
        assert_eq!(observations[1].device_id, None);
    }
    #[test]
    fn failures_and_partial_platform_results_are_distinct() {
        assert_eq!(
            classify("vulkan", Some(1), "", "Permission denied").reason,
            "permission_denied"
        );
        assert_eq!(
            classify("opengl", Some(124), "OpenGL renderer string: SVGA3D", "").state,
            DetectionState::Indeterminate
        );
        assert_eq!(
            classify(
                "opengl",
                Some(1),
                "OpenGL renderer string: SVGA3D",
                "X11 failed"
            )
            .state,
            DetectionState::Accelerated
        );
        assert_eq!(
            classify("opengl", Some(0), "OpenGL renderer string: llvmpipe", "").state,
            DetectionState::Software
        );
        assert_eq!(
            classify("opengl", Some(0), "", "").state,
            DetectionState::Indeterminate
        );
    }

    #[test]
    fn vulkan_drm_numbers_correlate_without_names_or_host_io() {
        let mut gpu = device("nvidia", "/dev/dri/renderD129");
        gpu.render_identifiers.push("drm:226:129".into());
        let text = "GPU0:\n deviceType = PHYSICAL_DEVICE_TYPE_DISCRETE_GPU\n deviceName = Same Name\n drmHasRender = true\n renderMajor = 226\n renderMinor = 129\nGPU1:\n deviceType = PHYSICAL_DEVICE_TYPE_DISCRETE_GPU\n deviceName = Same Name\n drmHasRender = true\n renderMajor = 226\n renderMinor = 130";
        let mut observations = parse_vulkan(text);
        correlate(&mut observations, &[gpu], true);
        assert_eq!(observations.len(), 2);
        assert_eq!(observations[0].device_id.as_deref(), Some("nvidia"));
        assert_eq!(observations[1].device_id, None);
        assert_eq!(
            observations[1].reported_device.as_deref(),
            Some("drm:226:130")
        );
    }

    #[test]
    fn failed_inventory_cannot_enable_single_device_inference() {
        let mut observations = parse_opengl("OpenGL renderer string: SVGA3D");
        correlate(
            &mut observations,
            &[device("virtual", "/dev/dri/renderD128")],
            false,
        );
        assert_eq!(observations[0].device_id, None);
        assert_eq!(observations[0].state, DetectionState::Accelerated);
    }

    #[test]
    fn software_and_unrecognized_vulkan_devices_are_not_accelerated() {
        for renderer in ["llvmpipe", "softpipe", "lavapipe", "SwiftShader"] {
            let text = format!(
                "GPU0:\n deviceType = PHYSICAL_DEVICE_TYPE_VIRTUAL_GPU\n deviceName = {renderer}"
            );
            assert_eq!(
                classify("vulkan", Some(0), &text, "").state,
                DetectionState::Software
            );
        }
        assert_eq!(
            classify(
                "vulkan",
                Some(0),
                "GPU0:\n deviceType = OTHER\n deviceName = Unknown",
                ""
            )
            .state,
            DetectionState::Indeterminate
        );
    }
}

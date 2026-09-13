pub fn vendor_from_text(text: &str) -> &'static str {
    let lower = text.to_lowercase();
    let words: Vec<_> = lower
        .split(|character: char| !character.is_alphanumeric())
        .collect();
    let vendors: &[(&str, &[&str])] = &[
        (
            "virtual",
            &[
                "vmware",
                "vmwgfx",
                "virtualbox",
                "virtio",
                "qxl",
                "virgl",
                "svga3d",
                "venus",
            ],
        ),
        ("amd", &["amd", "ati", "radeon", "amdgpu", "1002"]),
        ("nvidia", &["nvidia", "nouveau", "10de"]),
        ("intel", &["intel", "i915", "xe", "8086"]),
        ("broadcom", &["broadcom", "v3d", "vc4", "14e4"]),
        ("mali", &["mali", "panfrost", "panthor", "lima", "13b5"]),
        (
            "qualcomm",
            &["qualcomm", "adreno", "msm", "freedreno", "5143"],
        ),
        ("apple", &["apple", "asahi", "agx", "106b"]),
    ];
    vendors
        .iter()
        .find(|(_, identifiers)| {
            identifiers
                .iter()
                .any(|identifier| words.contains(identifier))
        })
        .map(|(vendor, _)| *vendor)
        .unwrap_or("unknown")
}

pub fn is_software_renderer(renderer: &str) -> bool {
    let lower = renderer.to_lowercase();
    [
        "llvmpipe",
        "softpipe",
        "swrast",
        "software rasterizer",
        "swiftshader",
        "lavapipe",
        "microsoft basic render",
        "gdi generic",
        "mesa software",
    ]
    .iter()
    .any(|software| lower.contains(software))
}

pub fn hardware_vulkan(summary: &str) -> Option<(String, Option<String>)> {
    let mut name = None;
    let mut version = None;
    let mut hardware = false;
    for line in summary.lines().chain(std::iter::once("GPU999:")) {
        let line = line.trim();
        if line.starts_with("GPU") && line.ends_with(':') {
            if hardware
                && name
                    .as_deref()
                    .is_some_and(|renderer| !is_software_renderer(renderer))
            {
                return name.map(|name| (name, version));
            }
            name = None;
            version = None;
            hardware = false;
        }
        if let Some((key, value)) = line.split_once('=') {
            let value = value.trim();
            match key.trim() {
                "deviceType" => {
                    hardware = value.contains("DISCRETE_GPU")
                        || value.contains("INTEGRATED_GPU")
                        || value.contains("VIRTUAL_GPU")
                }
                "deviceName" => name = Some(value.to_string()),
                "driverInfo" => version = Some(value.to_string()),
                _ => {}
            }
        }
    }
    None
}

pub fn opengl_renderer(summary: &str) -> Option<String> {
    let renderers: Vec<_> = summary
        .lines()
        .filter_map(|line| {
            let (key, value) = line.trim().split_once(':')?;
            match key {
                "OpenGL core profile renderer"
                | "OpenGL compatibility profile renderer"
                | "OpenGL ES profile renderer"
                | "OpenGL renderer string"
                    if !value.trim().is_empty() =>
                {
                    Some(value.trim().to_string())
                }
                _ => None,
            }
        })
        .collect();
    renderers
        .iter()
        .find(|renderer| !is_software_renderer(renderer))
        .or_else(|| renderers.first())
        .cloned()
}

pub fn detection_state(
    opengl: crate::models::graphics::DetectionState,
    vulkan: crate::models::graphics::DetectionState,
) -> crate::models::graphics::DetectionState {
    use crate::models::graphics::DetectionState;
    match (opengl, vulkan) {
        (DetectionState::Accelerated, _) | (_, DetectionState::Accelerated) => {
            DetectionState::Accelerated
        }
        (DetectionState::Software, DetectionState::Software) => DetectionState::Software,
        _ => DetectionState::Indeterminate,
    }
}

pub fn operational_backend(
    opengl: crate::models::graphics::DetectionState,
    vulkan: crate::models::graphics::DetectionState,
) -> &'static str {
    use crate::models::graphics::DetectionState;
    match (opengl, vulkan) {
        (DetectionState::Accelerated, _) => "opengl",
        (_, DetectionState::Accelerated) => "vulkan",
        (DetectionState::Software, DetectionState::Software) => "software",
        _ => "auto",
    }
}

#[cfg(test)]
mod detection_tests {
    use super::*;
    use crate::models::graphics::DetectionState::{Accelerated, Indeterminate, Software};

    #[test]
    fn probe_failure_is_not_software() {
        assert_eq!(detection_state(Accelerated, Indeterminate), Accelerated);
        assert_eq!(operational_backend(Accelerated, Indeterminate), "opengl");
        assert_eq!(detection_state(Indeterminate, Indeterminate), Indeterminate);
        assert_eq!(operational_backend(Indeterminate, Indeterminate), "auto");
        assert_eq!(detection_state(Software, Indeterminate), Indeterminate);
        assert_eq!(operational_backend(Software, Software), "software");
        assert_eq!(operational_backend(Software, Accelerated), "vulkan");
        assert_eq!(operational_backend(Accelerated, Accelerated), "opengl");
        assert_eq!(effective_backend("auto", Indeterminate, "auto"), "auto");
        assert_eq!(
            effective_backend("auto", Indeterminate, "software"),
            "software"
        );
        assert_eq!(
            effective_backend("opengl", Accelerated, "software"),
            "opengl"
        );
        assert_eq!(select_session("auto", "auto", true, true, false), "cage");
    }
}

pub fn effective_backend(
    detected: &str,
    state: crate::models::graphics::DetectionState,
    request: &str,
) -> String {
    if request == "software" && state != crate::models::graphics::DetectionState::Accelerated {
        "software".into()
    } else {
        detected.into()
    }
}

pub fn gamescope_supported(vulkan: bool, executable: bool, display_available: bool) -> bool {
    vulkan && executable && display_available
}

pub fn select_session(
    preference: &str,
    backend: &str,
    drm: bool,
    cage: bool,
    gamescope_ready: bool,
) -> &'static str {
    match (drm, preference, backend, cage, gamescope_ready) {
        (false, _, _, _, _) => "unavailable",
        (true, "gamescope", backend, _, true) if backend != "software" => "gamescope",
        (true, _, "opengl" | "software" | "auto", true, _) => "cage",
        (true, _, backend, _, true) if backend != "software" => "gamescope",
        _ => "unavailable",
    }
}

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
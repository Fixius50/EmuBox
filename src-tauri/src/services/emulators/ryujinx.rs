use super::EmulatorProfile;

/// Ryujinx fue discontinuado oficialmente en marzo de 2024 por presión legal de
/// Nintendo; el repositorio y binarios oficiales ya no se distribuyen. Se mantiene
/// el perfil por compatibilidad con instalaciones manuales de forks de la comunidad,
/// pero no se instala ni se recomienda de forma automática.
pub struct Ryujinx;

impl EmulatorProfile for Ryujinx {
    fn id(&self) -> &'static str { "ryujinx" }
    fn official_name(&self) -> &'static str { "Ryujinx" }
    fn binary_candidates(&self) -> &'static [&'static str] { &["ryujinx", "Ryujinx.AppImage"] }
    fn supported_platforms(&self) -> &'static [&'static str] { &["switch"] }
    fn core_type(&self) -> &'static str { "standalone" }
    fn default_arguments(&self) -> &'static [&'static str] { &["--fullscreen"] }
    fn version_flag(&self) -> &'static str { "--version" }
}

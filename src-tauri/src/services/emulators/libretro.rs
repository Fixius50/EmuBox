use super::EmulatorProfile;

pub struct LibretroProfile {
    pub id: &'static str,
    pub name: &'static str,
    pub platforms: &'static [&'static str],
    pub arguments: &'static [&'static str],
}

impl EmulatorProfile for LibretroProfile {
    fn id(&self) -> &'static str {
        self.id
    }
    fn official_name(&self) -> &'static str {
        self.name
    }
    fn binary_candidates(&self) -> &'static [&'static str] {
        &["retroarch"]
    }
    fn supported_platforms(&self) -> &'static [&'static str] {
        self.platforms
    }
    fn core_type(&self) -> &'static str {
        "libretro"
    }
    fn default_arguments(&self) -> &'static [&'static str] {
        self.arguments
    }
    fn version_flag(&self) -> &'static str {
        "--version"
    }
}

pub fn profiles() -> Vec<Box<dyn EmulatorProfile>> {
    vec![
        Box::new(LibretroProfile {
            id: "dolphin-libretro",
            name: "Dolphin (Libretro)",
            platforms: &["gamecube", "wii"],
            arguments: &["-f", "-L", "dolphin_libretro.so"],
        }),
        Box::new(LibretroProfile {
            id: "flycast-libretro",
            name: "Flycast (Libretro)",
            platforms: &["dreamcast"],
            arguments: &["-f", "-L", "flycast_libretro.so"],
        }),
        Box::new(LibretroProfile {
            id: "melonds-libretro",
            name: "melonDS (Libretro)",
            platforms: &["nds"],
            arguments: &["-f", "-L", "melonds_libretro.so"],
        }),
        Box::new(LibretroProfile {
            id: "mgba-libretro",
            name: "mGBA (Libretro)",
            platforms: &["gba", "gb"],
            arguments: &["-f", "-L", "mgba_libretro.so"],
        }),
        Box::new(LibretroProfile {
            id: "ppsspp-libretro",
            name: "PPSSPP (Libretro)",
            platforms: &["psp"],
            arguments: &["-f", "-L", "ppsspp_libretro.so"],
        }),
        Box::new(LibretroProfile {
            id: "azahar-libretro",
            name: "Azahar (Libretro)",
            platforms: &["3ds"],
            arguments: &["-f", "-L", "/opt/emubox/bin/azahar_libretro.so"],
        }),
        Box::new(LibretroProfile {
            id: "snes9x",
            name: "Snes9x (Libretro)",
            platforms: &["snes"],
            arguments: &["-f", "-L", "snes9x_libretro.so"],
        }),
        Box::new(LibretroProfile {
            id: "mupen64plus",
            name: "Mupen64Plus Next (Libretro)",
            platforms: &["n64"],
            arguments: &["-f", "-L", "mupen64plus_next_libretro.so"],
        }),
        Box::new(LibretroProfile {
            id: "genesis-plus-gx",
            name: "Genesis Plus GX (Libretro)",
            platforms: &["genesis"],
            arguments: &["-f", "-L", "genesis_plus_gx_libretro.so"],
        }),
        Box::new(LibretroProfile {
            id: "fbneo",
            name: "FinalBurn Neo (Libretro)",
            platforms: &["arcade"],
            arguments: &["-f", "-L", "fbneo_libretro.so"],
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn alternatives_always_name_their_core() {
        for profile in profiles() {
            assert!(profile
                .default_arguments()
                .windows(2)
                .any(|args| args[0] == "-L" && args[1].ends_with("_libretro.so")));
            assert!(!profile.supported_platforms().is_empty());
        }
    }
}

use super::DownloadService;

impl DownloadService {
    pub fn infer_platform(
        item_platform: Option<&str>,
        manifest_platform: Option<&str>,
        title: &str,
        uris: &[String],
        manifest_hint: Option<&str>,
    ) -> String {
        if let Some(p) = item_platform {
            let p_lower = p.to_ascii_lowercase();
            if Self::supported_platform(&p_lower) {
                return p_lower;
            }
        }
        if let Some(p) = manifest_platform {
            let p_lower = p.to_ascii_lowercase();
            if Self::supported_platform(&p_lower) {
                return p_lower;
            }
        }

        let title_lower = title.to_ascii_lowercase();

        if title_lower.contains("[pc]")
            || title_lower.contains("(pc)")
            || title_lower.contains("steamrip")
            || title_lower.contains("gog")
        {
            return "pc".to_string();
        }
        if title_lower.contains("[ps3]")
            || title_lower.contains("(ps3)")
            || title_lower.contains("ps3")
            || title_lower.contains("rpcs3")
        {
            return "ps3".to_string();
        }
        if title_lower.contains("[ps2]")
            || title_lower.contains("(ps2)")
            || title_lower.contains("pcsx2")
        {
            return "ps2".to_string();
        }
        if title_lower.contains("[ps1]")
            || title_lower.contains("(ps1)")
            || title_lower.contains("[psx]")
            || title_lower.contains("(psx)")
            || title_lower.contains("duckstation")
        {
            return "ps1".to_string();
        }
        if title_lower.contains("[psp]")
            || title_lower.contains("(psp)")
            || title_lower.contains("ppsspp")
        {
            return "psp".to_string();
        }
        if title_lower.contains("[wiiu]")
            || title_lower.contains("(wiiu)")
            || title_lower.contains("wii u")
            || title_lower.contains("cemu")
        {
            return "wiiu".to_string();
        }
        if title_lower.contains("[wii]") || title_lower.contains("(wii)") {
            return "wii".to_string();
        }
        if title_lower.contains("[gamecube]")
            || title_lower.contains("(gamecube)")
            || title_lower.contains("[gcn]")
            || title_lower.contains("dolphin")
        {
            return "gamecube".to_string();
        }
        if title_lower.contains("[snes]")
            || title_lower.contains("(snes)")
            || title_lower.contains("super nintendo")
        {
            return "snes".to_string();
        }
        if title_lower.contains("[gba]")
            || title_lower.contains("(gba)")
            || title_lower.contains("game boy advance")
            || title_lower.contains("mgba")
        {
            return "gba".to_string();
        }
        if title_lower.contains("[n64]")
            || title_lower.contains("(n64)")
            || title_lower.contains("nintendo 64")
        {
            return "n64".to_string();
        }
        if title_lower.contains("[nds]")
            || title_lower.contains("(nds)")
            || title_lower.contains("nintendo ds")
            || title_lower.contains("melonds")
        {
            return "nds".to_string();
        }
        if title_lower.contains("[genesis]")
            || title_lower.contains("(genesis)")
            || title_lower.contains("megadrive")
            || title_lower.contains("mega drive")
        {
            return "genesis".to_string();
        }
        if title_lower.contains("[dreamcast]")
            || title_lower.contains("(dreamcast)")
            || title_lower.contains("flycast")
        {
            return "dreamcast".to_string();
        }
        if title_lower.contains("[arcade]")
            || title_lower.contains("(arcade)")
            || title_lower.contains("mame")
        {
            return "arcade".to_string();
        }

        for uri in uris {
            let u_lower = uri.to_ascii_lowercase();
            if u_lower.contains(".pkg") {
                return "ps3".to_string();
            }
            if u_lower.contains(".sfc") || u_lower.contains(".smc") {
                return "snes".to_string();
            }
            if u_lower.contains(".gba") {
                return "gba".to_string();
            }
            if u_lower.contains(".z64") || u_lower.contains(".n64") || u_lower.contains(".v64") {
                return "n64".to_string();
            }
            if u_lower.contains(".nds") {
                return "nds".to_string();
            }
            if u_lower.contains(".cdi") || u_lower.contains(".gdi") {
                return "dreamcast".to_string();
            }
            if u_lower.contains(".rvz") || u_lower.contains(".gcm") || u_lower.contains(".ciso") {
                return "gamecube".to_string();
            }
            if u_lower.contains(".wua") || u_lower.contains(".wux") || u_lower.contains(".rpx") {
                return "wiiu".to_string();
            }
            if u_lower.contains(".pbp") {
                return "psp".to_string();
            }
            if u_lower.contains("steamrip") || u_lower.contains("gog") || u_lower.contains(".exe") {
                return "pc".to_string();
            }
        }

        if let Some(hint) = manifest_hint {
            let h_lower = hint.to_ascii_lowercase();
            if h_lower.contains("psx-roms") || h_lower.contains("ps1") {
                return "ps1".to_string();
            }
            if h_lower.contains("ps2") {
                return "ps2".to_string();
            }
            if h_lower.contains("ps3") {
                return "ps3".to_string();
            }
            if h_lower.contains("psp") {
                return "psp".to_string();
            }
            if h_lower.contains("snes") {
                return "snes".to_string();
            }
            if h_lower.contains("gba") {
                return "gba".to_string();
            }
            if h_lower.contains("n64") {
                return "n64".to_string();
            }
            if h_lower.contains("nds") {
                return "nds".to_string();
            }
            if h_lower.contains("linux") || h_lower.contains("pc") || h_lower.contains("repack") {
                return "pc".to_string();
            }
            if h_lower.contains("psx") {
                return "ps1".to_string();
            }
        }

        "pc".to_string()
    }

    pub(super) fn supported_platform(value: &str) -> bool {
        matches!(
            value,
            "ps1"
                | "ps2"
                | "ps3"
                | "psp"
                | "gamecube"
                | "wii"
                | "wiiu"
                | "n64"
                | "snes"
                | "gba"
                | "nds"
                | "genesis"
                | "dreamcast"
                | "arcade"
                | "pc"
                | "linux"
                | "all"
        )
    }
}

use super::DownloadService;
use crate::services::library::platforms::PLATFORM_SPECS;

const PLATFORM_TERMS: &[(&str, &[&str])] = &[
    ("ps4", &["ps4", "playstation 4", "shadps4"]),
    ("3ds", &["3ds", "nintendo 3ds", "azahar", "citra"]),
    ("ps3", &["ps3", "playstation 3", "rpcs3"]),
    ("ps2", &["ps2", "playstation 2", "pcsx2"]),
    ("ps1", &["ps1", "psx", "playstation 1", "duckstation"]),
    ("psp", &["psp", "ppsspp"]),
    ("wiiu", &["wiiu", "wii u", "cemu"]),
    ("wii", &["wii"]),
    ("gamecube", &["gamecube", "gcn", "dolphin"]),
    ("snes", &["snes", "super nintendo"]),
    ("gba", &["gba", "game boy advance", "mgba"]),
    ("n64", &["n64", "nintendo 64"]),
    ("nds", &["nds", "nintendo ds", "melonds"]),
    ("genesis", &["genesis", "megadrive", "mega drive"]),
    ("dreamcast", &["dreamcast", "flycast"]),
    ("arcade", &["arcade", "mame"]),
    ("pc", &["pc", "steamrip", "gog", "repack", "linux"]),
];

fn text_platform(text: &str) -> Option<&'static str> {
    let words = format!(
        " {} ",
        text.to_lowercase()
            .split(|character: char| !character.is_alphanumeric())
            .filter(|word| !word.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    );
    let matches: Vec<_> = PLATFORM_TERMS
        .iter()
        .filter_map(|(platform, terms)| {
            terms
                .iter()
                .any(|term| words.contains(&format!(" {term} ")))
                .then_some(*platform)
        })
        .filter(|platform| !(*platform == "wii" && words.contains(" wii u ")))
        .collect();
    match matches.as_slice() {
        [platform] => Some(*platform),
        _ => None,
    }
}

fn title_platform(title: &str) -> Option<&'static str> {
    let tags: Vec<_> = title
        .split(['[', '('])
        .skip(1)
        .filter_map(|part| {
            part.split_once([']', ')'])
                .and_then(|(tag, _)| text_platform(tag))
        })
        .collect();
    if let Some(first) = tags.first() {
        return tags
            .iter()
            .all(|platform| platform == first)
            .then_some(*first);
    }
    text_platform(title)
}

fn uri_platform(uri: &str) -> Option<&'static str> {
    let url = reqwest::Url::parse(uri).ok()?;
    let filename = match url.scheme() {
        "http" | "https" => url.path_segments()?.next_back()?.to_string(),
        "magnet" => url
            .query_pairs()
            .find(|(key, _)| key == "dn")?
            .1
            .into_owned(),
        _ => return None,
    };
    let extension = std::path::Path::new(&filename)
        .extension()?
        .to_str()?
        .to_ascii_lowercase();
    match extension.as_str() {
        "sfc" | "smc" => Some("snes"),
        "gba" => Some("gba"),
        "z64" | "n64" | "v64" => Some("n64"),
        "nds" => Some("nds"),
        "cci" | "cxi" | "3dsx" | "3ds" => Some("3ds"),
        "gdi" | "cdi" => Some("dreamcast"),
        "gcm" => Some("gamecube"),
        "wua" | "wux" | "rpx" => Some("wiiu"),
        "exe" => Some("pc"),
        _ => None,
    }
}

impl DownloadService {
    pub fn infer_platform(
        item_platform: Option<&str>,
        manifest_platform: Option<&str>,
        title: &str,
        uris: &[String],
        manifest_hint: Option<&str>,
    ) -> String {
        for value in [item_platform, manifest_platform].into_iter().flatten() {
            let value = value.trim().to_ascii_lowercase();
            if Self::supported_platform(&value) {
                return value;
            }
        }
        if let Some(platform) =
            title_platform(title).or_else(|| manifest_hint.and_then(text_platform))
        {
            return platform.into();
        }
        let candidates: Vec<_> = uris.iter().filter_map(|uri| uri_platform(uri)).collect();
        if let Some(first) = candidates.first() {
            if candidates.iter().all(|platform| platform == first) {
                return (*first).into();
            }
        }
        "pc".into()
    }

    pub(super) fn supported_platform(value: &str) -> bool {
        PLATFORM_SPECS.iter().any(|platform| platform.id == value)
            || matches!(value, "wii" | "wiiu" | "linux")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declared_platforms_and_tags_take_precedence() {
        for platform in PLATFORM_SPECS {
            assert_eq!(
                DownloadService::infer_platform(Some(platform.id), Some("pc"), "[PS3]", &[], None),
                platform.id
            );
        }
        assert_eq!(
            DownloadService::infer_platform(
                Some(" PS4 "),
                None,
                "Game",
                &["https://example.test/game.pkg".into()],
                None
            ),
            "ps4"
        );
        assert_eq!(
            DownloadService::infer_platform(None, None, "GOG [3DS]", &[], None),
            "3ds"
        );
        assert_eq!(
            DownloadService::infer_platform(
                None,
                None,
                "Game",
                &["https://example.test/game.pkg".into()],
                Some("ps4-games")
            ),
            "ps4"
        );
        assert_eq!(text_platform("Nintendo Wii U"), Some("wiiu"));
        assert!(!DownloadService::supported_platform("all"));
    }

    #[test]
    fn urls_do_not_infer_from_hosts_queries_or_ambiguous_extensions() {
        for uri in [
            "https://games.sfc.example/download",
            "https://example.test/download?file=game.nds",
            "https://example.test/game.pkg",
            "https://example.test/game.pbp",
            "https://example.test/game.rvz",
            "https://example.test/game.nds.zip",
        ] {
            assert_eq!(uri_platform(uri), None);
        }
        assert_eq!(
            uri_platform("https://example.test/game.GBA?token=abc"),
            Some("gba")
        );
        assert_eq!(
            uri_platform("magnet:?xt=urn:btih:fixture&dn=Game%20Name.nds"),
            Some("nds")
        );
        assert_eq!(text_platform("Cyclops3 Goggles"), None);
        assert_eq!(
            DownloadService::infer_platform(
                None,
                None,
                "Game",
                &[
                    "https://example.test/a.nds".into(),
                    "https://example.test/b.gba".into()
                ],
                None
            ),
            "pc"
        );
    }
}

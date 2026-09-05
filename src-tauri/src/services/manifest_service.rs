use serde_json::{Map, Value};
use scraper::Html;
use crate::models::DownloadSource;

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceOption {
    #[serde(flatten)]
    pub source: DownloadSource,
    pub access: String,
    pub downloadable: bool,
    pub reason: Option<String>,
}

pub fn source_option(source: DownloadSource) -> SourceOption {
    let access = source_access(&source.uri).unwrap_or("unsupported");
    let reason = if !source.available {
        Some("Fuente marcada como no disponible")
    } else {
        match access {
            "host_page" => Some("Pagina de alojamiento: requiere un conector; no es un archivo directo"),
            "magnet" | "torrent" => Some("BitTorrent no disponible en esta version"),
            "unverified_http" => Some("URL HTTP sin verificar; puede requerir una pagina intermedia"),
            "unsupported" => Some("Protocolo no admitido"),
            _ => None,
        }
    };
    SourceOption { downloadable: source.available && matches!(access, "http" | "unverified_http"),
        source, access: access.into(), reason: reason.map(str::to_string) }
}

pub fn text(value: &Value) -> Option<String> {
    let value = value.as_str()?.trim();
    if value.is_empty() || ["null", "undefined"].iter().any(|missing| value.eq_ignore_ascii_case(missing)) {
        None
    } else {
        Some(value.to_string())
    }
}

fn plain_html(value: &str) -> String {
    let html = Html::parse_fragment(value);
    html.root_element().descendants().filter_map(|node| {
        if node.ancestors().any(|parent| parent.value().as_element()
            .is_some_and(|element| matches!(element.name(), "script" | "style" | "noscript" | "template"))) {
            return None;
        }
        node.value().as_text().map(|text| text.to_string())
    }).collect::<Vec<_>>().join(" ").split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn normalize(item: &Value) -> Option<Value> {
    let original = item.as_object()?;
    let mut result: Map<String, Value> = original.clone();
    for value in result.values_mut() {
        if value.is_string() { *value = text(value).map(Value::String).unwrap_or(Value::Null); }
    }
    let title = result.get("title").and_then(text).or_else(|| result.get("name").and_then(text))?;
    result.insert("title".into(), title.into());
    let mut uris = Vec::new();
    if let Some(values) = original.get("uris").and_then(Value::as_array) {
        for value in values {
            if let Some(uri) = text(value).filter(|uri| source_access(uri).is_some()) {
                if !uris.contains(&uri) { uris.push(uri); }
            }
        }
    } else if let Some(uri) = original.get("url").and_then(text).or_else(|| original.get("uri").and_then(text)) {
        if source_access(&uri).is_some() { uris.push(uri); }
    }
    if uris.is_empty() { return None; }
    result.insert("uris".into(), serde_json::json!(uris));
    let year = ["releaseYear", "year"].iter().filter_map(|key| result.get(*key)).find_map(|value| {
        value.as_u64().or_else(|| text(value)?.parse::<u64>().ok()).filter(|year| (1900..=2100).contains(year))
    });
    result.insert("releaseYear".into(), year.map(Value::from).unwrap_or(Value::Null));
    let genre = result.get("genre").and_then(text).or_else(|| {
        let genres = original.get("genres")?.as_array()?.iter().filter_map(text).collect::<Vec<_>>().join(", ");
        (!genres.is_empty()).then_some(genres)
    });
    result.insert("genre".into(), genre.map(Value::String).unwrap_or(Value::Null));
    let description = result.get("description").and_then(text)
        .or_else(|| result.get("descriptionHtml").and_then(text).map(|html| plain_html(&html)));
    result.insert("description".into(), description.filter(|value| !value.is_empty()).map(Value::String).unwrap_or(Value::Null));
    let cover = result.get("coverImage").and_then(text).or_else(|| result.get("cover").and_then(text));
    result.insert("coverImage".into(), cover.map(Value::String).unwrap_or(Value::Null));
    Some(result.into())
}

pub fn source_access(uri: &str) -> Option<&'static str> {
    let url = reqwest::Url::parse(uri).ok()?;
    if url.scheme() == "magnet" { return Some("magnet"); }
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() { return None; }
    let path = url.path().to_ascii_lowercase();
    if path.ends_with(".torrent") { return Some("torrent"); }
    let host = url.host_str()?.trim_start_matches("www.");
    if host == "pixeldrain.com" && path.starts_with("/api/file/") { return Some("http"); }
    if ["megadb.net", "gofile.io", "mediafire.com", "1fichier.com", "pixeldrain.com",
        "datanodes.to", "buzzheavier.com", "bzzhr.to", "1337x.to", "rutor.info", "tapochek.net",
        "t.me", "vikingfile.com", "files.fm", "akirabox.com", "filekeeper.net"]
        .iter().any(|domain| host == *domain || host.ends_with(&format!(".{domain}"))) {
        return Some("host_page");
    }
    if ["zip", "7z", "rar", "iso", "chd", "pkg", "exe", "bin", "gz", "xz", "rvz", "gba", "sfc"]
        .iter().any(|extension| path.ends_with(&format!(".{extension}"))) { Some("http") } else { Some("unverified_http") }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn normalizes_aliases_and_html_without_inventing_metadata() {
        let entry = normalize(&json!({"title":"Fixture", "year":"2024", "genres":["Action", "null"],
            "description":"null", "descriptionHtml":"<p>Hello &amp; <b>world</b></p><script>bad()</script>",
            "coverImage":"undefined", "cover":"https://example.test/cover.jpg", "developer":" NULL ",
            "uris":["https://example.test/game.zip", "https://example.test/game.zip", "javascript:bad()", "magnet:?xt=test"]})).unwrap();
        assert_eq!(entry["releaseYear"], 2024);
        assert_eq!(entry["genre"], "Action");
        assert_eq!(entry["description"], "Hello & world");
        assert!(entry["developer"].is_null());
        assert_eq!(entry["coverImage"], "https://example.test/cover.jpg");
        assert_eq!(entry["uris"].as_array().unwrap().len(), 2);
        let legacy = normalize(&json!({"name":"Legacy", "url":"https://example.test/game.zip", "uploadDate":"2020-01-01"})).unwrap();
        assert!(legacy["releaseYear"].is_null());
        assert_eq!(legacy["title"], "Legacy");
        assert!(normalize(&json!({"title":"Broken", "uris":[]})).is_none());
    }

    #[test]
    fn distinguishes_transport_from_host_pages() {
        assert_eq!(source_access("https://torrent.example.test/game.zip"), Some("http"));
        assert_eq!(source_access("https://example.test/game.torrent?token=fixture"), Some("torrent"));
        assert_eq!(source_access("https://megadb.net/fixture"), Some("host_page"));
        assert_eq!(source_access("https://pixeldrain.com/api/file/fixture"), Some("http"));
        assert_eq!(source_access("https://example.test/download/42"), Some("unverified_http"));
        assert_eq!(source_access("file:///etc/passwd"), None);
    }
}
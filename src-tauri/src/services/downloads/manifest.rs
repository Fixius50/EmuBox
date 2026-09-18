pub use crate::models::DownloadSourceOption as SourceOption;
pub use crate::services::download_resolver::{source_access, source_option};
use serde_json::{Map, Value};

pub fn text(value: &Value) -> Option<String> {
    let value = value.as_str()?.trim();
    if value.is_empty()
        || ["null", "undefined"]
            .iter()
            .any(|missing| value.eq_ignore_ascii_case(missing))
    {
        None
    } else {
        Some(value.to_string())
    }
}

fn plain_html(value: &str) -> String {
    strip_html(value)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn strip_html(value: &str) -> String {
    let mut output = String::new();
    let mut cursor = 0;
    let mut skipped_tag: Option<String> = None;
    while let Some(start) = value[cursor..].find('<').map(|index| cursor + index) {
        if skipped_tag.is_none() {
            output.push_str(&decode_entities(&value[cursor..start]));
        }
        let Some(end) = value[start..].find('>').map(|index| start + index) else {
            if skipped_tag.is_none() {
                output.push_str(&decode_entities(&value[start..]));
            }
            return output;
        };
        let tag = tag_name(&value[start + 1..end]);
        if let Some(blocked) = skipped_tag.as_deref() {
            if value[start + 1..end].trim_start().starts_with('/')
                && tag.as_deref() == Some(blocked)
            {
                skipped_tag = None;
            }
        } else if tag
            .as_deref()
            .is_some_and(|name| matches!(name, "script" | "style" | "noscript" | "template"))
        {
            skipped_tag = tag;
        } else {
            output.push(' ');
        }
        cursor = end + 1;
    }
    if skipped_tag.is_none() {
        output.push_str(&decode_entities(&value[cursor..]));
    }
    output
}

fn tag_name(tag: &str) -> Option<String> {
    let name = tag
        .trim_start_matches('/')
        .trim_start()
        .chars()
        .take_while(|character| character.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase();
    (!name.is_empty()).then_some(name)
}

fn decode_entities(value: &str) -> String {
    let mut output = String::new();
    let mut cursor = 0;
    while let Some(start) = value[cursor..].find('&').map(|index| cursor + index) {
        output.push_str(&value[cursor..start]);
        let Some(end) = value[start..].find(';').map(|index| start + index) else {
            output.push_str(&value[start..]);
            return output;
        };
        let entity = &value[start + 1..end];
        if let Some(decoded) = decode_entity(entity) {
            output.push(decoded);
        } else {
            output.push_str(&value[start..=end]);
        }
        cursor = end + 1;
    }
    output.push_str(&value[cursor..]);
    output
}

fn decode_entity(entity: &str) -> Option<char> {
    match entity {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" | "#39" => Some('\''),
        "nbsp" => Some(' '),
        value if value.starts_with("#x") || value.starts_with("#X") => {
            u32::from_str_radix(&value[2..], 16)
                .ok()
                .and_then(char::from_u32)
        }
        value if value.starts_with('#') => value[1..].parse::<u32>().ok().and_then(char::from_u32),
        _ => None,
    }
}

pub fn normalize(item: &Value) -> Option<Value> {
    let original = item.as_object()?;
    let mut result: Map<String, Value> = original.clone();
    for value in result.values_mut() {
        if value.is_string() {
            *value = text(value).map(Value::String).unwrap_or(Value::Null);
        }
    }
    let title = result
        .get("title")
        .and_then(text)
        .or_else(|| result.get("name").and_then(text))?;
    result.insert("title".into(), title.into());
    let mut uris = Vec::new();
    if let Some(values) = original.get("uris").and_then(Value::as_array) {
        for value in values {
            if let Some(uri) = text(value).filter(|uri| source_access(uri).is_some()) {
                if !uris.contains(&uri) {
                    uris.push(uri);
                }
            }
        }
    } else if let Some(uri) = original
        .get("url")
        .and_then(text)
        .or_else(|| original.get("uri").and_then(text))
    {
        if source_access(&uri).is_some() {
            uris.push(uri);
        }
    }
    if uris.is_empty() {
        return None;
    }
    result.insert("uris".into(), serde_json::json!(uris));
    let year = ["releaseYear", "year"]
        .iter()
        .filter_map(|key| result.get(*key))
        .find_map(|value| {
            value
                .as_u64()
                .or_else(|| text(value)?.parse::<u64>().ok())
                .filter(|year| (1900..=2100).contains(year))
        });
    result.insert(
        "releaseYear".into(),
        year.map(Value::from).unwrap_or(Value::Null),
    );
    let genre = result.get("genre").and_then(text).or_else(|| {
        let genres = original
            .get("genres")?
            .as_array()?
            .iter()
            .filter_map(text)
            .collect::<Vec<_>>()
            .join(", ");
        (!genres.is_empty()).then_some(genres)
    });
    result.insert(
        "genre".into(),
        genre.map(Value::String).unwrap_or(Value::Null),
    );
    let description = result.get("description").and_then(text).or_else(|| {
        result
            .get("descriptionHtml")
            .and_then(text)
            .map(|html| plain_html(&html))
    });
    result.insert(
        "description".into(),
        description
            .filter(|value| !value.is_empty())
            .map(Value::String)
            .unwrap_or(Value::Null),
    );
    let cover = result
        .get("coverImage")
        .and_then(text)
        .or_else(|| result.get("cover").and_then(text));
    result.insert(
        "coverImage".into(),
        cover.map(Value::String).unwrap_or(Value::Null),
    );
    Some(result.into())
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
    fn strips_blocked_html_content_and_common_entities() {
        let text = plain_html(
            "<style>bad</style><p>A&nbsp;B &#x26; C</p><template>hidden</template><span>&lt;ok&gt;</span>",
        );
        assert_eq!(text, "A B & C <ok>");
    }

    #[test]
    fn distinguishes_transport_from_host_pages() {
        assert_eq!(
            source_access("https://torrent.example.test/game.zip"),
            Some("http")
        );
        assert_eq!(
            source_access("https://example.test/game.torrent?token=fixture"),
            Some("torrent")
        );
        assert_eq!(
            source_access("https://megadb.net/fixture"),
            Some("host_page")
        );
        assert_eq!(
            source_access("https://pixeldrain.com/api/file/fixture"),
            Some("http")
        );
        assert_eq!(
            source_access("https://example.test/download/42"),
            Some("unverified_http")
        );
        assert_eq!(source_access("file:///etc/passwd"), None);
    }
}

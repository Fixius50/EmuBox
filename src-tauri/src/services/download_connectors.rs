use crate::errors::EmuBoxError;

pub fn connector(uri: &str) -> Option<&'static str> {
    let url = reqwest::Url::parse(uri).ok()?;
    let host = url.host_str()?.trim_start_matches("www.");
    let segments: Vec<_> = url.path_segments()?.collect();
    (host == "pixeldrain.com"
        && segments.len() == 2
        && segments[0] == "u"
        && !segments[1].is_empty()
        && segments[1]
            .chars()
            .all(|character| character.is_ascii_alphanumeric()))
    .then_some("pixeldrain_public_file")
}

pub fn resolve_uri(uri: &str) -> Result<String, EmuBoxError> {
    match connector(uri) {
        Some("pixeldrain_public_file") => {
            let url = reqwest::Url::parse(uri)
                .map_err(|_| EmuBoxError::InvalidConfiguration("URL Pixeldrain invalida".into()))?;
            let id = url
                .path_segments()
                .and_then(|mut segments| segments.nth(1))
                .unwrap();
            Ok(format!("https://pixeldrain.com/api/file/{id}"))
        }
        _ => Ok(uri.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn public_api_does_not_resolve_lists_or_other_domains() {
        assert_eq!(
            resolve_uri("https://pixeldrain.com/u/Abcd1234").unwrap(),
            "https://pixeldrain.com/api/file/Abcd1234"
        );
        assert_eq!(connector("https://pixeldrain.com/l/Abcd1234"), None);
        assert_eq!(
            connector("https://pixeldrain.com.other.test/u/Abcd1234"),
            None
        );
    }
}

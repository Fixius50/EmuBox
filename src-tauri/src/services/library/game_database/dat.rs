use crate::errors::EmuBoxError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Release {
    pub(super) title: String,
    pub(super) canonical_title: String,
    pub(super) region: Option<String>,
    pub(super) serial: Option<String>,
    pub(super) crc: Option<String>,
    pub(super) md5: Option<String>,
    pub(super) sha1: Option<String>,
}

pub fn normalize_title(value: &str) -> String {
    value
        .trim_start_matches(|character: char| !character.is_alphanumeric())
        .chars()
        .flat_map(char::to_lowercase)
        .map(|character| {
            if character.is_alphanumeric() {
                character
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn edition_marker(value: &str) -> bool {
    let lower = value.to_lowercase();
    [
        "usa",
        "europe",
        "world",
        "japan",
        "asia",
        "korea",
        "brazil",
        "australia",
        "rev ",
        "revision",
        "disc ",
        "disk ",
        "side ",
        "beta",
        "demo",
        "proto",
        "sample",
        "unl",
        "unlicensed",
        "virtual console",
        "psn",
        "v1.",
        "v2.",
        "en,",
        "fr,",
        "de,",
        "es,",
        "it,",
        "ja,",
    ]
    .iter()
    .any(|marker| lower == *marker || lower.starts_with(marker))
}

pub fn canonical_title(value: &str) -> String {
    let mut title = value
        .trim_start_matches(|character: char| !character.is_alphanumeric())
        .trim()
        .to_string();
    loop {
        let Some(end) = title.strip_suffix(')') else {
            break;
        };
        let Some(start) = end.rfind('(') else { break };
        if !edition_marker(end[start + 1..].trim()) {
            break;
        }
        title.truncate(start);
        title = title.trim_end().to_string();
    }
    for separator in [" [", " {"].iter() {
        if let Some(index) = title.rfind(separator) {
            let marker = title[index + 2..].trim_end_matches([']', '}']).trim();
            if edition_marker(marker) {
                title.truncate(index);
            }
        }
    }
    title
        .trim()
        .trim_end_matches(['-', '–', '—', ':'])
        .trim()
        .to_string()
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Word(String),
    Text(String),
    Open,
    Close,
}

fn lex(input: &str) -> Result<Vec<Token>, EmuBoxError> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(character) = chars.next() {
        if character.is_whitespace() {
            continue;
        }
        match character {
            '(' => tokens.push(Token::Open),
            ')' => tokens.push(Token::Close),
            '"' => {
                let mut text = String::new();
                let mut closed = false;
                while let Some(next) = chars.next() {
                    match next {
                        '"' => {
                            closed = true;
                            break;
                        }
                        '\\' => match chars.next() {
                            Some('"') => text.push('"'),
                            Some('\\') => text.push('\\'),
                            Some(value) => {
                                text.push('\\');
                                text.push(value);
                            }
                            None => break,
                        },
                        value => text.push(value),
                    }
                }
                if !closed {
                    return Err(EmuBoxError::InvalidConfiguration(
                        "DAT maestro contiene texto sin cerrar".into(),
                    ));
                }
                tokens.push(Token::Text(text));
            }
            value => {
                let mut word = value.to_string();
                while chars
                    .peek()
                    .is_some_and(|next| !next.is_whitespace() && !matches!(next, '(' | ')' | '"'))
                {
                    word.push(chars.next().unwrap());
                }
                tokens.push(Token::Word(word));
            }
        }
    }
    Ok(tokens)
}

fn field(tokens: &[Token], key: &str) -> Option<String> {
    tokens.windows(2).find_map(|pair| match pair {
        [Token::Word(name), Token::Text(value)] if name == key => Some(value.clone()),
        [Token::Word(name), Token::Word(value)] if name == key => Some(value.clone()),
        _ => None,
    })
}

pub(super) fn parse_dat(input: &str) -> Result<Vec<Release>, EmuBoxError> {
    let tokens = lex(input)?;
    let mut releases = Vec::new();
    let mut index = 0;
    while index + 1 < tokens.len() {
        if tokens[index] != Token::Word("game".into()) || tokens[index + 1] != Token::Open {
            index += 1;
            continue;
        }
        let start = index + 2;
        let mut depth = 1;
        index = start;
        while index < tokens.len() && depth > 0 {
            match tokens[index] {
                Token::Open => depth += 1,
                Token::Close => depth -= 1,
                _ => {}
            }
            index += 1;
        }
        if depth != 0 {
            return Err(EmuBoxError::InvalidConfiguration(
                "DAT maestro contiene parentesis sin cerrar".into(),
            ));
        }
        let body = &tokens[start..index - 1];
        let Some(title) = field(body, "name").or_else(|| field(body, "description")) else {
            continue;
        };
        let canonical = canonical_title(&title);
        if canonical.is_empty() {
            continue;
        }
        releases.push(Release {
            title,
            canonical_title: canonical,
            region: field(body, "region"),
            serial: field(body, "serial"),
            crc: field(body, "crc").map(|value| value.to_uppercase()),
            md5: field(body, "md5").map(|value| value.to_uppercase()),
            sha1: field(body, "sha1").map(|value| value.to_uppercase()),
        });
    }
    if releases.is_empty() {
        return Err(EmuBoxError::InvalidConfiguration(
            "DAT maestro sin juegos reconocibles".into(),
        ));
    }
    Ok(releases)
}

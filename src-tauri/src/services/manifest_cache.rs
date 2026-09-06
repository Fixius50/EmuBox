use std::{io::Read, time::Duration};
use reqwest::{blocking::Client, header, StatusCode};
use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use crate::errors::EmuBoxError;

pub const REFRESH_SECONDS: u64 = 6 * 60 * 60;
const MAX_BYTES: u64 = 32 * 1024 * 1024;
const FORMAT_VERSION: i64 = 1;

#[derive(Debug)]
pub struct CacheRecord {
    pub(crate) url: String,
    etag: Option<String>,
    modified: Option<String>,
    digest: String,
    checked: u64,
}

#[cfg(test)]
impl CacheRecord {
    pub(crate) fn fixture(url: &str, content: &str, checked: u64) -> Self {
        Self { url: url.into(), etag: None, modified: None,
            digest: format!("{:x}", Sha256::digest(content.as_bytes())), checked }
    }
}

pub enum FetchResult {
    Fresh,
    Unchanged(CacheRecord),
    Changed { content: String, filename: Option<String>, cache: CacheRecord },
}

fn storage(error: impl std::fmt::Display) -> EmuBoxError {
    EmuBoxError::StorageUnavailable(error.to_string())
}

fn initialize(connection: &Connection) -> Result<(), EmuBoxError> {
    connection.execute_batch("CREATE TABLE IF NOT EXISTS manifest_http_cache (
        url TEXT PRIMARY KEY, etag TEXT, modified TEXT, digest TEXT NOT NULL,
        checked INTEGER NOT NULL, format_version INTEGER NOT NULL);
        CREATE TABLE IF NOT EXISTS manifest_fetch_failures (url TEXT PRIMARY KEY, retry_after INTEGER NOT NULL);").map_err(storage)
}

pub fn remember(connection: &Connection, record: &CacheRecord) -> Result<(), EmuBoxError> {
    initialize(connection)?;
    connection.execute("INSERT INTO manifest_http_cache VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        ON CONFLICT(url) DO UPDATE SET etag=excluded.etag, modified=excluded.modified,
        digest=excluded.digest, checked=excluded.checked, format_version=excluded.format_version",
        params![record.url, record.etag, record.modified, record.digest, record.checked, FORMAT_VERSION]).map_err(storage)?;
    connection.execute("DELETE FROM manifest_fetch_failures WHERE url=?1", params![record.url]).map_err(storage)?;
    Ok(())
}

pub fn fetch(connection: &Connection, url: &str, now: u64) -> Result<FetchResult, EmuBoxError> {
    initialize(connection)?;
    let retry_after: Option<u64> = connection.query_row("SELECT retry_after FROM manifest_fetch_failures WHERE url=?1",
        params![url], |row| row.get(0)).optional().map_err(storage)?;
    if retry_after.is_some_and(|retry| retry > now && retry - now <= 15 * 60) { return Ok(FetchResult::Fresh); }
    let result = fetch_current(connection, url, now);
    if result.is_err() {
        connection.execute("INSERT INTO manifest_fetch_failures VALUES (?1, ?2)
            ON CONFLICT(url) DO UPDATE SET retry_after=excluded.retry_after", params![url, now + 15 * 60]).map_err(storage)?;
    }
    result
}

fn fetch_current(connection: &Connection, url: &str, now: u64) -> Result<FetchResult, EmuBoxError> {
    let previous = connection.query_row(
        "SELECT etag, modified, digest, checked FROM manifest_http_cache WHERE url=?1 AND format_version=?2",
        params![url, FORMAT_VERSION], |row| Ok(CacheRecord { url: url.into(), etag: row.get(0)?,
            modified: row.get(1)?, digest: row.get(2)?, checked: row.get(3)? }))
        .optional().map_err(storage)?;
    if previous.as_ref().is_some_and(|record| now >= record.checked && now - record.checked < REFRESH_SECONDS) {
        return Ok(FetchResult::Fresh);
    }
    let parsed = reqwest::Url::parse(url).map_err(|error| EmuBoxError::InvalidConfiguration(error.to_string()))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(EmuBoxError::InvalidConfiguration("Solo se aceptan manifiestos HTTP/HTTPS".into()));
    }
    let client = Client::builder().connect_timeout(Duration::from_secs(5)).timeout(Duration::from_secs(25))
        .build().map_err(|error| EmuBoxError::Unknown(error.to_string()))?;
    let mut request = client.get(parsed.clone());
    if let Some(record) = &previous {
        if let Some(etag) = &record.etag { request = request.header(header::IF_NONE_MATCH, etag); }
        if let Some(modified) = &record.modified { request = request.header(header::IF_MODIFIED_SINCE, modified); }
    }
    let response = request.send().and_then(|response| response.error_for_status())
        .map_err(|error| EmuBoxError::Unknown(format!("No se pudo comprobar el manifiesto: {error}")))?;
    let etag = response.headers().get(header::ETAG).and_then(|value| value.to_str().ok()).map(str::to_owned);
    let modified = response.headers().get(header::LAST_MODIFIED).and_then(|value| value.to_str().ok()).map(str::to_owned);
    if response.status() == StatusCode::NOT_MODIFIED {
        let mut record = previous.ok_or_else(|| EmuBoxError::InvalidConfiguration("HTTP 304 sin catalogo importado".into()))?;
        record.checked = now;
        record.etag = etag.or(record.etag);
        record.modified = modified.or(record.modified);
        return Ok(FetchResult::Unchanged(record));
    }
    let mut content = String::new();
    response.take(MAX_BYTES + 1).read_to_string(&mut content).map_err(storage)?;
    if content.len() as u64 > MAX_BYTES {
        return Err(EmuBoxError::InvalidConfiguration("El manifiesto supera 32 MiB".into()));
    }
    let digest = format!("{:x}", Sha256::digest(content.as_bytes()));
    let unchanged = previous.as_ref().is_some_and(|record| record.digest == digest);
    let cache = CacheRecord { url: url.into(), etag, modified, digest, checked: now };
    if unchanged { return Ok(FetchResult::Unchanged(cache)); }
    let filename = parsed.path_segments().and_then(|mut segments| segments.rfind(|segment| !segment.is_empty())).map(str::to_owned);
    Ok(FetchResult::Changed { content, filename, cache })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::Write, net::TcpListener, thread};

    #[test]
    fn conditional_requests_skip_fresh_and_unchanged_content() {
        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/catalog.json", server.local_addr().unwrap());
        let worker = thread::spawn(move || {
            let mut requests = Vec::new();
            for response in [
                "HTTP/1.1 200 OK\r\nETag: \"v1\"\r\nLast-Modified: Sat, 05 Sep 2026 10:00:00 GMT\r\nContent-Length: 16\r\nConnection: close\r\n\r\n{\"downloads\":[]}",
                "HTTP/1.1 304 Not Modified\r\nConnection: close\r\n\r\n",
                "HTTP/1.1 200 OK\r\nContent-Length: 16\r\nConnection: close\r\n\r\n{\"downloads\":[]}",
                "HTTP/1.1 200 OK\r\nContent-Length: 12\r\nConnection: close\r\n\r\n{\"games\":[]}",
                "HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            ] {
                let (mut stream, _) = server.accept().unwrap();
                stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
                let mut bytes = Vec::new();
                let mut buffer = [0; 1024];
                while !bytes.windows(4).any(|part| part == b"\r\n\r\n") {
                    let size = stream.read(&mut buffer).unwrap();
                    assert!(size > 0);
                    bytes.extend_from_slice(&buffer[..size]);
                }
                requests.push(String::from_utf8(bytes).unwrap());
                stream.write_all(response.as_bytes()).unwrap();
            }
            requests
        });
        let connection = Connection::open_in_memory().unwrap();
        let start = 100_000;
        let FetchResult::Changed { cache, .. } = fetch(&connection, &url, start).unwrap() else { panic!("first import"); };
        remember(&connection, &cache).unwrap();
        assert!(matches!(fetch(&connection, &url, start + 1).unwrap(), FetchResult::Fresh));
        let FetchResult::Unchanged(cache) = fetch(&connection, &url, start + REFRESH_SECONDS).unwrap() else { panic!("304"); };
        remember(&connection, &cache).unwrap();
        let FetchResult::Unchanged(cache) = fetch(&connection, &url, start + 2 * REFRESH_SECONDS).unwrap() else { panic!("digest"); };
        remember(&connection, &cache).unwrap();
        assert!(matches!(fetch(&connection, &url, start + 3 * REFRESH_SECONDS).unwrap(), FetchResult::Changed { .. }));
        assert!(fetch(&connection, &url, start + 3 * REFRESH_SECONDS + 1).is_err());
        assert!(matches!(fetch(&connection, &url, start + 3 * REFRESH_SECONDS + 2).unwrap(), FetchResult::Fresh));
        let requests = worker.join().unwrap();
        assert!(!requests[0].contains("if-none-match"));
        assert!(requests[1].contains("if-none-match: \"v1\""));
        assert!(requests[1].contains("if-modified-since:"));
    }
}
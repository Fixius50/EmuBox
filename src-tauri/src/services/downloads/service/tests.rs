use crate::{
    errors::EmuBoxError,
    models::DownloadSourceType,
    services::{db_service::DatabaseService, game_service::GameService, paths},
};
use rusqlite::params;
use std::path::Path;
fn network_message(detail: &str, timeout: bool) -> &'static str {
    if detail.contains("certificate") || detail.contains("certificado") {
        "No se pudo validar el certificado HTTPS de la fuente. No se ha desactivado la comprobacion de seguridad."
    } else if timeout {
        "La fuente no respondio a tiempo. Comprueba la conexion o selecciona otra fuente."
    } else if detail.contains("dns") || detail.contains("name resolution") {
        "No se pudo resolver el dominio de la fuente."
    } else {
        "No se pudo conectar con la fuente de descarga. Puede estar caida o requerir un conector de alojamiento."
    }
}

use super::*;

#[test]
fn manifest_cache_updates_only_changed_entries_and_keeps_user_state() {
    use crate::services::manifest_cache::CacheRecord;
    let url = "https://cache-fixture.test/main.json";
    let mirror = "https://cache-fixture.test/mirror.json";
    let manifest = serde_json::json!({"platform":"ps1", "downloads":[
            {"gameId":"cache-fixture-one", "title":"Cache one", "uris":["https://cache-fixture.test/one.chd"]},
            {"gameId":"cache-fixture-two", "title":"Cache two", "uris":["https://cache-fixture.test/two.chd"]}
        ]}).to_string();
    let record = CacheRecord::fixture(url, &manifest, 100);
    let imported = DownloadService::import_manifest_cached(&manifest, None, Some(&record)).unwrap();
    assert_eq!(imported.len(), 2);
    assert!(
        DownloadService::import_manifest_cached(&manifest, None, Some(&record))
            .unwrap()
            .is_empty()
    );
    let connection = DatabaseService::get_connection().unwrap();
    connection
        .execute(
            "UPDATE games SET favorite=1, rom_path='/fixture/one.chd' WHERE id='cache-fixture-one'",
            [],
        )
        .unwrap();
    let mirrored = serde_json::json!({"platform":"ps1", "downloads":[
            {"gameId":"cache-fixture-two", "title":"Cache two", "uris":["https://cache-fixture.test/two.chd"]}
        ]}).to_string();
    DownloadService::import_manifest_cached(
        &mirrored,
        None,
        Some(&CacheRecord::fixture(mirror, &mirrored, 100)),
    )
    .unwrap();
    let changed = serde_json::json!({"platform":"ps1", "downloads":[
        {"gameId":"cache-fixture-one", "title":"Cache one", "year":2001,
            "uris":["https://cache-fixture.test/new.chd"]}
    ]})
    .to_string();
    assert_eq!(
        DownloadService::import_manifest_cached(
            &changed,
            None,
            Some(&CacheRecord::fixture(url, &changed, 200))
        )
        .unwrap()
        .len(),
        1
    );
    let old_available: i64 = connection
        .query_row(
            "SELECT available FROM download_sources WHERE id=?1",
            params![imported[0].id],
            |row| row.get(0),
        )
        .unwrap();
    let shared_available: i64 = connection
        .query_row(
            "SELECT available FROM download_sources WHERE id=?1",
            params![imported[1].id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(old_available, 0);
    assert_eq!(shared_available, 1);
    let game = GameService::get_game_by_id("cache-fixture-one".into())
        .unwrap()
        .unwrap();
    assert!(game.favorite && game.installed);
    assert_eq!(game.release_year, 2001);
    assert!(DownloadService::import_manifest_cached(
        "invalid",
        None,
        Some(&CacheRecord::fixture(url, "invalid", 300))
    )
    .is_err());
    drop(connection);
    let reopened = DatabaseService::get_connection().unwrap();
    let checked: u64 = reopened
        .query_row(
            "SELECT checked FROM manifest_http_cache WHERE url=?1",
            params![url],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(checked, 200);
    assert!(matches!(
        crate::services::manifest_cache::fetch(&reopened, url, 201).unwrap(),
        crate::services::manifest_cache::FetchResult::Fresh
    ));
    assert!(DownloadService::list_jobs()
        .unwrap()
        .iter()
        .all(|job| !job.game_id.starts_with("cache-fixture")));
}

#[test]
fn reports_tls_dns_and_timeout_without_exposing_urls() {
    assert!(network_message("invalid peer certificate", false).contains("certificado HTTPS"));
    assert!(network_message("dns error", false).contains("dominio"));
    assert!(network_message("", true).contains("tiempo"));
}

#[test]
fn imports_aliases_all_sources_and_legacy_without_jobs() {
    let manifest = r#"{"downloads":[{"gameId":"normalization-fixture", "title":"Fixture", "year":2024,
            "genres":["Action","Puzzle"], "descriptionHtml":"<p>Real &amp; plain</p>", "developer":"null",
            "sourceId":"shared", "uris":["https://megadb.net/fixture", "https://torrent.example.test/game.zip", "magnet:?xt=fixture"]}]}"#;
    let first = DownloadService::import_from_json(manifest).unwrap();
    let second = DownloadService::import_from_json(manifest).unwrap();
    assert_eq!(first.len(), 3);
    let options = DownloadService::list_sources("normalization-fixture").unwrap();
    assert!(!options[0].downloadable);
    assert!(options[1].downloadable);
    assert!(!options[2].downloadable);
    assert!(DownloadService::download_game("normalization-fixture".into()).is_err());
    assert!(DownloadService::download_game_from_source(
        "normalization-fixture".into(),
        Some(options[0].source.id.clone())
    )
    .is_err());
    assert!(DownloadService::download_game_from_source(
        "normalization-fixture".into(),
        Some("another-game".into())
    )
    .is_err());
    assert_eq!(
        first.iter().map(|source| &source.id).collect::<Vec<_>>(),
        second.iter().map(|source| &source.id).collect::<Vec<_>>()
    );
    assert!(matches!(first[1].source_type, DownloadSourceType::Http));
    let game = GameService::get_game_by_id("normalization-fixture".into())
        .unwrap()
        .unwrap();
    assert_eq!(game.release_year, 2024);
    assert_eq!(game.genre, "Action, Puzzle");
    assert_eq!(game.description, "Real & plain");
    assert_eq!(game.developer, "");
    assert!(!game.installed);
    assert!(DownloadService::list_jobs()
        .unwrap()
        .iter()
        .all(|job| job.game_id != game.id));
    let legacy = DownloadService::import_from_json(r#"{"games":[{"gameId":"legacy-normalization-fixture", "name":"Legacy", "platform":"ps1", "url":"https://example.test/game.torrent?key=fixture"}]}"#).unwrap();
    assert_eq!(legacy.len(), 1);
    assert!(matches!(legacy[0].source_type, DownloadSourceType::Torrent));
}

#[test]
fn destination_is_confined_to_platform_directory() {
    let destination =
        DownloadService::destination_path("ps3", "https://example.test/files/game.pkg").unwrap();
    assert_eq!(
        destination,
        Path::new(&paths::games_dir()).join("ps3/game.pkg")
    );
    assert!(DownloadService::destination_path("../ps3", "https://example.test/game.pkg").is_err());
}

#[test]
fn destination_extracts_dn_from_magnet() {
    let destination = DownloadService::destination_path(
        "pc",
        "magnet:?xt=urn:btih:123&dn=Dead.Rising.3.zip&tr=udp",
    )
    .unwrap();
    assert_eq!(
        destination,
        Path::new(&paths::games_dir()).join("pc/Dead.Rising.3.zip")
    );
}

#[test]
fn parses_hydra_format_json_with_downloads_and_file_sizes() {
    let json = r#"{
            "name": "Hydra Test Source",
            "downloads": [
                {
                    "title": "Crash Bandicoot [PS1]",
                    "uris": ["https://example.test/crash.chd"],
                    "fileSize": "450 MB",
                    "uploadDate": "2023-01-15T12:00:00Z"
                },
                {
                    "title": "Dead Rising 3",
                    "uris": ["magnet:?xt=urn:btih:abc&dn=Dead.Rising.3.zip"],
                    "fileSize": "13.58 GB",
                    "uploadDate": "2014-09-30T10:53:57.000Z"
                }
            ]
        }"#;
    let sources = DownloadService::import_from_json(json).unwrap();
    assert_eq!(sources.len(), 2);
    let first = GameService::get_game_by_id(sources[0].game_id.clone())
        .unwrap()
        .unwrap();
    assert_eq!(first.platform, "ps1");
    assert!(!first.installed);
    assert_ne!(first.release_year, 2023);
    assert_eq!(sources[0].size_bytes, Some(450 * 1024 * 1024));
    assert!(DownloadService::list_jobs()
        .unwrap()
        .iter()
        .all(|job| !sources.iter().any(|source| source.game_id == job.game_id)));
    assert_eq!(DownloadService::import_from_json(json).unwrap().len(), 2);
    assert!(matches!(
        DownloadService::download_game(sources[1].game_id.clone()),
        Err(EmuBoxError::InvalidConfiguration(_))
    ));
    assert!(DownloadService::list_jobs()
        .unwrap()
        .iter()
        .all(|job| !sources.iter().any(|source| source.game_id == job.game_id)));
}

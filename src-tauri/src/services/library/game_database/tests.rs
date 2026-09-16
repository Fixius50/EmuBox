use super::dat::{parse_dat, Release};
use super::repository::import_releases;
use super::*;
use crate::services::db_service::DatabaseService;
use rusqlite::params;
#[test]
fn dat_parser_preserves_versions_and_identity_fields() {
    let releases = parse_dat(r##"clrmamepro ( name "Fixture" )
game ( name "#Mario Bros. (USA) (Rev 1)" description "ignored" region "USA" serial "NES-MA-USA" rom ( name "Mario.nes" size 4 crc 01020304 md5 AABB sha1 CCDDEE ) )
game ( name "Mario Bros. (Europe)" rom ( name "Mario Europe.nes" crc 05060708 ) )"##).unwrap();
    assert_eq!(releases.len(), 2);
    assert_eq!(releases[0].canonical_title, "Mario Bros.");
    assert_eq!(releases[0].serial.as_deref(), Some("NES-MA-USA"));
    assert_eq!(releases[0].crc.as_deref(), Some("01020304"));
    assert_ne!(releases[0].title, releases[1].title);
}
#[test]
fn title_matching_is_conservative_and_strips_only_known_markers() {
    assert_eq!(
        canonical_title("^^Mario Bros. (USA) (Rev 1)"),
        "Mario Bros."
    );
    assert_eq!(
        canonical_title("Mario (The Lost Levels)"),
        "Mario (The Lost Levels)"
    );
    assert_eq!(normalize_title("  #Mario: Bros.  "), "mario bros");
}
#[test]
fn local_variants_upgrade_to_authoritative_identity_without_changing_catalog_ids() {
    use crate::{models::game::CatalogEntry, services::GameService};
    GameService::get_platforms().unwrap();
    let ids = ["game-db-canonical-fixture-a", "game-db-canonical-fixture-b"];
    for (id, title) in [
        (ids[0], "#Canonical Fixture (USA)"),
        (ids[1], "^^Canonical Fixture [Rev 1]"),
    ] {
        GameService::upsert_catalog_entry(CatalogEntry {
            id: id.into(),
            title: title.into(),
            platform_id: "nes".into(),
            platform_name: "Nintendo Entertainment System".into(),
            release_year: None,
            genre: None,
            developer: None,
            publisher: None,
            rating: None,
            cover_image: None,
            backdrop_image: None,
            description: None,
        })
        .unwrap();
    }
    ensure_local_index().unwrap();
    let connection = DatabaseService::get_connection().unwrap();
    let local_count: i64 = connection.query_row("SELECT COUNT(DISTINCT canonical_game_id) FROM catalog_game_matches WHERE catalog_game_id IN (?1,?2)", ids, |row| row.get(0)).unwrap();
    assert_eq!(local_count, 1);
    drop(connection);
    assert!(GameService::toggle_favorite(ids[0].into()).unwrap());
    import_releases(
        "nes",
        &[Release {
            title: "Canonical Fixture (USA)".into(),
            canonical_title: "Canonical Fixture".into(),
            region: Some("USA".into()),
            serial: None,
            crc: Some("ABCDEF01".into()),
            md5: None,
            sha1: None,
        }],
    )
    .unwrap();
    let connection = DatabaseService::get_connection().unwrap();
    let matches: Vec<(String, String, i64)> = {
        let mut statement = connection.prepare("SELECT match.catalog_game_id,canonical.authority,match.confidence FROM catalog_game_matches AS match JOIN canonical_games AS canonical ON canonical.id=match.canonical_game_id WHERE match.catalog_game_id IN (?1,?2) ORDER BY match.catalog_game_id").unwrap();
        statement
            .query_map(ids, |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap()
    };
    assert_eq!(matches.len(), 2);
    assert!(matches
        .iter()
        .all(|entry| entry.1 == AUTHORITY && entry.2 >= 90));
    assert_eq!(matches[0].0, ids[0]);
    assert_eq!(matches[1].0, ids[1]);
    let canonical_id: String = connection
        .query_row(
            "SELECT canonical_game_id FROM catalog_game_matches WHERE catalog_game_id=?1",
            [ids[0]],
            |row| row.get(0),
        )
        .unwrap();
    for (index, id) in ids.iter().enumerate() {
        connection.execute("INSERT INTO download_sources(id,game_id,name,source_type,uri,available) VALUES(?1,?2,?3,'http',?4,1)",
            params![format!("game-db-source-{index}"), id, format!("Source {index}"), format!("https://example.invalid/{index}")]).unwrap();
    }
    drop(connection);
    let options = options(&canonical_id).unwrap();
    assert!(options.game.favorite);
    assert_eq!(options.releases.len(), 2);
    assert_eq!(options.sources.len(), 2);
    assert!(ids.iter().all(|id| options
        .sources
        .iter()
        .any(|option| option.source.game_id == *id)));
    let connection = DatabaseService::get_connection().unwrap();
    connection
        .execute(
            "DELETE FROM download_sources WHERE id LIKE 'game-db-source-%'",
            [],
        )
        .unwrap();
    for id in ids {
        connection
            .execute("DELETE FROM games WHERE id=?1", [id])
            .unwrap();
    }
    connection
        .execute(
            "DELETE FROM canonical_games WHERE normalized_title='canonical fixture'",
            [],
        )
        .unwrap();
}

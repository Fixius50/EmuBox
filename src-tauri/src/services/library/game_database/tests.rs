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
fn concurrent_local_indexers_preserve_one_match_per_variant() {
    crate::services::GameService::get_platforms().unwrap();
    let connection = DatabaseService::get_connection().unwrap();
    for index in 0..16 {
        connection.execute("INSERT INTO games(id,title,platform_id,platform_name) VALUES(?1,'Concurrent Index Fixture','nes','NES')",
            [format!("concurrent-index-fixture-{index}")]).unwrap();
    }
    let barrier = std::sync::Barrier::new(6);
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..6)
            .map(|_| {
                scope.spawn(|| {
                    barrier.wait();
                    ensure_local_index()
                })
            })
            .collect();
        for worker in workers {
            worker.join().unwrap().unwrap();
        }
    });
    let (variants, identities): (usize, usize) = connection.query_row(
        "SELECT COUNT(*),COUNT(DISTINCT canonical_game_id) FROM catalog_game_matches WHERE catalog_game_id LIKE 'concurrent-index-fixture-%'",
        [], |row| Ok((row.get(0)?, row.get(1)?))).unwrap();
    assert_eq!((variants, identities), (16, 1));
    connection
        .execute(
            "DELETE FROM games WHERE id LIKE 'concurrent-index-fixture-%'",
            [],
        )
        .unwrap();
    connection
        .execute(
            "DELETE FROM canonical_games WHERE normalized_title='concurrent index fixture'",
            [],
        )
        .unwrap();
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

#[test]
fn indexed_package_upgrades_after_public_database_is_already_cached() {
    use crate::{models::game::CatalogEntry, services::GameService};
    let game_id = "game-db-late-package-fixture";
    let local_id = "game-db-late-package-local";
    GameService::get_platforms().unwrap();
    import_releases("snes", &[Release {
        title: "Late Package Fixture (USA)".into(),
        canonical_title: "Late Package Fixture".into(),
        region: Some("USA".into()),
        serial: None, crc: Some("A1B2C3D4".into()), md5: None, sha1: None,
    }]).unwrap();
    GameService::upsert_catalog_entry(CatalogEntry {
        id: game_id.into(), title: "[Español] Late Package Fixture (USA) [Rev A]".into(),
        platform_id: "snes".into(), platform_name: "SNES".into(), release_year: None,
        genre: None, developer: None, publisher: None, rating: None,
        cover_image: None, backdrop_image: None, description: None,
    }).unwrap();
    let connection = DatabaseService::get_connection().unwrap();
    connection.execute("INSERT INTO canonical_games(id,authority,authority_id,title,normalized_title,platform_id,updated_at) VALUES(?1,'local-derived',?1,'[Español] Late Package Fixture (USA) [Rev A]','español late package fixture usa rev a','snes',0)", [local_id]).unwrap();
    connection.execute("INSERT INTO catalog_game_matches(catalog_game_id,canonical_game_id,method,confidence,matched_at) VALUES(?1,?2,'local-title',60,0)", params![game_id, local_id]).unwrap();
    drop(connection);
    assert_eq!(ensure_local_index().unwrap(), 1);
    let connection = DatabaseService::get_connection().unwrap();
    let (authority, release): (String, Option<String>) = connection.query_row("SELECT canonical.authority,match.release_id FROM catalog_game_matches AS match JOIN canonical_games AS canonical ON canonical.id=match.canonical_game_id WHERE match.catalog_game_id=?1", [game_id], |row| Ok((row.get(0)?, row.get(1)?))).unwrap();
    assert_eq!(authority, "libretro");
    assert!(release.is_some());
    assert_eq!(ensure_local_index().unwrap(), 0);
    connection.execute("DELETE FROM games WHERE id=?1", [game_id]).unwrap();
    connection.execute("DELETE FROM canonical_games WHERE id=?1", [local_id]).unwrap();
    connection.execute("DELETE FROM canonical_games WHERE normalized_title='late package fixture'", []).unwrap();
}

#[test]
fn pc_package_variants_share_local_identity_and_preserve_favorites() {
    use crate::{models::game::CatalogEntry, services::GameService};
    GameService::get_platforms().unwrap();
    let ids = ["game-db-pc-base", "game-db-pc-package", "game-db-pc-deluxe"];
    for (id, title) in [
        (ids[0], "Grouping Fixture PC"),
        (ids[1], "Grouping Fixture PC Free Download"),
        (ids[2], "Grouping Fixture PC Deluxe Edition"),
    ] {
        GameService::upsert_catalog_entry(CatalogEntry {
            id: id.into(), title: title.into(), platform_id: "pc".into(),
            platform_name: "PC".into(), release_year: None, genre: None,
            developer: None, publisher: None, rating: None, cover_image: None,
            backdrop_image: None, description: None,
        }).unwrap();
    }
    ensure_local_index().unwrap();
    let connection = DatabaseService::get_connection().unwrap();
    let base_id: String = connection.query_row("SELECT canonical_game_id FROM catalog_game_matches WHERE catalog_game_id=?1", [ids[0]], |row| row.get(0)).unwrap();
    assert_eq!(connection.query_row("SELECT canonical_game_id FROM catalog_game_matches WHERE catalog_game_id=?1", [ids[1]], |row| row.get::<_, String>(0)).unwrap(), base_id);
    assert_ne!(connection.query_row("SELECT canonical_game_id FROM catalog_game_matches WHERE catalog_game_id=?1", [ids[2]], |row| row.get::<_, String>(0)).unwrap(), base_id);
    let old_id = "game-db-pc-old-package";
    connection.execute("INSERT INTO canonical_games(id,authority,authority_id,title,normalized_title,platform_id,favorite,updated_at) VALUES(?1,'local-derived',?1,'Grouping Fixture PC Free Download','grouping fixture pc free download','pc',1,0)", [old_id]).unwrap();
    connection.execute("UPDATE catalog_game_matches SET canonical_game_id=?1 WHERE catalog_game_id=?2", params![old_id, ids[1]]).unwrap();
    connection.execute("INSERT INTO download_sources(id,game_id,name,source_type,uri,available) VALUES('game-db-pc-source',?1,'PC package','http','https://example.invalid/fixture',1)", [ids[1]]).unwrap();
    drop(connection);
    assert_eq!(ensure_local_index().unwrap(), 1);
    let upgraded = GameService::get_game_by_id(ids[1].into()).unwrap().unwrap();
    assert_eq!(upgraded.id, ids[1]);
    assert_eq!(upgraded.canonical_id.as_deref(), Some(base_id.as_str()));
    assert!(upgraded.favorite);
    let grouped = options(&base_id).unwrap();
    assert_eq!(grouped.releases.len(), 2);
    assert_eq!(grouped.sources[0].source.game_id, ids[1]);
    let connection = DatabaseService::get_connection().unwrap();
    connection.execute("DELETE FROM download_sources WHERE id='game-db-pc-source'", []).unwrap();
    for id in ids {
        connection.execute("DELETE FROM games WHERE id=?1", [id]).unwrap();
    }
    connection.execute("DELETE FROM canonical_games WHERE id=?1", [old_id]).unwrap();
    connection.execute("DELETE FROM canonical_games WHERE platform_id='pc' AND normalized_title LIKE 'grouping fixture pc%'", []).unwrap();
}

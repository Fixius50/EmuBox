use crate::models::{GameFilter, ScanGamesRequest};
use std::fs;

use super::*;

#[test]
fn failed_scan_preserves_records_and_does_not_invent_metadata() {
    let root = std::env::temp_dir().join(format!("emubox-scan-errors-{}", std::process::id()));
    let platform = root.join("ps2");
    fs::create_dir_all(&platform).unwrap();
    let file = platform.join("Scan Integrity Fixture.iso");
    fs::write(&file, b"fixture").unwrap();
    std::os::unix::fs::symlink(&platform, platform.join("cycle")).unwrap();
    let request = || {
        Some(ScanGamesRequest {
            platforms: Some(vec!["ps2".into()]),
            roms_directory: Some(root.to_string_lossy().into_owned()),
            deep_scan: Some(true),
        })
    };
    let first = GameService::scan_games(request()).unwrap();
    assert_eq!(first.scanned_count, 1);
    let id = "ps2-scan-integrity-fixture";
    let game = GameService::get_game_by_id(id.into()).unwrap().unwrap();
    assert_eq!(game.rating, 0.0);
    assert_eq!(game.release_year, 0);
    assert!(game.genre.is_empty());
    fs::remove_dir_all(&platform).unwrap();
    fs::write(&platform, b"not a directory").unwrap();
    let failed = GameService::scan_games(request()).unwrap();
    assert_eq!(failed.errors.len(), 1);
    assert_eq!(failed.removed_count, 0);
    assert!(GameService::get_game_by_id(id.into()).unwrap().is_some());
    fs::remove_file(&platform).unwrap();
    fs::create_dir(&platform).unwrap();
    assert_eq!(GameService::scan_games(request()).unwrap().removed_count, 1);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_clean_title_from_filename() {
    assert_eq!(
        GameService::clean_title_from_filename("2048 (World) (Homebrew)"),
        "2048"
    );
    assert_eq!(
        GameService::clean_title_from_filename("Gran_Turismo_4_[v1.0]_(USA)"),
        "Gran Turismo 4"
    );
    assert_eq!(
        GameService::clean_title_from_filename("Super_Mario_World"),
        "Super Mario World"
    );
}

#[test]
fn test_scan_games_directory() {
    let temp_dir = std::env::temp_dir().join(format!("emubox-game-test-{}", std::process::id()));
    let snes_dir = temp_dir.join("snes");
    fs::create_dir_all(&snes_dir).unwrap();

    let rom_file = snes_dir.join("Test_Game_(USA)_(v1.0).sfc");
    fs::write(&rom_file, "TEST_ROM_CONTENT").unwrap();

    let scan_req = ScanGamesRequest {
        platforms: Some(vec!["snes".to_string()]),
        roms_directory: Some(temp_dir.to_string_lossy().to_string()),
        deep_scan: None,
    };

    let res = GameService::scan_games(Some(scan_req)).unwrap();
    assert!(res.scanned_count >= 1);

    let games = GameService::get_games(Some(GameFilter {
        platform: Some("snes".to_string()),
        search: Some("Test Game".to_string()),
        favorite: None,
        limit: None,
        offset: None,
    }))
    .unwrap();

    assert!(!games.is_empty());
    assert_eq!(games[0].title, "Test Game");

    fs::remove_dir_all(&temp_dir).unwrap();
}

#[test]
fn test_scan_ps3_installed_folder() {
    let temp_dir = std::env::temp_dir().join(format!("emubox-ps3-test-{}", std::process::id()));
    let game_dir = temp_dir
        .join("ps3")
        .join("Gran Turismo 6 (Europe)")
        .join("PS3_GAME");
    fs::create_dir_all(&game_dir).unwrap();

    let scan_req = ScanGamesRequest {
        platforms: Some(vec!["ps3".to_string()]),
        roms_directory: Some(temp_dir.to_string_lossy().to_string()),
        deep_scan: Some(true),
    };
    let result = GameService::scan_games(Some(scan_req)).unwrap();
    assert_eq!(result.scanned_count, 1);

    let games = GameService::get_games(Some(GameFilter {
        platform: Some("ps3".to_string()),
        search: Some("Gran Turismo 6".to_string()),
        favorite: None,
        limit: None,
        offset: None,
    }))
    .unwrap();
    assert!(games.iter().any(|game| game.title == "Gran Turismo 6"));

    fs::remove_dir_all(&temp_dir).unwrap();
}

#[test]
fn ps4_folder_registers_boot_file_once() {
    let root = std::env::temp_dir().join(format!("emubox-ps4-scan-{}", std::process::id()));
    let game = root.join("ps4/Local PS4 Test");
    fs::create_dir_all(game.join("sce_sys")).unwrap();
    fs::write(game.join("eboot.bin"), b"fixture").unwrap();
    fs::write(game.join("sce_sys/param.sfo"), b"fixture").unwrap();
    let result = GameService::scan_games(Some(ScanGamesRequest {
        platforms: Some(vec!["ps4".into()]),
        roms_directory: Some(root.to_string_lossy().into_owned()),
        deep_scan: Some(true),
    }))
    .unwrap();
    assert_eq!(result.scanned_count, 1);
    let games = GameService::get_games(Some(GameFilter {
        platform: Some("ps4".into()),
        search: None,
        favorite: None,
        limit: None,
        offset: None,
    }))
    .unwrap();
    assert!(games
        .iter()
        .any(|entry| entry.rom_path.as_deref() == game.join("eboot.bin").to_str()));
    fs::remove_dir_all(root).unwrap();
}

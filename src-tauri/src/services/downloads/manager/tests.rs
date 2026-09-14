
    use super::*;
use super::publication::prepare_published;
use crate::models::PublishedDownload;
use sha2::{Digest, Sha256};
    static QUEUE_TEST: Mutex<()> = Mutex::new(());
    use std::{
        io::{Read, Write},
        net::TcpListener,
        time::{Duration, Instant},
    };

    #[test]
    fn prepares_published_archive_without_accessing_source() {
        let _test = QUEUE_TEST.lock().unwrap();
        let imported = DownloadService::import_from_json(&serde_json::json!({"downloads":[{"gameId":"local-preparation", "title":"Local preparation", "platform":"ps2", "uris":["http://127.0.0.1:1/unreachable.zip"]}]}).to_string()).unwrap();
        let mut source = imported[0].clone();
        let job = DownloadService::create_job(crate::models::CreateDownloadRequest {
            game_id: source.game_id.clone(),
            platform: "ps2".into(),
            source: source.clone(),
        })
        .unwrap();
        source.available = false;
        DatabaseService::get_connection()
            .unwrap()
            .execute(
                "UPDATE download_execution SET source_json=?1 WHERE job_id=?2",
                params![serde_json::to_string(&source).unwrap(), job.id],
            )
            .unwrap();
        let destination = Path::new(&job.destination_path);
        fs::create_dir_all(destination).unwrap();
        let archive_path = destination.join("original.zip");
        let mut archive = zip::ZipWriter::new(fs::File::create(&archive_path).unwrap());
        archive
            .start_file("game.iso", zip::write::SimpleFileOptions::default())
            .unwrap();
        archive.write_all(b"local test bytes").unwrap();
        archive.finish().unwrap();
        let mut package = PublishedDownload {
            installation: None,
            job_id: job.id.clone(),
            source_digest: format!("{:x}", Sha256::digest(serde_json::to_vec(&source).unwrap())),
            files: vec!["original.zip".into()],
            launch: None,
            preparation_reason: None,
        };
        fs::write(
            destination.join(".emubox-managed"),
            serde_json::to_vec(&package).unwrap(),
        )
        .unwrap();
        status(&job.id, "downloaded", "preparation_required", None).unwrap();
        let original = fs::read(&archive_path).unwrap();
        let marker = fs::read(destination.join(".emubox-managed")).unwrap();
        let mut invalid_checksum = source.clone();
        invalid_checksum.checksum = Some("0".repeat(64));
        prepare_published(
            &job,
            &invalid_checksum,
            destination,
            &mut package,
            &[archive_path.clone()],
            &TransferControl::default(),
        )
        .unwrap();
        assert!(matches!(
            DownloadService::get_job(&job.id).unwrap().unwrap().status,
            DownloadStatus::Downloaded
        ));
        assert_eq!(fs::read(&archive_path).unwrap(), original);
        assert_eq!(
            fs::read(destination.join(".emubox-managed")).unwrap(),
            marker
        );
        let cancelled = TransferControl::default();
        cancelled.cancelled.store(true, Ordering::Relaxed);
        run(&job, &source, None, &cancelled).unwrap();
        assert_eq!(fs::read(&archive_path).unwrap(), original);
        assert_eq!(
            fs::read(destination.join(".emubox-managed")).unwrap(),
            marker
        );
        DownloadService::resume(job.id.clone()).unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        let finished = loop {
            let current = DownloadService::get_job(&job.id).unwrap().unwrap();
            if matches!(
                current.status,
                DownloadStatus::Completed | DownloadStatus::Failed | DownloadStatus::Downloaded
            ) {
                break current;
            }
            assert!(Instant::now() < deadline, "preparation did not finish");
            thread::sleep(Duration::from_millis(10));
        };
        assert!(matches!(finished.status, DownloadStatus::Completed));
        assert!(archive_path.is_file());
        let package: PublishedDownload =
            serde_json::from_slice(&fs::read(destination.join(".emubox-managed")).unwrap())
                .unwrap();
        assert_eq!(
            fs::read(destination.join(package.launch.as_ref().unwrap())).unwrap(),
            b"local test bytes"
        );
        run(&job, &source, None, &TransferControl::default()).unwrap();
        fs::remove_dir_all(destination).unwrap();
        status(&job.id, "paused", "paused", None).unwrap();
        assert!(DownloadService::resume(job.id.clone()).is_err());
    }

    #[test]
    fn ambiguous_archive_requires_explicit_local_selection() {
        let _test = QUEUE_TEST.lock().unwrap();
        let imported = DownloadService::import_from_json(&serde_json::json!({"downloads":[{"gameId":"local-selection", "title":"Local selection", "platform":"ps2", "uris":["http://127.0.0.1:1/unreachable.zip"]}]}).to_string()).unwrap();
        let source = imported[0].clone();
        let job = DownloadService::create_job(crate::models::CreateDownloadRequest {
            game_id: source.game_id.clone(),
            platform: "ps2".into(),
            source: source.clone(),
        })
        .unwrap();
        let destination = Path::new(&job.destination_path);
        fs::create_dir_all(destination).unwrap();
        let original = destination.join("original.zip");
        let mut archive = zip::ZipWriter::new(fs::File::create(&original).unwrap());
        for name in ["disc1.iso", "disc2.iso"] {
            archive
                .start_file(name, zip::write::SimpleFileOptions::default())
                .unwrap();
            archive.write_all(b"local selection test").unwrap();
        }
        archive.finish().unwrap();
        let package = PublishedDownload {
            installation: None,
            job_id: job.id.clone(),
            source_digest: format!("{:x}", Sha256::digest(serde_json::to_vec(&source).unwrap())),
            files: vec!["original.zip".into()],
            launch: None,
            preparation_reason: None,
        };
        fs::write(
            destination.join(".emubox-managed"),
            serde_json::to_vec(&package).unwrap(),
        )
        .unwrap();
        status(&job.id, "downloaded", "preparation_required", None).unwrap();
        run(&job, &source, None, &TransferControl::default()).unwrap();
        let paths = candidates(&job.id).unwrap();
        assert_eq!(paths.len(), 2);
        assert!(original.is_file());
        assert!(matches!(
            DownloadService::get_job(&job.id).unwrap().unwrap().status,
            DownloadStatus::Downloaded
        ));
        assert!(select_candidate(&job.id, "../escape.iso").is_err());
        assert!(select_candidate(&job.id, "original.zip").is_err());
        run(&job, &source, None, &TransferControl::default()).unwrap();
        assert_eq!(candidates(&job.id).unwrap(), paths);
        let selected = select_candidate(&job.id, &paths[1]).unwrap();
        assert!(matches!(selected.status, DownloadStatus::Completed));
        let game = crate::services::GameService::get_game_by_id(job.game_id.clone())
            .unwrap()
            .unwrap();
        assert_eq!(
            game.rom_path,
            Some(destination.join(&paths[1]).to_string_lossy().into_owned())
        );
        fs::write(
            destination.join(".emubox-managed.next"),
            b"interrupted old write",
        )
        .unwrap();
        select_candidate(&job.id, &paths[0]).unwrap();
        status(&job.id, "downloading", "preparing", None).unwrap();
        assert!(select_candidate(&job.id, &paths[1]).is_err());
        status(&job.id, "downloaded", "preparation_required", None).unwrap();
        let replaced = destination.join(&paths[0]);
        fs::remove_file(&replaced).unwrap();
        std::os::unix::fs::symlink(&original, &replaced).unwrap();
        assert!(candidates(&job.id).is_err());
        assert!(select_candidate(&job.id, &paths[0]).is_err());
        fs::remove_dir_all(destination).unwrap();
    }

    #[test]
    #[ignore = "Requires EMUBOX_INNO_FIXTURE official innosetup-6.0.5.exe, innoextract and bubblewrap; never executes extracted code"]
    fn inno_package_is_prepared_selected_and_configured_locally() {
        let _test = QUEUE_TEST.lock().unwrap();
        let fixture = PathBuf::from(std::env::var_os("EMUBOX_INNO_FIXTURE").expect("Inno fixture required"));
        let imported = DownloadService::import_from_json(&serde_json::json!({"downloads":[{"gameId":"inno-integration", "title":"Inno integration", "platform":"pc", "uris":["http://127.0.0.1:1/setup.exe"]}]}).to_string()).unwrap();
        let source = imported[0].clone();
        let job = DownloadService::create_job(crate::models::CreateDownloadRequest { game_id: source.game_id.clone(), platform: "pc".into(), source: source.clone() }).unwrap();
        let destination = Path::new(&job.destination_path);
        fs::create_dir_all(destination).unwrap();
        fs::copy(&fixture, destination.join("setup.exe")).unwrap();
        let package = PublishedDownload { job_id: job.id.clone(), source_digest: format!("{:x}", Sha256::digest(serde_json::to_vec(&source).unwrap())), files: vec!["setup.exe".into()], launch: None, preparation_reason: None, installation: None };
        fs::write(destination.join(".emubox-managed"), serde_json::to_vec(&package).unwrap()).unwrap();
        status(&job.id, "downloaded", "preparation_required", None).unwrap();
        run(&job, &source, None, &TransferControl::default()).unwrap();
        let paths = candidates(&job.id).unwrap();
        let selected = paths.iter().find(|path| path.ends_with("/ISCC.exe")).expect("Extracted compiler candidate");
        assert!(!paths.iter().any(|path| path == "setup.exe"));
        assert_eq!(fs::read(destination.join("setup.exe")).unwrap(), fs::read(fixture).unwrap());
        select_candidate(&job.id, selected).unwrap();
        let mut command = std::process::Command::new("wine");
        crate::services::installer_preparation::configure_launch(&mut command, "wine", &destination.join(selected)).unwrap();
        assert!(destination.join(".wine-prefix").is_dir());
        fs::remove_dir_all(destination).unwrap();
    }

    #[test]
    fn http_jobs_keep_source_snapshot_and_verify_before_publication() {
        let _test = QUEUE_TEST.lock().unwrap();
        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let uri = format!("http://{}/content.dat", server.local_addr().unwrap());
        let worker = thread::spawn(move || {
            for _ in 0..2 {
                let (mut stream, _) = server.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                let mut buffer = [0u8; 4096];
                stream.read(&mut buffer).unwrap();
                stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\nETag: \"v1\"\r\nConnection: close\r\n\r\ndata").unwrap();
            }
        });
        for (index, checksum) in [format!("{:x}", Sha256::digest(b"data")), "0".repeat(64)]
            .into_iter()
            .enumerate()
        {
            let game_id = format!("manager-http-{index}");
            let imported=DownloadService::import_from_json(&serde_json::json!({"downloads":[{"gameId":game_id,"title":game_id,"platform":"ps2","uris":[uri],"checksum":checksum}]}).to_string()).unwrap();
            let source = imported[0].clone();
            let job = DownloadService::create_job(crate::models::CreateDownloadRequest {
                game_id: game_id.clone(),
                platform: "ps2".into(),
                source: source.clone(),
            })
            .unwrap();
            let mut changed = source;
            changed.uri = "http://127.0.0.1:1/changed".into();
            DownloadService::create_source(changed).unwrap();
            start(job.id.clone()).unwrap();
            let deadline = Instant::now() + Duration::from_secs(10);
            let finished = loop {
                let current = DownloadService::get_job(&job.id).unwrap().unwrap();
                if matches!(
                    current.status,
                    DownloadStatus::Downloaded | DownloadStatus::Failed
                ) {
                    break current;
                }
                assert!(Instant::now() < deadline, "job did not finish");
                thread::sleep(Duration::from_millis(10));
            };
            if index == 0 {
                assert!(matches!(finished.status, DownloadStatus::Downloaded));
                assert_eq!(finished.provider.as_deref(), Some("http"));
                assert_eq!(
                    fs::read(Path::new(&finished.destination_path).join("content.dat")).unwrap(),
                    b"data"
                );
                assert!(Path::new(&finished.destination_path)
                    .join(".emubox-managed")
                    .is_file());
                assert!(
                    !crate::services::GameService::get_game_by_id(game_id)
                        .unwrap()
                        .unwrap()
                        .installed
                );
                assert!(matches!(
                    start(job.id).unwrap().status,
                    DownloadStatus::Downloaded
                ));
            } else {
                assert!(matches!(finished.status, DownloadStatus::Failed));
                assert!(!Path::new(&finished.destination_path).exists());
            }
        }
        worker.join().unwrap();
    }

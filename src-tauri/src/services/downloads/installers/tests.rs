
    use super::*;
use super::{detection::windows_executable, sandbox::sandbox};
use std::process::{Command, Stdio};

    #[test]
    fn discovers_firmware_in_managed_environment_and_honors_override() {
        let root = std::env::temp_dir().join(format!("emubox-firmware-path-{}", std::process::id()));
        let flash = root.join("rpcs3/dev_flash");
        fs::create_dir_all(flash.join("sys/external")).unwrap();
        assert!(firmware_directory(None, &root).is_err());
        fs::write(flash.join("sys/external/liblv2.sprx"), b"fixture").unwrap();
        assert_eq!(firmware_directory(None, &root).unwrap(), flash);
        assert!(firmware_directory(Some(root.join("missing")), &root).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn recognizes_installer_content_not_filename() {
        assert_eq!(kind(b"MZexample"), Some(InstallerKind::Inno));
        assert_eq!(kind(b"\x7fPKGexample"), Some(InstallerKind::Ps3));
        assert_eq!(kind(b"not a package.exe"), None);
    }

    #[test]
    fn prepared_windows_requires_pe_and_explicit_selection() {
        let root =
            std::env::temp_dir().join(format!("emubox-inno-candidates-{}", std::process::id()));
        fs::create_dir_all(root.join("extracted/app")).unwrap();
        let mut pe = vec![0u8; 88];
        pe[..2].copy_from_slice(b"MZ");
        pe[60..64].copy_from_slice(&64u32.to_le_bytes());
        pe[64..68].copy_from_slice(b"PE\0\0");
        pe[68..70].copy_from_slice(&0x8664u16.to_le_bytes());
        fs::write(root.join("extracted/app/game.exe"), &pe).unwrap();
        fs::write(root.join("extracted/app/setup.exe"), &pe).unwrap();
        fs::write(root.join("extracted/app/fake.exe"), b"not PE").unwrap();
        let mut package = crate::models::PublishedDownload {
            job_id: "test".into(),
            source_digest: "test".into(),
            files: vec![
                "extracted/app/game.exe".into(),
                "extracted/app/setup.exe".into(),
                "extracted/app/fake.exe".into(),
            ],
            launch: None,
            preparation_reason: None,
            installation: Some(crate::models::PreparedInstallation {
                kind: InstallerKind::Inno,
                root: "extracted".into(),
            }),
        };
        assert_eq!(
            candidates("pc", &root, &package),
            ["extracted/app/game.exe"]
        );
        assert!(candidates("ps2", &root, &package).is_empty());
        fs::write(
            root.join(".emubox-managed"),
            serde_json::to_vec(&package).unwrap(),
        )
        .unwrap();
        assert!(configure_launch(
            &mut Command::new("wine"),
            "wine",
            &root.join("extracted/app/game.exe")
        )
        .is_err());
        package.launch = Some("extracted/app/game.exe".into());
        fs::write(
            root.join(".emubox-managed"),
            serde_json::to_vec(&package).unwrap(),
        )
        .unwrap();
        let mut command = Command::new("wine");
        configure_launch(&mut command, "wine", &root.join("extracted/app/game.exe")).unwrap();
        assert!(command
            .get_envs()
            .any(|(key, value)| key == "WINEDLLOVERRIDES"
                && value == Some(std::ffi::OsStr::new("mscoree,mshtml="))));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_links_in_prepared_content() {
        let root =
            std::env::temp_dir().join(format!("emubox-installer-links-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        std::os::unix::fs::symlink("/etc/passwd", root.join("link")).unwrap();
        assert!(inventory(&root).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn pkg_confirmation_and_candidates_require_installation_evidence() {
        let root = std::env::temp_dir().join(format!("emubox-pkg-evidence-{}", std::process::id()));
        let title = PathBuf::from("extracted/home/config/rpcs3/dev_hdd0/game/TEST00001");
        let boot = title.join("USRDIR/EBOOT.BIN");
        let sfo = title.join("PARAM.SFO");
        fs::create_dir_all(root.join(boot.parent().unwrap())).unwrap();
        fs::write(root.join(&boot), b"SCE\0test").unwrap();
        fs::write(root.join(&sfo), b"\0PSFtest").unwrap();
        let package = crate::models::PublishedDownload {
            job_id: "test".into(),
            source_digest: String::new(),
            files: vec![boot.clone(), sfo.clone()],
            launch: Some(boot.clone()),
            preparation_reason: None,
            installation: Some(crate::models::PreparedInstallation {
                kind: InstallerKind::Ps3,
                root: "extracted".into(),
            }),
        };
        assert_eq!(
            candidates("ps3", &root, &package),
            [boot.to_string_lossy().into_owned()]
        );
        assert!(candidates("pc", &root, &package).is_empty());
        let log = root.join("extracted/home/config/rpcs3/RPCS3.log");
        fs::write(&log, "GUI: Aborted installation of /input/package.pkg").unwrap();
        assert!(!pkg_succeeded(&root.join("extracted"), &[log.clone()]));
        fs::write(&log, "S GUI: Successfully installed /input/package.pkg (title_id=TEST00001, title=Test, version=1).").unwrap();
        assert!(pkg_succeeded(&root.join("extracted"), &[log]));
        fs::write(
            root.join(".emubox-managed"),
            serde_json::to_vec(&package).unwrap(),
        )
        .unwrap();
        assert!(
            configure_launch(&mut Command::new("rpcs3"), "rpcs3", &root.join(&boot))
                .unwrap_err()
                .to_string()
                .contains("firmware")
        );
        fs::write(root.join(&sfo), b"bad SFO").unwrap();
        assert!(candidates("ps3", &root, &package).is_empty());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[ignore = "Requires bubblewrap, prlimit and innoextract installed; isolated local invalid EXE only"]
    fn real_innoextract_rejects_invalid_exe_without_changing_original() {
        let root =
            std::env::temp_dir().join(format!("emubox-installer-invalid-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let source = root.join("input.exe");
        fs::write(&source, b"MZ invalid installer").unwrap();
        assert!(prepare(
            &source,
            &root,
            InstallerKind::Inno,
            &TransferControl::default()
        )
        .is_err());
        assert_eq!(fs::read(&source).unwrap(), b"MZ invalid installer");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[ignore = "Requires EMUBOX_INNO_FIXTURE pointing to official innosetup-6.0.5.exe and installed sandbox tools"]
    fn real_inno_installer_extracts_without_execution() {
        let source =
            PathBuf::from(std::env::var_os("EMUBOX_INNO_FIXTURE").expect("Inno fixture required"));
        let root = std::env::temp_dir().join(format!("emubox-inno-real-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let files = prepare(
            &source,
            &root,
            InstallerKind::Inno,
            &TransferControl::default(),
        )
        .unwrap();
        let compiler = files
            .iter()
            .find(|path| path.file_name().is_some_and(|name| name == "ISCC.exe"))
            .expect("Inno compiler must have been extracted");
        assert!(windows_executable(compiler));
        let installation = prepared_metadata(&[source], &files, &root).unwrap();
        let package = crate::models::PublishedDownload {
            job_id: "inno-fixture".into(),
            source_digest: String::new(),
            files: files
                .iter()
                .map(|path| path.strip_prefix(&root).unwrap().into())
                .collect(),
            launch: None,
            preparation_reason: None,
            installation: Some(installation),
        };
        assert!(!candidates("pc", &root, &package).is_empty());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[ignore = "Requires official RPCS3 AppImage, bubblewrap and prlimit; no games or firmware"]
    fn real_rpcs3_starts_in_installation_sandbox() {
        let root =
            std::env::temp_dir().join(format!("emubox-rpcs3-sandbox-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let source = root.join("invalid.pkg");
        fs::write(&source, b"\x7fPKG invalid test package").unwrap();
        let tool = Path::new("/opt/emubox/bin/RPCS3.AppImage");
        let mut command = sandbox(&source, &root, tool, "/input/package.pkg").unwrap();
        command
            .args(["--headless", "--version"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("RPCS3 0."));
        let produced = inventory(&root).unwrap();
        assert!(
            produced
                .iter()
                .any(|path| path.file_name().is_some_and(|name| name == "RPCS3.log")),
            "RPCS3 must leave its log: {produced:?}"
        );
        let failed = prepare(
            &source,
            &root,
            InstallerKind::Ps3,
            &TransferControl::default(),
        )
        .unwrap_err()
        .to_string();
        assert!(
            failed.contains("firmware")
                || failed.contains("no confirmo")
                || failed.contains("rechazo"),
            "{failed}"
        );
        assert_eq!(fs::read(&source).unwrap(), b"\x7fPKG invalid test package");
        fs::remove_dir_all(root).unwrap();
    }

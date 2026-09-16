use super::{event, Config};
use log::Level;
use serde_json::{json, Value};
use std::{
    fs::{self, File},
    io::{Read, Seek, SeekFrom},
    os::unix::fs::MetadataExt,
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};

pub(super) fn start(config: Config) {
    if config.collect_session {
        std::thread::spawn(session);
    }
    if config.collect_journal {
        let priority = config.journal_priority.clone();
        std::thread::spawn(move || journal(priority));
    }
    std::thread::spawn(move || processes(config.process_interval_seconds));
}

fn session() {
    let path = PathBuf::from(crate::services::paths::LOG_DIR).join("session.log");
    let mut reader = SessionReader::default();
    let mut reported_error = false;
    loop {
        match reader.read(&path) {
            Ok(lines) => {
                reported_error = false;
                for line in lines {
                    if line.starts_with("[Telemetry]") {
                        continue;
                    }
                    let (level, source) = classify_session(&line);
                    event(
                        level,
                        source,
                        "session.line",
                        &line,
                        json!({"timestampKind": "observed", "path": path}),
                    );
                }
            }
            Err(error) if !reported_error => {
                event(
                    Level::Warn,
                    "telemetry.session",
                    "collector.unavailable",
                    &error.to_string(),
                    Value::Null,
                );
                reported_error = true;
            }
            Err(_) => (),
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}

#[derive(Default)]
struct SessionReader {
    identity: Option<(u64, u64)>,
    offset: u64,
    pending: String,
}

impl SessionReader {
    fn read(&mut self, path: &std::path::Path) -> std::io::Result<Vec<String>> {
        let mut file = File::open(path)?;
        let metadata = file.metadata()?;
        let identity = (metadata.dev(), metadata.ino());
        if self.identity != Some(identity) || metadata.len() < self.offset {
            self.offset = if self.identity.is_none() {
                metadata.len()
            } else {
                0
            };
            self.pending.clear();
            self.identity = Some(identity);
        }
        file.seek(SeekFrom::Start(self.offset))?;
        let mut bytes = Vec::new();
        file.take(256 * 1024).read_to_end(&mut bytes)?;
        self.offset += bytes.len() as u64;
        self.pending.push_str(&String::from_utf8_lossy(&bytes));
        let mut lines = Vec::new();
        if let Some(last) = self.pending.rfind('\n') {
            for line in self.pending[..last].lines() {
                lines.push(line.chars().take(4096).collect());
            }
            self.pending.drain(..=last);
        }
        if self.pending.len() > 16384 {
            lines.push("[WARN] Linea de sesion truncada por superar 16 KiB".into());
            self.pending.clear();
        }
        Ok(lines)
    }
}

fn classify_session(line: &str) -> (Level, &'static str) {
    let lower = line.to_ascii_lowercase();
    let level = if ["error", "failed", "panic", "fatal", "fallo", "agotad"]
        .iter()
        .any(|word| lower.contains(word))
    {
        Level::Error
    } else if ["warn", "aviso"].iter().any(|word| lower.contains(word)) {
        Level::Warn
    } else if lower.contains("debug") {
        Level::Debug
    } else {
        Level::Info
    };
    let source = if lower.contains("webkit") {
        "session.webkit"
    } else if lower.contains("vmwgfx") || lower.contains("[backend/") || lower.contains("[render/")
    {
        "session.graphics"
    } else if line.starts_with("[Catalog]") {
        "session.catalog"
    } else if line.starts_with("[Startup]") {
        "session.startup"
    } else {
        "session.stderr"
    };
    (level, source)
}

fn journal(priority: String) {
    let mut cursor = None::<String>;
    let mut reported_error = false;
    loop {
        let mut command = Command::new("/usr/bin/timeout");
        command.args(["--kill-after=1s", "4s", "/usr/bin/journalctl", "--boot", "--no-pager", "--quiet",
			"--output=json", "--lines=200", "--priority"]).arg(&priority)
			.arg("--output-fields=MESSAGE,PRIORITY,SYSLOG_IDENTIFIER,_COMM,_PID,_SYSTEMD_UNIT,__REALTIME_TIMESTAMP,__MONOTONIC_TIMESTAMP,_BOOT_ID")
			.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null());
        if let Some(cursor) = &cursor {
            command.arg(format!("--after-cursor={cursor}"));
        }
        let result = (|| -> std::io::Result<String> {
            let mut child = command.spawn()?;
            let mut bytes = Vec::new();
            let read = child
                .stdout
                .take()
                .ok_or_else(|| std::io::Error::other("Journal sin salida"))?
                .take(1024 * 1024)
                .read_to_end(&mut bytes);
            let status = child.wait()?;
            read?;
            if !status.success() || bytes.len() >= 1024 * 1024 {
                return Err(std::io::Error::other(
                    "Journal no accesible, agotado o lectura >1 MiB",
                ));
            }
            Ok(String::from_utf8_lossy(&bytes).into_owned())
        })();
        match result {
            Ok(text) => {
                reported_error = false;
                if text.lines().count() >= 200 {
                    event(
                        Level::Warn,
                        "telemetry.journal",
                        "collector.limit",
                        "Limite de 200 entradas; puede haber huecos",
                        Value::Null,
                    );
                }
                for line in text.lines() {
                    let Ok(entry) = serde_json::from_str::<Value>(line) else {
                        continue;
                    };
                    if let Some(value) = entry["__CURSOR"].as_str() {
                        cursor = Some(value.into());
                    }
                    record_journal(&entry);
                }
            }
            Err(error) if !reported_error => {
                event(
                    Level::Warn,
                    "telemetry.journal",
                    "collector.unavailable",
                    &error.to_string(),
                    Value::Null,
                );
                reported_error = true;
            }
            Err(_) => (),
        }
        std::thread::sleep(Duration::from_secs(5));
    }
}

fn record_journal(entry: &Value) {
    let priority = entry["PRIORITY"]
        .as_str()
        .and_then(|value| value.parse::<u8>().ok());
    let level = match priority {
        Some(0..=3) => Level::Error,
        Some(4) => Level::Warn,
        Some(7) => Level::Debug,
        _ => Level::Info,
    };
    let source = entry["SYSLOG_IDENTIFIER"]
        .as_str()
        .or(entry["_COMM"].as_str())
        .unwrap_or("unknown");
    let message = entry["MESSAGE"]
        .as_str()
        .unwrap_or("[non-text journal message]");
    event(
        level,
        &format!("os.{source}"),
        "journal.message",
        message,
        json!({
            "originTimestampUs": entry["__REALTIME_TIMESTAMP"], "originMonotonicUs": entry["__MONOTONIC_TIMESTAMP"],
            "originPid": entry["_PID"], "originBootId": entry["_BOOT_ID"], "unit": entry["_SYSTEMD_UNIT"], "priority": priority
        }),
    );
}

fn processes(interval: u64) {
    let mut system = sysinfo::System::new();
    loop {
        let started = std::time::Instant::now();
        let pids: Vec<_> = fs::read_dir("/proc")
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|entry| {
                entry
                    .file_name()
                    .to_str()?
                    .parse::<u32>()
                    .ok()
                    .map(sysinfo::Pid::from_u32)
            })
            .collect();
        system.refresh_pids_specifics(
            &pids,
            sysinfo::ProcessRefreshKind::new().with_cpu().with_memory(),
        );
        system.refresh_memory();
        let mut entries: Vec<_> = pids.iter().filter_map(|pid| system.process(*pid)).collect();
        entries.sort_by(|left, right| right.cpu_usage().total_cmp(&left.cpu_usage()));
        let samples: Vec<_> = entries.iter().enumerate().filter(|(index, process)| *index < 6 ||
			["emubox", "WebKit", "cage", "gamescope", "ld-linux", "pipewire", "wireplumber", "Xwayland"].iter().any(|name| process.name().contains(name)))
			.take(30).map(|(_, process)| json!({"pid": process.pid().as_u32(), "parentPid": process.parent().map(|pid| pid.as_u32()),
				"name": process.name(), "cpuPercent": process.cpu_usage(), "residentBytes": process.memory(), "status": format!("{:?}", process.status())})).collect();
        let pressure = ["cpu", "memory", "io"].map(|kind| {
            (
                kind,
                fs::read_to_string(format!("/proc/pressure/{kind}")).unwrap_or_default(),
            )
        });
        event(
            Level::Info,
            "system.resources",
            "process.sample",
            "Muestra de procesos y recursos",
            json!({
                "intervalSeconds": interval, "memoryAvailableBytes": system.available_memory(), "swapUsedBytes": system.used_swap(),
                "processes": samples, "pressure": pressure, "sampleDurationMs": started.elapsed().as_millis() as u64,
                "processScope": "process-leaders"
            }),
        );
        std::thread::sleep(Duration::from_secs(interval));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn resource_refresh_excludes_thread_rows() {
        let pid = sysinfo::Pid::from_u32(std::process::id());
        let mut system = sysinfo::System::new();
        system.refresh_pids_specifics(
            &[pid],
            sysinfo::ProcessRefreshKind::new().with_cpu().with_memory(),
        );
        assert!(system.process(pid).is_some());
        assert_eq!(system.processes().len(), 1);
    }

    #[test]
    fn session_reader_reads_only_new_complete_lines_and_recovers_rotation() {
        let root = std::env::temp_dir().join(format!("emubox-tail-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let path = root.join("session.log");
        fs::write(&path, "old\n").unwrap();
        let mut reader = SessionReader::default();
        assert!(reader.read(&path).unwrap().is_empty());
        let mut file = fs::OpenOptions::new().append(true).open(&path).unwrap();
        file.write_all(b"[WARN] part").unwrap();
        assert!(reader.read(&path).unwrap().is_empty());
        file.write_all(b"ial\n").unwrap();
        assert_eq!(reader.read(&path).unwrap(), ["[WARN] partial"]);
        assert!(reader.read(&path).unwrap().is_empty());
        drop(file);
        fs::rename(&path, root.join("previous")).unwrap();
        fs::write(&path, "new session\n").unwrap();
        assert_eq!(reader.read(&path).unwrap(), ["new session"]);
        assert_eq!(
            classify_session("[ERROR] vmwgfx failure"),
            (Level::Error, "session.graphics")
        );
        fs::remove_dir_all(root).unwrap();
    }
}

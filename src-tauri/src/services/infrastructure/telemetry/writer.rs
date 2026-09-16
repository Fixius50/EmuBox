use super::Entry;
use serde_json::json;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    os::unix::fs::OpenOptionsExt,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::Receiver,
        Arc,
    },
};

pub(super) struct RotatingWriter {
    path: PathBuf,
    file: Option<File>,
    bytes: u64,
    max: u64,
    retained: usize,
    _lock: File,
}

impl RotatingWriter {
    pub(super) fn open(path: PathBuf, max: u64, retained: usize) -> io::Result<Self> {
        let lock = Self::file(&path.with_extension("lock"))?;
        lock.try_lock().map_err(io::Error::other)?;
        let file = Self::file(&path)?;
        let bytes = file.metadata()?.len();
        Ok(Self {
            path,
            file: Some(file),
            bytes,
            max,
            retained,
            _lock: lock,
        })
    }
    fn file(path: &PathBuf) -> io::Result<File> {
        if fs::symlink_metadata(path)
            .is_ok_and(|metadata| !metadata.is_file() || metadata.file_type().is_symlink())
        {
            return Err(io::Error::other("Unsafe log file"));
        }
        OpenOptions::new()
            .create(true)
            .append(true)
            .mode(0o600)
            .open(path)
    }
    fn backup(&self, index: usize) -> PathBuf {
        self.path.with_extension(format!("jsonl.{index}"))
    }
    pub(super) fn write(&mut self, data: &[u8]) -> io::Result<()> {
        if self.bytes > 0 && self.bytes + data.len() as u64 > self.max {
            self.file.take();
            for index in (1..self.retained).rev() {
                let previous = self.backup(index);
                if previous.exists() {
                    fs::rename(previous, self.backup(index + 1))?;
                }
            }
            fs::rename(&self.path, self.backup(1))?;
            self.file = Some(Self::file(&self.path)?);
            self.bytes = 0;
        }
        self.file
            .as_mut()
            .ok_or_else(|| io::Error::other("Log writer unavailable"))?
            .write_all(data)?;
        self.bytes += data.len() as u64;
        Ok(())
    }
}

pub(super) fn run(mut writer: RotatingWriter, receiver: Receiver<Entry>, dropped: Arc<AtomicU64>) {
    while let Ok(entry) = receiver.recv() {
        let lost = dropped.swap(0, Ordering::Relaxed);
        let value = if lost > 0 {
            json!({"timestamp": entry.timestamp, "timestampMs": entry.timestamp_ms,
            "elapsedMs": entry.elapsed_ms, "sessionId": entry.session_id, "bootId": entry.boot_id,
            "level": "warn", "source": "telemetry", "event": "events.dropped", "data": {"count": lost, "reason": "queue_or_rate_limit"}})
        } else {
            serde_json::Value::Null
        };
        for value in [value, serde_json::to_value(entry).unwrap_or_default()] {
            if value.is_null() {
                continue;
            }
            let mut bytes = serde_json::to_vec(&value).unwrap_or_default();
            bytes.push(b'\n');
            if let Err(error) = writer.write(&bytes) {
                eprintln!("[Telemetry] Escritor detenido: {error}");
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rotation_is_bounded_and_preserves_complete_lines() {
        let root = std::env::temp_dir().join(format!("emubox-log-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let path = root.join("events.jsonl");
        let mut writer = RotatingWriter::open(path.clone(), 8, 2).unwrap();
        assert!(RotatingWriter::open(path.clone(), 8, 2).is_err());
        for _ in 0..8 {
            writer.write(b"{\"a\":1}\n").unwrap();
        }
        assert_eq!(fs::read_dir(&root).unwrap().count(), 4);
        for entry in fs::read_dir(&root).unwrap() {
            let entry = entry.unwrap();
            if entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "lock")
            {
                continue;
            }
            let content = fs::read_to_string(entry.path()).unwrap();
            assert!(content.ends_with('\n'));
            for line in content.lines() {
                assert!(serde_json::from_str::<serde_json::Value>(line).is_ok());
            }
        }
        drop(writer);
        fs::remove_dir_all(root).unwrap();
    }
}

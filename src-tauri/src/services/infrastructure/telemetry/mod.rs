mod collectors;
mod writer;

use crate::services::paths;
use log::{Level, LevelFilter, Log, Metadata, Record};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{sync_channel, SyncSender},
        Mutex, OnceLock,
    },
    time::Instant,
};

static LOGGER: OnceLock<Controller> = OnceLock::new();

#[derive(Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
    enabled: bool,
    level: String,
    sources: BTreeMap<String, String>,
    max_file_bytes: u64,
    retained_files: usize,
    collect_session: bool,
    collect_journal: bool,
    journal_priority: String,
    process_interval_seconds: u64,
    events_per_second: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: true,
            level: "info".into(),
            sources: BTreeMap::new(),
            max_file_bytes: 8 * 1024 * 1024,
            retained_files: 4,
            collect_session: true,
            collect_journal: true,
            journal_priority: "warning".into(),
            process_interval_seconds: 5,
            events_per_second: 200,
        }
    }
}

impl Config {
    fn validate(&self) -> Result<(), String> {
        for level in std::iter::once(&self.level).chain(self.sources.values()) {
            level
                .parse::<LevelFilter>()
                .map_err(|_| format!("Nivel de registro invalido: {level}"))?;
        }
        if !(65536..=64 * 1024 * 1024).contains(&self.max_file_bytes)
            || !(1..=8).contains(&self.retained_files)
            || !(2..=60).contains(&self.process_interval_seconds)
            || !(10..=1000).contains(&self.events_per_second)
            || !["err", "warning", "notice", "info", "debug"]
                .contains(&self.journal_priority.as_str())
        {
            return Err("Limites de telemetria invalidos".into());
        }
        Ok(())
    }

    fn accepts(&self, source: &str, level: Level) -> bool {
        let threshold = self
            .sources
            .iter()
            .filter(|(prefix, _)| source.starts_with(prefix.as_str()))
            .max_by_key(|(prefix, _)| prefix.len())
            .map(|(_, level)| level)
            .unwrap_or(&self.level);
        self.enabled
            && level
                <= threshold
                    .parse::<LevelFilter>()
                    .unwrap_or(LevelFilter::Info)
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Entry {
    timestamp: String,
    timestamp_ms: i64,
    elapsed_ms: u64,
    session_id: String,
    boot_id: String,
    sequence: u64,
    level: String,
    source: String,
    event: String,
    pid: u32,
    thread: String,
    message: String,
    data: Value,
}

struct Controller {
    config: Config,
    started: Instant,
    session_id: String,
    boot_id: String,
    sender: SyncSender<Entry>,
    sequence: AtomicU64,
    dropped: std::sync::Arc<AtomicU64>,
    rate: Mutex<RateWindow>,
}

#[derive(Default)]
struct RateWindow {
    second: u64,
    count: u64,
}

impl RateWindow {
    fn admit(&mut self, second: u64, limit: u64) -> bool {
        if self.second != second {
            self.second = second;
            self.count = 0;
        }
        if self.count >= limit {
            return false;
        }
        self.count += 1;
        true
    }
}

pub fn init() {
    if LOGGER.get().is_some() {
        return;
    }
    let configured = format!("{}/logging.json", paths::CONFIG_DIR);
    let config_path = if std::path::Path::new(&configured).exists() {
        configured
    } else {
        "/opt/emubox/data/logging.json".into()
    };
    let config = fs::read(&config_path)
        .map_err(|error| error.to_string())
        .and_then(|bytes| {
            serde_json::from_slice::<Config>(&bytes).map_err(|error| error.to_string())
        })
        .and_then(|config| config.validate().map(|_| config));
    let (config, warning) = match config {
        Ok(config) => (config, None),
        Err(error) => (Config::default(), Some(error)),
    };
    if !config.enabled {
        return;
    }
    let destination = PathBuf::from(paths::LOG_DIR).join("events.jsonl");
    let writer = match writer::RotatingWriter::open(
        destination,
        config.max_file_bytes,
        config.retained_files,
    ) {
        Ok(writer) => writer,
        Err(error) => {
            eprintln!("[Telemetry] No se puede abrir events.jsonl: {error}");
            return;
        }
    };
    let now = chrono::Utc::now();
    let (sender, receiver) = sync_channel(1024);
    let dropped = std::sync::Arc::new(AtomicU64::new(0));
    let controller = Controller {
        config: config.clone(),
        started: Instant::now(),
        session_id: format!("{}-{}", now.timestamp_millis(), std::process::id()),
        boot_id: fs::read_to_string("/proc/sys/kernel/random/boot_id")
            .unwrap_or_default()
            .trim()
            .into(),
        sender,
        sequence: AtomicU64::new(0),
        dropped: dropped.clone(),
        rate: Mutex::new(RateWindow::default()),
    };
    if LOGGER.set(controller).is_err() {
        return;
    }
    std::thread::Builder::new()
        .name("telemetry-writer".into())
        .spawn(move || writer::run(writer, receiver, dropped))
        .expect("telemetry writer thread");
    if let Err(error) = log::set_logger(LOGGER.get().unwrap()) {
        event(
            Level::Warn,
            "telemetry",
            "logger.conflict",
            &error.to_string(),
            Value::Null,
        );
    }
    log::set_max_level(LevelFilter::Trace);
    event(
        Level::Info,
        "telemetry",
        "session.start",
        "Registro estructurado iniciado",
        json!({"config": config_path, "maxFileBytes": config.max_file_bytes, "retainedFiles": config.retained_files}),
    );
    if let Some(error) = warning {
        event(
            Level::Warn,
            "telemetry",
            "config.defaults",
            "Se usan valores por defecto de registro",
            json!({"reason": error}),
        );
    }
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        event(
            Level::Error,
            "rust.panic",
            "panic",
            &panic.to_string(),
            Value::Null,
        );
        previous(panic);
    }));
    collectors::start(config);
}

impl Controller {
    fn record(&self, level: Level, source: &str, name: &str, message: &str, data: Value) {
        if !self.config.accepts(source, level) {
            return;
        }
        if !self
            .rate
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .admit(
                self.started.elapsed().as_secs(),
                self.config.events_per_second,
            )
        {
            self.dropped.fetch_add(1, Ordering::Relaxed);
            return;
        }
        let now = chrono::Utc::now();
        let entry = Entry {
            timestamp: now.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            timestamp_ms: now.timestamp_millis(),
            elapsed_ms: self.started.elapsed().as_millis() as u64,
            session_id: self.session_id.clone(),
            boot_id: self.boot_id.clone(),
            sequence: self.sequence.fetch_add(1, Ordering::Relaxed),
            level: level.as_str().to_lowercase(),
            source: clean_text(source),
            event: clean_text(name),
            pid: std::process::id(),
            thread: format!("{:?}", std::thread::current().id()),
            message: clean_text(message),
            data: clean_value(data, 0),
        };
        if self.sender.try_send(entry).is_err() {
            self.dropped.fetch_add(1, Ordering::Relaxed);
        }
    }
}

impl Log for Controller {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        self.config.accepts(metadata.target(), metadata.level())
    }
    fn log(&self, record: &Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }
        self.record(
            record.level(),
            record.target(),
            "rust.log",
            &record.args().to_string(),
            json!({"file": record.file(), "line": record.line(), "module": record.module_path()}),
        );
    }
    fn flush(&self) {}
}

pub fn event(level: Level, source: &str, name: &str, message: &str, data: Value) {
    if let Some(controller) = LOGGER.get() {
        controller.record(level, source, name, message, data);
    }
}

fn sensitive(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    [
        "password",
        "passwd",
        "token",
        "secret",
        "authorization",
        "cookie",
        "credential",
        "private key",
        "api_key",
        "apikey",
    ]
    .iter()
    .any(|key| value.contains(key))
}

fn clean_text(text: &str) -> String {
    if sensitive(text) {
        return "[redacted]".into();
    }
    text.split_whitespace()
        .map(|word| {
            if word.contains("://") || word.starts_with("magnet:") {
                "[url]"
            } else if word.contains("/home/") {
                "[home-path]"
            } else {
                word
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(2048)
        .collect()
}

fn clean_value(value: Value, depth: usize) -> Value {
    if depth > 4 {
        return Value::String("[depth-limit]".into());
    }
    match value {
        Value::String(text) => Value::String(clean_text(&text)),
        Value::Object(fields) => Value::Object(
            fields
                .into_iter()
                .take(32)
                .map(|(key, value)| {
                    let value = if sensitive(&key) {
                        Value::String("[redacted]".into())
                    } else {
                        clean_value(value, depth + 1)
                    };
                    (key.chars().take(80).collect(), value)
                })
                .collect(),
        ),
        Value::Array(values) => Value::Array(
            values
                .into_iter()
                .take(32)
                .map(|value| clean_value(value, depth + 1))
                .collect(),
        ),
        other => other,
    }
}

pub struct Span {
    source: &'static str,
    name: &'static str,
    id: u64,
    started: Instant,
}
static SPAN_ID: AtomicU64 = AtomicU64::new(0);

pub fn span(source: &'static str, name: &'static str) -> Span {
    let id = SPAN_ID.fetch_add(1, Ordering::Relaxed);
    event(
        Level::Info,
        source,
        "operation.start",
        name,
        json!({"operationId": id}),
    );
    Span {
        source,
        name,
        id,
        started: Instant::now(),
    }
}

pub fn operation<T, E: std::fmt::Display>(
    source: &'static str,
    name: &'static str,
    action: impl FnOnce() -> Result<T, E>,
) -> Result<T, E> {
    let span = span(source, name);
    let result = action();
    event(
        if result.is_ok() {
            Level::Info
        } else {
            Level::Error
        },
        source,
        "operation.result",
        name,
        json!({"operationId": span.id, "success": result.is_ok(), "error": result.as_ref().err().map(ToString::to_string)}),
    );
    result
}

impl Drop for Span {
    fn drop(&mut self) {
        event(
            Level::Info,
            self.source,
            "operation.end",
            self.name,
            json!({"operationId": self.id, "durationMs": self.started.elapsed().as_millis() as u64, "panicking": std::thread::panicking()}),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_uses_longest_source_prefix_and_validates_limits() {
        let mut config = Config::default();
        config.sources.insert("ui".into(), "warn".into());
        config.sources.insert("ui.input".into(), "off".into());
        assert!(!config.accepts("ui.render", Level::Info));
        assert!(config.accepts("ui.render", Level::Error));
        assert!(!config.accepts("ui.input", Level::Error));
        assert!(config.accepts("startup", Level::Info));
        config.max_file_bytes = 1;
        assert!(config.validate().is_err());
    }

    #[test]
    fn rate_limit_resets_without_unbounded_source_tracking() {
        let mut rate = RateWindow::default();
        assert!(rate.admit(0, 2));
        assert!(rate.admit(0, 2));
        assert!(!rate.admit(0, 2));
        assert!(rate.admit(1, 2));
    }

    #[test]
    fn redacts_sensitive_fields_and_urls_and_bounds_messages() {
        let cleaned = clean_value(
            json!({"token": "value", "url": "https://host/private?value=123", "message": "password=hidden", "durationMs": 42}),
            0,
        );
        assert_eq!(cleaned["token"], "[redacted]");
        assert_eq!(cleaned["url"], "[url]");
        assert_eq!(cleaned["durationMs"], 42);
        assert!(!cleaned.to_string().contains("hidden"));
        assert_eq!(clean_text(&"x".repeat(4000)).len(), 2048);
    }

    #[test]
    fn complete_pipeline_writes_timestamp_origin_and_redacted_data() {
        let root =
            std::env::temp_dir().join(format!("emubox-telemetry-pipeline-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let path = root.join("events.jsonl");
        let writer = writer::RotatingWriter::open(path.clone(), 65536, 1).unwrap();
        let (sender, receiver) = sync_channel(8);
        let dropped = std::sync::Arc::new(AtomicU64::new(0));
        let controller = Controller {
            config: Config::default(),
            started: Instant::now(),
            session_id: "fixture-session".into(),
            boot_id: "fixture-boot".into(),
            sender,
            sequence: AtomicU64::new(0),
            dropped: dropped.clone(),
            rate: Mutex::new(RateWindow::default()),
        };
        controller.record(
            Level::Info,
            "startup.task",
            "operation.start",
            "fixture",
            json!({"operationId": 1, "token": "hidden-value"}),
        );
        controller.record(
            Level::Debug,
            "startup.task",
            "filtered",
            "not written",
            Value::Null,
        );
        controller.record(Level::Warn, "ui", "incident.mark", "fixture", Value::Null);
        drop(controller);
        writer::run(writer, receiver, dropped);
        let content = fs::read_to_string(&path).unwrap();
        let entries: Vec<Value> = content
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0]["sessionId"], "fixture-session");
        assert_eq!(entries[0]["source"], "startup.task");
        assert!(entries[0]["timestampMs"].as_i64().unwrap() > 0);
        assert!(entries[0]["elapsedMs"].is_u64());
        assert!(entries[0]["pid"].is_u64());
        assert!(entries[0]["thread"].is_string());
        assert!(!content.contains("hidden-value"));
        assert_eq!(entries[1]["level"], "warn");
        fs::remove_dir_all(root).unwrap();
    }
}

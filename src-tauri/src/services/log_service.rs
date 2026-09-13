use crate::{errors::EmuBoxError, models::LogEntry};

pub fn system(limit: Option<usize>) -> Result<Vec<LogEntry>, EmuBoxError> {
    let limit = limit.unwrap_or(100).min(1000).to_string();
    let output = super::host_command::output(
        "journalctl",
        &["--no-pager", "--quiet", "--output=json", "-n", &limit],
    )?;
    output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let entry: serde_json::Value = serde_json::from_str(line)
                .map_err(|error| EmuBoxError::ProcessFailed(error.to_string()))?;
            let priority = entry["PRIORITY"]
                .as_str()
                .and_then(|value| value.parse::<u8>().ok());
            Ok(LogEntry {
                timestamp: entry["__REALTIME_TIMESTAMP"]
                    .as_str()
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(|value| value / 1000),
                level: match priority {
                    Some(0..=3) => "error",
                    Some(4) => "warn",
                    Some(5..=6) => "info",
                    Some(7) => "debug",
                    _ => "unknown",
                }
                .into(),
                source: entry["SYSLOG_IDENTIFIER"]
                    .as_str()
                    .or(entry["_COMM"].as_str())
                    .unwrap_or("journal")
                    .into(),
                category: "system".into(),
                message: entry["MESSAGE"]
                    .as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| entry["MESSAGE"].to_string()),
                data: None,
            })
        })
        .collect()
}

pub fn session(limit: Option<usize>) -> Result<Vec<LogEntry>, EmuBoxError> {
    let path = format!("{}/session.log", super::paths::LOG_DIR);
    let limit = limit.unwrap_or(100).min(1000).to_string();
    let output = super::host_command::output("tail", &["-n", &limit, &path])?;
    Ok(output
        .lines()
        .map(|line| LogEntry {
            timestamp: None,
            level: "unknown".into(),
            source: "emubox-session".into(),
            category: "session".into(),
            message: line.into(),
            data: None,
        })
        .collect())
}

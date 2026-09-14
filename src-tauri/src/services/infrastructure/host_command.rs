use crate::errors::EmuBoxError;
use std::process::Command;

pub fn output(program: &str, arguments: &[&str]) -> Result<String, EmuBoxError> {
    let result = Command::new("timeout")
        .arg("10s")
        .arg(program)
        .args(arguments)
        .env("LC_ALL", "C")
        .output()
        .map_err(|error| EmuBoxError::ProcessFailed(format!("{program}: {error}")))?;
    if !result.status.success() {
        return Err(EmuBoxError::ProcessFailed(format!(
            "{program}: {}",
            String::from_utf8_lossy(&result.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&result.stdout).trim().to_string())
}

pub fn json(program: &str, arguments: &[&str]) -> Result<serde_json::Value, EmuBoxError> {
    serde_json::from_str(&output(program, arguments)?)
        .map_err(|error| EmuBoxError::ProcessFailed(format!("{program}: JSON invalido: {error}")))
}

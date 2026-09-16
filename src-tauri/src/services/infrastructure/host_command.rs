use crate::errors::EmuBoxError;
use std::process::Command;

pub fn output(program: &str, arguments: &[&str]) -> Result<String, EmuBoxError> {
    output_with_timeout(program, arguments, "10s")
}

pub fn output_with_timeout(
    program: &str,
    arguments: &[&str],
    duration: &str,
) -> Result<String, EmuBoxError> {
    let result = Command::new("timeout")
        .args(["--kill-after=1s", duration])
        .arg(program)
        .args(arguments)
        .env("LC_ALL", "C")
        .output()
        .map_err(|error| EmuBoxError::ProcessFailed(format!("{program}: {error}")))?;
    if !result.status.success() {
        if matches!(result.status.code(), Some(124 | 137)) {
            return Err(EmuBoxError::ProcessFailed(format!(
                "{program}: tiempo limite de {duration} agotado"
            )));
        }
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

#[cfg(test)]
mod tests {
    #[test]
    fn bounded_commands_report_success_failure_and_timeout() {
        assert_eq!(
            super::output_with_timeout("printf", &["version"], "1s").unwrap(),
            "version"
        );
        assert!(super::output_with_timeout("false", &[], "1s").is_err());
        let error =
            super::output_with_timeout("sh", &["-c", "while :; do :; done"], "0.1s").unwrap_err();
        assert!(error.to_string().contains("tiempo limite"));
    }
}

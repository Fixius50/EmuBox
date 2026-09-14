use crate::{errors::EmuBoxError, models::system::PowerAction};

pub fn execute(action: PowerAction) -> Result<(), EmuBoxError> {
    match action {
        PowerAction::Shutdown => {
            crate::services::host_command::output("systemctl", &["--no-ask-password", "poweroff"])
        }
        PowerAction::Restart => {
            crate::services::host_command::output("systemctl", &["--no-ask-password", "reboot"])
        }
        PowerAction::Sleep => {
            crate::services::host_command::output("systemctl", &["--no-ask-password", "suspend"])
        }
        PowerAction::Logout => {
            let session = std::env::var("XDG_SESSION_ID").map_err(|_| {
                EmuBoxError::HardwareUnavailable("No hay una sesion logind identificada".into())
            })?;
            crate::services::host_command::output(
                "loginctl",
                &["--no-ask-password", "terminate-session", &session],
            )
        }
    }
    .map(|_| ())
}

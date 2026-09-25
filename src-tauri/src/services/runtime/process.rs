use super::{game_sandbox, launch_policy};
use crate::errors::EmuBoxError;
use crate::models::{LaunchGameRequest, LaunchResult, ProcessStatus, RunningGameInfo};
use crate::services::compatibility_service::CompatibilityService;
use crate::services::game_service::GameService;
use std::os::unix::process::ExitStatusExt;
use std::process::{Child, ExitStatus};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

struct RunningGame {
    info: RunningGameInfo,
    child: Child,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unrelated_process_cannot_be_stopped_over_ipc() {
        assert!(ProcessService::kill_process(std::process::id()).is_err());
    }

    #[test]
    fn completed_child_is_reaped_without_an_ipc_poll() {
        let child = std::process::Command::new("/usr/bin/true").spawn().unwrap();
        let pid = child.id();
        let info = RunningGameInfo {
            pid,
            game_id: "fixture".into(),
            game_title: "Fixture".into(),
            platform_id: "snes".into(),
            emulator_id: "retroarch".into(),
            emulator_name: "RetroArch".into(),
            executable: "/usr/bin/true".into(),
            arguments: vec![],
            start_time: ProcessService::now_epoch_secs(),
            cpu_percent: None,
            memory_mb: None,
            status: "running".into(),
        };
        *CURRENT_RUNNING_GAME.lock().unwrap() = Some(RunningGame { info, child });
        ProcessService::watch_game(pid);
        assert!(CURRENT_RUNNING_GAME.lock().unwrap().is_none());
    }
}

static CURRENT_RUNNING_GAME: Mutex<Option<RunningGame>> = Mutex::new(None);

pub struct ProcessService;

impl ProcessService {
    fn now_epoch_secs() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    fn record_exit(info: &RunningGameInfo, status: ExitStatus) {
        crate::services::infrastructure::telemetry::event(
            if status.success() { log::Level::Info } else { log::Level::Warn },
            "game.launch",
            "game.exit",
            "El sandbox de juego termino",
            serde_json::json!({
                "pid": info.pid,
                "emulatorId": info.emulator_id,
                "elapsedSecs": Self::now_epoch_secs().saturating_sub(info.start_time),
                "exitCode": status.code(),
                "signal": status.signal(),
            }),
        );
    }

    fn watch_game(pid: u32) {
        loop {
            let running = {
                let Ok(mut current) = CURRENT_RUNNING_GAME.lock() else { return };
                let Some(game) = current.as_mut().filter(|game| game.info.pid == pid) else { return };
                match game.child.try_wait() {
                    Ok(Some(status)) => {
                        Self::record_exit(&game.info, status);
                        *current = None;
                        false
                    }
                    Ok(None) => true,
                    Err(error) => {
                        eprintln!("[Game] No se pudo consultar la salida del sandbox: {error}");
                        return;
                    }
                }
            };
            if !running { return; }
            std::thread::sleep(Duration::from_millis(500));
        }
    }

    pub fn launch_game(request: LaunchGameRequest) -> Result<LaunchResult, EmuBoxError> {
        let mut current = CURRENT_RUNNING_GAME
            .lock()
            .map_err(|error| launch_policy::failure(error.to_string()))?;
        if let Some(running) = current.as_mut() {
            let status = running
                .child
                .try_wait()
                .map_err(|error| launch_policy::failure(error.to_string()))?;
            if status.is_none() {
                let info = &running.info;
                return Ok(LaunchResult {
                    success: false,
                    message: format!(
                        "Ya hay un juego en ejecución: {} (PID: {})",
                        info.game_title, info.pid
                    ),
                    pid: Some(info.pid),
                    executable: Some(info.executable.clone()),
                    start_time: Some(info.start_time),
                });
            }
            Self::record_exit(&running.info, status.unwrap());
            *current = None;
        }

        // 2. Obtener metadatos del juego
        let game = GameService::get_game_by_id(request.game_id.clone())?.ok_or_else(|| {
            EmuBoxError::NotFound(format!("Juego no encontrado: {}", request.game_id))
        })?;

        // 3. Obtener metadatos del emulador solicitado (o resolver emulador por defecto de la plataforma)
        let requested_id = request.emulator_id.trim();
        let (emulator, association_args, association_config) =
            CompatibilityService::resolve_for_game(
                &game,
                (!requested_id.is_empty()).then_some(requested_id),
            )?;

        launch_policy::validate_request(
            &request,
            &association_args,
            association_config.as_deref(),
        )?;
        let policy = launch_policy::resolve(&emulator.id, &game.platform)?;
        let executable_path = policy.executable.to_string_lossy().into_owned();
        let mut command = game_sandbox::command(
            &game,
            &emulator.id,
            &policy,
            request.use_gamescope.unwrap_or(false),
        )?;
        let final_args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        let child = game_sandbox::spawn(&mut command)?;

        let pid = child.id();
        let start_time = Self::now_epoch_secs();

        let running_info = RunningGameInfo {
            pid,
            game_id: game.id.clone(),
            game_title: game.title.clone(),
            platform_id: game.platform.clone(),
            emulator_id: emulator.id.clone(),
            emulator_name: emulator.name.clone(),
            executable: executable_path.clone(),
            arguments: final_args,
            start_time,
            cpu_percent: None,
            memory_mb: None,
            status: "running".to_string(),
        };

        *current = Some(RunningGame {
            info: running_info,
            child,
        });
        drop(current);
        if let Err(error) = std::thread::Builder::new().name("game-exit".into()).spawn(move || Self::watch_game(pid)) {
            eprintln!("[Game] No se pudo iniciar la observacion del sandbox: {error}");
        }

        Ok(LaunchResult {
            success: true,
            message: format!(
                "{} iniciado correctamente con {}",
                game.title, emulator.name
            ),
            pid: Some(pid),
            executable: Some(executable_path),
            start_time: Some(start_time),
        })
    }

    pub fn stop_game() -> Result<(), EmuBoxError> {
        let mut current = CURRENT_RUNNING_GAME
            .lock()
            .map_err(|error| launch_policy::failure(error.to_string()))?;
        if let Some(running) = current.as_mut() {
            Self::terminate(running)?;
        }
        *current = None;
        Ok(())
    }

    pub fn is_game_running() -> Result<bool, EmuBoxError> {
        Ok(Self::get_running_game()?.is_some())
    }

    pub fn get_running_game() -> Result<Option<RunningGameInfo>, EmuBoxError> {
        let mut current = CURRENT_RUNNING_GAME
            .lock()
            .map_err(|error| launch_policy::failure(error.to_string()))?;
        if let Some(running) = current.as_mut() {
            let status = running
                .child
                .try_wait()
                .map_err(|error| launch_policy::failure(error.to_string()))?;
            if status.is_none() {
                return Ok(Some(running.info.clone()));
            }
            Self::record_exit(&running.info, status.unwrap());
        }
        *current = None;
        Ok(None)
    }

    pub fn get_process_status() -> Result<ProcessStatus, EmuBoxError> {
        let running_game = Self::get_running_game()?;
        let has_active_game = running_game.is_some();
        let active_child_pids = running_game
            .as_ref()
            .map(|g| vec![g.pid])
            .unwrap_or_default();

        Ok(ProcessStatus {
            has_active_game,
            running_game,
            active_child_pids,
        })
    }

    pub fn kill_process(pid: u32) -> Result<bool, EmuBoxError> {
        let mut current = CURRENT_RUNNING_GAME
            .lock()
            .map_err(|error| launch_policy::failure(error.to_string()))?;
        let running = current
            .as_mut()
            .filter(|running| running.child.id() == pid)
            .ok_or_else(|| {
                launch_policy::failure("Solo se puede detener el sandbox de juego activo")
            })?;
        Self::terminate(running)?;
        *current = None;
        Ok(true)
    }

    fn terminate(running: &mut RunningGame) -> Result<(), EmuBoxError> {
        if running
            .child
            .try_wait()
            .map_err(|error| launch_policy::failure(error.to_string()))?
            .is_none()
        {
            running
                .child
                .kill()
                .map_err(|error| launch_policy::failure(error.to_string()))?;
        }
        running
            .child
            .wait()
            .map_err(|error| launch_policy::failure(error.to_string()))?;
        Ok(())
    }
}

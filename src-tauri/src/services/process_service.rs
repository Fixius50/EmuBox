use std::process::Command;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use std::path::Path;
use crate::models::{LaunchGameRequest, LaunchResult, RunningGameInfo, ProcessStatus};
use crate::errors::EmuBoxError;
use crate::services::game_service::GameService;
use crate::services::compatibility_service::CompatibilityService;

static CURRENT_RUNNING_GAME: Mutex<Option<RunningGameInfo>> = Mutex::new(None);

pub struct ProcessService;

impl ProcessService {
    fn now_epoch_secs() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    fn resolve_executable_path(executable: &str) -> Option<std::path::PathBuf> {
        let trimmed = executable.trim();
        if trimmed.is_empty() {
            return None;
        }

        let direct = std::path::Path::new(trimmed);
        if direct.is_absolute() && direct.exists() {
            return Some(direct.to_path_buf());
        }

        if direct.exists() {
            return Some(direct.to_path_buf());
        }

        if let Ok(output) = Command::new("which").arg(trimmed).output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    let resolved = std::path::PathBuf::from(path);
                    if resolved.exists() {
                        return Some(resolved);
                    }
                }
            }
        }

        None
    }

    pub fn launch_game(request: LaunchGameRequest) -> Result<LaunchResult, EmuBoxError> {
        // 1. Validar si ya hay un juego en ejecución
        {
            let current = CURRENT_RUNNING_GAME.lock().unwrap();
            if let Some(info) = &*current {
                return Ok(LaunchResult {
                    success: false,
                    message: format!("Ya hay un juego en ejecución: {} (PID: {})", info.game_title, info.pid),
                    pid: Some(info.pid),
                    executable: Some(info.executable.clone()),
                    start_time: Some(info.start_time),
                });
            }
        }

        // 2. Obtener metadatos del juego
        let game = GameService::get_game_by_id(request.game_id.clone())?
            .ok_or_else(|| EmuBoxError::NotFound(format!("Juego no encontrado: {}", request.game_id)))?;

        // 3. Obtener metadatos del emulador solicitado (o resolver emulador por defecto de la plataforma)
        let requested_id = request.emulator_id.trim();
        let (emulator, association_args, _association_config) = CompatibilityService::resolve_for_game(
            &game,
            (!requested_id.is_empty()).then_some(requested_id),
        )?;

        let executable_path = Self::resolve_executable_path(&emulator.executable)
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_else(|| emulator.executable.clone());

        if executable_path.is_empty() || Self::resolve_executable_path(&executable_path).is_none() {
            return Err(EmuBoxError::EmulatorNotInstalled(format!(
                "El emulador '{}' no está instalado o no se encuentra en PATH",
                emulator.name
            )));
        }

        // 4. Resolver ruta del archivo ROM
        let rom_path = request.rom_path
            .or(game.rom_path.clone())
            .ok_or_else(|| EmuBoxError::NotFound(format!("No se especificó la ruta ROM para: {}", game.title)))?;

        if !Path::new(&rom_path).exists() {
            return Err(EmuBoxError::NotFound(format!("El archivo de juego no existe en disco: {}", rom_path)));
        }

        // 5. Construir argumentos
        let mut final_args = emulator.arguments.clone();
        final_args.extend(association_args);
        if let Some(custom) = request.custom_args {
            final_args.extend(custom);
        }
        final_args.push(rom_path);

        // 6. Determinar si usar Gamescope para composición nativa
        let use_gamescope = request.use_gamescope.unwrap_or(true);
        let has_gamescope = Command::new("which").arg("gamescope").output().map(|o| o.status.success()).unwrap_or(false);

        let child = if use_gamescope && has_gamescope {
            let mut cmd = Command::new("gamescope");
            cmd.arg("-f").arg("--").arg(&executable_path).args(&final_args);
            cmd.spawn().map_err(|e| EmuBoxError::GameLaunchFailed(e.to_string()))?
        } else {
            let mut cmd = Command::new(&executable_path);
            cmd.args(&final_args);
            cmd.spawn().map_err(|e| EmuBoxError::GameLaunchFailed(e.to_string()))?
        };

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
            cpu_percent: 0.0,
            memory_mb: 0,
            status: "running".to_string(),
        };

        {
            let mut current = CURRENT_RUNNING_GAME.lock().unwrap();
            *current = Some(running_info);
        }

        Ok(LaunchResult {
            success: true,
            message: format!("{} iniciado correctamente con {}", game.title, emulator.name),
            pid: Some(pid),
            executable: Some(executable_path),
            start_time: Some(start_time),
        })
    }

    pub fn stop_game() -> Result<(), EmuBoxError> {
        let pid = {
            let current = CURRENT_RUNNING_GAME.lock().unwrap();
            current.as_ref().map(|info| info.pid)
        };

        if let Some(pid) = pid {
            Self::kill_process(pid)?;
            let mut current = CURRENT_RUNNING_GAME.lock().unwrap();
            *current = None;
        }

        Ok(())
    }

    pub fn is_game_running() -> Result<bool, EmuBoxError> {
        let mut current = CURRENT_RUNNING_GAME.lock().unwrap();
        if let Some(info) = &*current {
            // Comprobar si el PID sigue vivo mediante kill(pid, 0)
            let is_alive = Command::new("kill")
                .arg("-0")
                .arg(info.pid.to_string())
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            if !is_alive {
                *current = None;
                return Ok(false);
            }
            return Ok(true);
        }
        Ok(false)
    }

    pub fn get_running_game() -> Result<Option<RunningGameInfo>, EmuBoxError> {
        let is_running = Self::is_game_running()?;
        if !is_running {
            return Ok(None);
        }
        let current = CURRENT_RUNNING_GAME.lock().unwrap();
        Ok(current.clone())
    }

    pub fn get_process_status() -> Result<ProcessStatus, EmuBoxError> {
        let running_game = Self::get_running_game()?;
        let has_active_game = running_game.is_some();
        let active_child_pids = running_game.as_ref().map(|g| vec![g.pid]).unwrap_or_default();

        Ok(ProcessStatus {
            has_active_game,
            running_game,
            active_child_pids,
        })
    }

    pub fn kill_process(pid: u32) -> Result<bool, EmuBoxError> {
        let status = Command::new("kill")
            .arg("-15") // SIGTERM
            .arg(pid.to_string())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        Ok(status)
    }
}

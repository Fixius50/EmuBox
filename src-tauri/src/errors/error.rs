use serde::{Serialize, Deserialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize)]
#[serde(tag = "code", content = "details")]
pub enum EmuBoxError {
    #[error("Recurso no encontrado: {0}")]
    NotFound(String),

    #[error("Permiso denegado: {0}")]
    PermissionDenied(String),

    #[error("Configuración no válida: {0}")]
    InvalidConfiguration(String),

    #[error("Emulador no instalado en el sistema: {0}")]
    EmulatorNotInstalled(String),

    #[error("Emulador no configurado: {0}")]
    EmulatorNotConfigured(String),

    #[error("Falta archivo BIOS requerido: {0}")]
    BiosMissing(String),

    #[error("Binario ejecutable ausente: {0}")]
    ExecutableMissing(String),

    #[error("Fallo al ejecutar proceso: {0}")]
    ProcessFailed(String),

    #[error("Fallo en el lanzamiento del juego: {0}")]
    GameLaunchFailed(String),

    #[error("Almacenamiento no disponible: {0}")]
    StorageUnavailable(String),

    #[error("Hardware o aceleración no disponible: {0}")]
    HardwareUnavailable(String),

    #[error("Error de comunicación IPC: {0}")]
    IpcError(String),

    #[error("Error desconocido: {0}")]
    Unknown(String),
}

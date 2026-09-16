pub mod bios;
pub mod compatibility;
pub mod diagnostics;
pub mod downloads;
pub mod emulators;
pub mod games;
pub mod input;
pub mod processes;
pub mod startup;
pub mod storage;
pub mod system;

pub(crate) async fn blocking<T, F>(operation: F) -> Result<T, crate::errors::EmuBoxError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, crate::errors::EmuBoxError> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(operation)
        .await
        .map_err(|error| {
            crate::errors::EmuBoxError::ProcessFailed(format!(
                "Tarea de sistema interrumpida: {error}"
            ))
        })?
}

#[cfg(test)]
mod tests {
    #[test]
    fn blocking_commands_use_a_worker_and_preserve_errors() {
        let caller = std::thread::current().id();
        let result = tauri::async_runtime::block_on(super::blocking(move || {
            assert_ne!(std::thread::current().id(), caller);
            Ok(42)
        }));
        assert_eq!(result.unwrap(), 42);
        let failure = tauri::async_runtime::block_on(super::blocking(|| {
            Err::<(), _>(crate::errors::EmuBoxError::NotFound(
                "missing fixture".into(),
            ))
        }));
        assert!(matches!(
            failure,
            Err(crate::errors::EmuBoxError::NotFound(_))
        ));
    }
}

pub use bios::*;
pub use compatibility::*;
pub use diagnostics::*;
pub use downloads::*;
pub use emulators::*;
pub use games::*;
pub use input::*;
pub use processes::*;
pub use storage::*;
pub use system::*;

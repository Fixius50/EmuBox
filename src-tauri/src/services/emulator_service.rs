use crate::models::Emulator;
use crate::errors::EmuBoxError;

pub struct EmulatorService;

impl EmulatorService {
    pub fn get_emulators() -> Result<Vec<Emulator>, EmuBoxError> {
        Ok(vec![])
    }

    pub fn get_emulator_by_id(_id: String) -> Result<Option<Emulator>, EmuBoxError> {
        Ok(None)
    }

    pub fn scan_emulators() -> Result<Vec<Emulator>, EmuBoxError> {
        Ok(vec![])
    }

    pub fn get_emulator_status(_id: String) -> Result<String, EmuBoxError> {
        Ok("active".to_string())
    }

    pub fn save_emulator(_emulator: Emulator) -> Result<(), EmuBoxError> {
        Ok(())
    }

    pub fn delete_emulator(_id: String) -> Result<(), EmuBoxError> {
        Ok(())
    }
}

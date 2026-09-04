use std::path::{Path, PathBuf};
use std::fs;
use rusqlite::Connection;
use crate::errors::EmuBoxError;

pub struct DatabaseService;

impl DatabaseService {
    pub fn get_db_path() -> PathBuf {
        let base = Path::new("/var/lib/emubox");
        if base.exists() || fs::create_dir_all(base).is_ok() {
            base.join("emubox.db")
        } else {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            let fallback = PathBuf::from(home).join(".config/emubox");
            let _ = fs::create_dir_all(&fallback);
            fallback.join("emubox.db")
        }
    }

    pub fn get_connection() -> Result<Connection, EmuBoxError> {
        let db_path = Self::get_db_path();
        let conn = Connection::open(db_path)
            .map_err(|e| EmuBoxError::StorageUnavailable(format!("Error al abrir base de datos SQLite: {}", e)))?;
        
        // Configuración de alto rendimiento para consola dedicada
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;
             PRAGMA busy_timeout = 5000;"
        ).map_err(|e| EmuBoxError::StorageUnavailable(format!("Error al configurar pragmas de SQLite: {}", e)))?;

        Self::init_schema(&conn)?;
        Ok(conn)
    }

    fn init_schema(conn: &Connection) -> Result<(), EmuBoxError> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS systems (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                short_name TEXT NOT NULL,
                manufacturer TEXT NOT NULL,
                generation INTEGER NOT NULL,
                release_year INTEGER NOT NULL,
                color TEXT NOT NULL,
                icon TEXT NOT NULL,
                default_emulator_id TEXT NOT NULL,
                extensions_json TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS emulators (
                id TEXT PRIMARY KEY,
                official_name TEXT NOT NULL,
                version TEXT NOT NULL,
                supported_platforms_json TEXT NOT NULL,
                core_type TEXT NOT NULL,
                status TEXT NOT NULL,
                executable_path TEXT NOT NULL,
                default_arguments_json TEXT NOT NULL,
                installed_at INTEGER
            );

            CREATE TABLE IF NOT EXISTS emulator_metadata (
                emulator_id TEXT PRIMARY KEY,
                config_dir TEXT,
                bios_dir TEXT,
                saves_dir TEXT,
                states_dir TEXT,
                renderer TEXT,
                custom_flags_json TEXT,
                FOREIGN KEY(emulator_id) REFERENCES emulators(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS games (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                platform_id TEXT NOT NULL,
                platform_name TEXT NOT NULL,
                release_year INTEGER,
                genre TEXT,
                developer TEXT,
                publisher TEXT,
                rating REAL,
                play_time_minutes INTEGER DEFAULT 0,
                favorite INTEGER DEFAULT 0,
                cover_image TEXT,
                backdrop_image TEXT,
                description TEXT,
                rom_path TEXT UNIQUE,
                file_size_bytes INTEGER DEFAULT 0,
                added_at INTEGER,
                last_played_at INTEGER,
                FOREIGN KEY(platform_id) REFERENCES systems(id)
            );

            CREATE TABLE IF NOT EXISTS game_emulator_associations (
                game_id TEXT NOT NULL,
                emulator_id TEXT NOT NULL,
                is_default INTEGER DEFAULT 1,
                priority INTEGER DEFAULT 0,
                custom_arguments_json TEXT,
                custom_config_path TEXT,
                enabled INTEGER DEFAULT 1,
                PRIMARY KEY(game_id, emulator_id),
                FOREIGN KEY(game_id) REFERENCES games(id) ON DELETE CASCADE,
                FOREIGN KEY(emulator_id) REFERENCES emulators(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_games_platform ON games(platform_id);
            CREATE INDEX IF NOT EXISTS idx_games_favorite ON games(favorite);
            CREATE INDEX IF NOT EXISTS idx_assocs_game ON game_emulator_associations(game_id);

            CREATE TABLE IF NOT EXISTS download_sources (
                id TEXT PRIMARY KEY,
                game_id TEXT NOT NULL,
                name TEXT NOT NULL,
                source_type TEXT NOT NULL,
                uri TEXT NOT NULL,
                size_bytes INTEGER,
                checksum TEXT,
                available INTEGER NOT NULL DEFAULT 1
            );

            CREATE TABLE IF NOT EXISTS download_jobs (
                id TEXT PRIMARY KEY,
                game_id TEXT NOT NULL,
                source_id TEXT NOT NULL,
                platform TEXT NOT NULL,
                destination_path TEXT NOT NULL,
                status TEXT NOT NULL,
                progress REAL NOT NULL DEFAULT 0,
                downloaded_bytes INTEGER NOT NULL DEFAULT 0,
                total_bytes INTEGER,
                speed_bytes_per_second INTEGER NOT NULL DEFAULT 0,
                error TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_download_jobs_status ON download_jobs(status);"
        ).map_err(|e| EmuBoxError::StorageUnavailable(format!("Error al inicializar tablas en SQLite: {}", e)))?;

        Ok(())
    }
}

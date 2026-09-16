use crate::errors::EmuBoxError;
#[cfg(not(test))]
use crate::services::paths;
use rusqlite::Connection;
#[cfg(not(test))]
use std::fs;
#[cfg(not(test))]
use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct DatabaseService;

static CONNECTION_SETUP: Mutex<()> = Mutex::new(());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concurrent_connections_initialize_consistently() {
        let barrier = std::sync::Barrier::new(8);
        std::thread::scope(|scope| {
            let workers: Vec<_> = (0..8)
                .map(|_| {
                    scope.spawn(|| {
                        barrier.wait();
                        let connection = DatabaseService::get_connection().unwrap();
                        let mode: String = connection
                            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
                            .unwrap();
                        let foreign_keys: i32 = connection
                            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
                            .unwrap();
                        assert_eq!(mode, "wal");
                        assert_eq!(foreign_keys, 1);
                    })
                })
                .collect();
            for worker in workers {
                worker.join().unwrap();
            }
        });
    }
}

impl DatabaseService {
    pub fn get_db_path() -> PathBuf {
        // Los tests nunca deben tocar la base de datos real de producción: cada
        // proceso de test obtiene su propio archivo aislado bajo /tmp.
        #[cfg(test)]
        {
            let isolated =
                std::env::temp_dir().join(format!("emubox-test-{}.db", std::process::id()));
            return isolated;
        }
        #[cfg(not(test))]
        {
            let base = Path::new(paths::DATA_DIR);
            if base.exists() || fs::create_dir_all(base).is_ok() {
                PathBuf::from(paths::database_path())
            } else {
                let fallback = PathBuf::from("/tmp/emubox");
                let _ = fs::create_dir_all(&fallback);
                fallback.join("emubox.db")
            }
        }
    }

    pub fn get_connection() -> Result<Connection, EmuBoxError> {
        let _setup = CONNECTION_SETUP.lock().map_err(|error| {
            EmuBoxError::StorageUnavailable(format!("Inicializacion SQLite interrumpida: {error}"))
        })?;
        let db_path = Self::get_db_path();
        let conn = Connection::open(db_path).map_err(|e| {
            EmuBoxError::StorageUnavailable(format!("Error al abrir base de datos SQLite: {}", e))
        })?;
        conn.busy_timeout(std::time::Duration::from_secs(5))
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;

        // Configuración de alto rendimiento para consola dedicada
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;",
        )
        .map_err(|e| {
            EmuBoxError::StorageUnavailable(format!("Error al configurar pragmas de SQLite: {}", e))
        })?;

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

            CREATE TABLE IF NOT EXISTS canonical_games (
                id TEXT PRIMARY KEY,
                authority TEXT NOT NULL,
                authority_id TEXT NOT NULL,
                title TEXT NOT NULL,
                normalized_title TEXT NOT NULL,
                platform_id TEXT NOT NULL,
                release_year INTEGER,
                genre TEXT,
                developer TEXT,
                publisher TEXT,
                cover_image TEXT,
                description TEXT,
                favorite INTEGER NOT NULL DEFAULT 0,
                updated_at INTEGER NOT NULL,
                UNIQUE(authority, authority_id),
                FOREIGN KEY(platform_id) REFERENCES systems(id)
            );

            CREATE INDEX IF NOT EXISTS idx_canonical_games_platform_title
                ON canonical_games(platform_id, normalized_title);

            CREATE TABLE IF NOT EXISTS game_releases (
                id TEXT PRIMARY KEY,
                canonical_game_id TEXT NOT NULL,
                authority TEXT NOT NULL,
                authority_id TEXT NOT NULL,
                title TEXT NOT NULL,
                normalized_title TEXT NOT NULL,
                region TEXT,
                serial TEXT,
                crc TEXT,
                md5 TEXT,
                sha1 TEXT,
                UNIQUE(authority, authority_id),
                FOREIGN KEY(canonical_game_id) REFERENCES canonical_games(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_game_releases_canonical
                ON game_releases(canonical_game_id);
            CREATE INDEX IF NOT EXISTS idx_game_releases_title
                ON game_releases(normalized_title);

            CREATE TABLE IF NOT EXISTS canonical_aliases (
                canonical_game_id TEXT NOT NULL,
                platform_id TEXT NOT NULL,
                normalized_alias TEXT NOT NULL,
                kind TEXT NOT NULL,
                PRIMARY KEY(canonical_game_id, normalized_alias, kind),
                FOREIGN KEY(canonical_game_id) REFERENCES canonical_games(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_canonical_alias_lookup
                ON canonical_aliases(platform_id, normalized_alias);

            CREATE TABLE IF NOT EXISTS catalog_game_matches (
                catalog_game_id TEXT PRIMARY KEY,
                canonical_game_id TEXT NOT NULL,
                release_id TEXT,
                method TEXT NOT NULL,
                confidence INTEGER NOT NULL CHECK(confidence BETWEEN 0 AND 100),
                matched_at INTEGER NOT NULL,
                FOREIGN KEY(catalog_game_id) REFERENCES games(id) ON DELETE CASCADE,
                FOREIGN KEY(canonical_game_id) REFERENCES canonical_games(id) ON DELETE CASCADE,
                FOREIGN KEY(release_id) REFERENCES game_releases(id) ON DELETE SET NULL
            );

            CREATE INDEX IF NOT EXISTS idx_catalog_matches_canonical
                ON catalog_game_matches(canonical_game_id);

            CREATE TABLE IF NOT EXISTS game_database_sources (
                platform_id TEXT PRIMARY KEY,
                authority TEXT NOT NULL,
                url TEXT NOT NULL,
                etag TEXT,
                modified TEXT,
                checked_at INTEGER,
                imported_at INTEGER,
                entry_count INTEGER NOT NULL DEFAULT 0,
                error TEXT
            );

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

            CREATE INDEX IF NOT EXISTS idx_download_jobs_status ON download_jobs(status);
            CREATE TABLE IF NOT EXISTS download_execution (
                job_id TEXT PRIMARY KEY,
                source_json TEXT NOT NULL,
                provider TEXT NOT NULL,
                phase TEXT NOT NULL DEFAULT 'queued',
                artifacts_json TEXT
            );",
        )
        .map_err(|e| {
            EmuBoxError::StorageUnavailable(format!("Error al inicializar tablas en SQLite: {}", e))
        })?;

        Ok(())
    }
}

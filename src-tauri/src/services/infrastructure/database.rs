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
static SCHEMA_INITIALIZED: Mutex<bool> = Mutex::new(false);

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
        conn.busy_timeout(std::time::Duration::from_secs(30))
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;

        // Configuración de alto rendimiento para consola dedicada
        conn.execute_batch(
            "PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;
             PRAGMA temp_store = MEMORY;
             PRAGMA cache_size = -64000;
             PRAGMA mmap_size = 268435456;",
        )
        .map_err(|e| {
            EmuBoxError::StorageUnavailable(format!("Error al configurar pragmas de SQLite: {}", e))
        })?;

        let mut schema_initialized = SCHEMA_INITIALIZED.lock().map_err(|error| {
            EmuBoxError::StorageUnavailable(format!("Inicializacion SQLite interrumpida: {error}"))
        })?;
        if !*schema_initialized {
            let _ = conn.execute_batch("PRAGMA journal_mode = WAL;");
            Self::init_schema(&conn)?;
            *schema_initialized = true;
        }
        Ok(conn)
    }

    pub fn get_platform_db_path(platform_id: &str) -> PathBuf {
        let clean_id: String = platform_id
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        #[cfg(test)]
        {
            return std::env::temp_dir()
                .join(format!("emubox-platform-{clean_id}-{}.db", std::process::id()));
        }
        #[cfg(not(test))]
        {
            let base = Path::new(paths::DATA_DIR).join("catalogs");
            if !base.exists() {
                let _ = fs::create_dir_all(&base);
            }
            base.join(format!("{clean_id}.db"))
        }
    }

    pub fn get_platform_connection(platform_id: &str) -> Result<Connection, EmuBoxError> {
        let db_path = Self::get_platform_db_path(platform_id);
        if let Some(parent) = db_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let conn = Connection::open(&db_path).map_err(|e| {
            EmuBoxError::StorageUnavailable(format!("Error al abrir base de datos de plataforma {}: {}", platform_id, e))
        })?;
        conn.busy_timeout(std::time::Duration::from_secs(30))
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;
             PRAGMA temp_store = MEMORY;
             PRAGMA cache_size = -32000;
             PRAGMA mmap_size = 67108864;",
        ).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;

        Self::init_platform_schema(&conn)?;
        Ok(conn)
    }

    fn init_platform_schema(conn: &Connection) -> Result<(), EmuBoxError> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS games (
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
                last_played_at INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_platform_games_title ON games(title COLLATE NOCASE);
            CREATE TABLE IF NOT EXISTS download_sources (
                id TEXT PRIMARY KEY,
                game_id TEXT NOT NULL,
                name TEXT NOT NULL,
                source_type TEXT NOT NULL,
                uri TEXT NOT NULL,
                size_bytes INTEGER,
                format TEXT,
                region TEXT,
                version TEXT,
                available INTEGER DEFAULT 1,
                priority INTEGER DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_platform_download_sources_game_id ON download_sources(game_id);
            CREATE TABLE IF NOT EXISTS manifest_sources (
                manifest_url TEXT NOT NULL,
                game_id TEXT NOT NULL,
                source_id TEXT NOT NULL,
                PRIMARY KEY(manifest_url, game_id, source_id)
            );
            CREATE TABLE IF NOT EXISTS manifest_entry_cache (
                manifest_url TEXT NOT NULL,
                game_id TEXT NOT NULL,
                digest TEXT NOT NULL,
                PRIMARY KEY(manifest_url, game_id)
            );"
        ).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
        Ok(())
    }

    fn init_schema(conn: &Connection) -> Result<(), EmuBoxError> {
        let current_version: u32 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap_or(0);

        if current_version < 1 {
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

            CREATE TABLE IF NOT EXISTS store_accounts (
                id TEXT PRIMARY KEY,
                provider TEXT NOT NULL,
                external_account_id TEXT NOT NULL,
                display_name TEXT NOT NULL,
                avatar_url TEXT,
                status TEXT NOT NULL,
                last_login_at INTEGER,
                last_sync_at INTEGER,
                UNIQUE(provider, external_account_id)
            );

            CREATE TABLE IF NOT EXISTS store_games (
                provider TEXT NOT NULL,
                external_game_id TEXT NOT NULL,
                title TEXT NOT NULL,
                metadata_json TEXT NOT NULL DEFAULT '{}',
                updated_at INTEGER NOT NULL,
                PRIMARY KEY(provider, external_game_id)
            );

            CREATE TABLE IF NOT EXISTS store_entitlements (
                store_account_id TEXT NOT NULL,
                provider TEXT NOT NULL,
                external_game_id TEXT NOT NULL,
                owned INTEGER NOT NULL CHECK(owned IN (0, 1)),
                installed INTEGER NOT NULL CHECK(installed IN (0, 1)),
                last_seen_at INTEGER NOT NULL,
                PRIMARY KEY(store_account_id, provider, external_game_id),
                FOREIGN KEY(store_account_id) REFERENCES store_accounts(id) ON DELETE CASCADE,
                FOREIGN KEY(provider, external_game_id) REFERENCES store_games(provider, external_game_id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS store_game_links (
                provider TEXT NOT NULL,
                external_game_id TEXT NOT NULL,
                canonical_game_id TEXT NOT NULL,
                PRIMARY KEY(provider, external_game_id),
                FOREIGN KEY(provider, external_game_id) REFERENCES store_games(provider, external_game_id) ON DELETE CASCADE,
                FOREIGN KEY(canonical_game_id) REFERENCES canonical_games(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_store_entitlements_account
                ON store_entitlements(store_account_id);
            CREATE INDEX IF NOT EXISTS idx_store_links_canonical
                ON store_game_links(canonical_game_id);

            CREATE TABLE IF NOT EXISTS store_provider_states (
                provider TEXT PRIMARY KEY,
                status TEXT NOT NULL CHECK(status IN ('authorization_required', 'ready', 'syncing', 'error')),
                authorization_status TEXT NOT NULL DEFAULT 'required' CHECK(authorization_status IN ('required', 'connected')),
                last_sync_at INTEGER,
                error_message TEXT,
                updated_at INTEGER NOT NULL
            );

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

        let has_authorization_status = conn
            .prepare("PRAGMA table_info(store_provider_states)")
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?
            .iter()
            .any(|column| column == "authorization_status");
        if !has_authorization_status {
            conn.execute_batch("ALTER TABLE store_provider_states ADD COLUMN authorization_status TEXT NOT NULL DEFAULT 'required' CHECK(authorization_status IN ('required', 'connected')); UPDATE store_provider_states SET authorization_status='connected' WHERE provider IN (SELECT DISTINCT provider FROM store_accounts WHERE status='authenticated');")
                .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        }
        conn.execute_batch("INSERT OR IGNORE INTO store_provider_states(provider,status,authorization_status,updated_at) VALUES ('steam','authorization_required','required',0), ('epic','authorization_required','required',0), ('gog','authorization_required','required',0);")
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;

            conn.execute_batch("PRAGMA user_version = 1;")
                .map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
        }

        if current_version < 2 {
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS game_sessions (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    game_id TEXT NOT NULL,
                    emulator_id TEXT NOT NULL,
                    started_at INTEGER NOT NULL,
                    ended_at INTEGER,
                    duration_seconds INTEGER DEFAULT 0,
                    exit_code INTEGER,
                    error_message TEXT,
                    FOREIGN KEY(game_id) REFERENCES games(id) ON DELETE CASCADE,
                    FOREIGN KEY(emulator_id) REFERENCES emulators(id) ON DELETE CASCADE
                );
                CREATE INDEX IF NOT EXISTS idx_game_sessions_game ON game_sessions(game_id);
                CREATE INDEX IF NOT EXISTS idx_game_sessions_started ON game_sessions(started_at);

                CREATE TABLE IF NOT EXISTS bios_cache (
                    filename TEXT NOT NULL,
                    platform_id TEXT NOT NULL,
                    file_size INTEGER NOT NULL,
                    file_mtime INTEGER NOT NULL,
                    md5 TEXT,
                    sha1 TEXT,
                    is_valid INTEGER NOT NULL DEFAULT 0,
                    checked_at INTEGER NOT NULL,
                    PRIMARY KEY(filename, platform_id)
                );

                CREATE TABLE IF NOT EXISTS app_settings_kv (
                    key TEXT PRIMARY KEY,
                    value_json TEXT NOT NULL,
                    updated_at INTEGER NOT NULL
                );

                PRAGMA user_version = 2;",
            )
            .map_err(|e| EmuBoxError::StorageUnavailable(format!("Error en migracion v2 SQLite: {}", e)))?;
        }

        Ok(())
    }
}

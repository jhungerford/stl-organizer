use std::fmt::Formatter;

use rusqlite::{Connection, OpenFlags};
use rusqlite::{NO_PARAMS, params};
use serde::Serialize;

/// Commands contains the settings tauri commands.  Testing behavior with tauri's State is tricky,
/// so these commands are thin wrappers around the functions in the settings module.
pub mod commands {
    use super::*;
    use tauri::{ command, State };

    /// `list_dirs` returns a list of all of the directories sql-organizer scans,
    /// in alphabetical order.
    #[command]
    pub fn list_dirs(conn_manager: State<InMemoryConnectionManager>) -> Result<Vec<String>, SettingsError> {
        super::list_dirs(&conn_manager)
    }

    /// `add_dir` adds a directory to the list that stl-organizer scans.
    #[command]
    pub fn add_dir(conn_manager: State<InMemoryConnectionManager>, dir: String) -> Result<(), SettingsError> {
        super::add_dir(&conn_manager, dir)
    }
}

/// SettingsError is a unified error type for settings results.
#[derive(Debug, Clone, Serialize)]
pub struct SettingsError {
    message: String,
}

impl SettingsError {
    /// Constructs a new SettingsError with the given message.
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl std::fmt::Display for SettingsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<rusqlite::Error> for SettingsError {
    fn from(err: rusqlite::Error) -> Self {
        SettingsError::new(err.to_string())
    }
}

impl From<refinery::Error> for SettingsError {
    fn from(err: refinery::Error) -> Self {
        SettingsError::new(err.to_string())
    }
}

/// `list_dirs` returns a list of all of the directories that stl-organizer will scan,
/// in alphabetical order.
pub fn list_dirs(conn_manager: &InMemoryConnectionManager) -> Result<Vec<String>, SettingsError> {
    let conn = conn_manager.get_connection()?;
    let mut stmt = conn.prepare("SELECT name FROM directories ORDER BY name")?;
    let rows = stmt.query_map(NO_PARAMS, |row| row.get(0));

    let mut dirs = Vec::new();
    for row in rows? {
        dirs.push(row?);
    }

    Ok(dirs)
}

/// `add_dir` adds a directory to the list or directories that stl-organizer scans.
pub fn add_dir(conn_manager: &InMemoryConnectionManager, dir: String) -> Result<(), SettingsError> {
    let conn = conn_manager.get_connection()?;
    conn.execute("INSERT INTO directories (name) VALUES (?)", params![dir])?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_dirs_empty() {
        let conn_manager = InMemoryConnectionManager::new("test_get_dirs_empty").unwrap();
        conn_manager.migrate().unwrap();

        clear_dirs(&conn_manager).expect("Error clearing dirs");

        let listed_dirs = list_dirs(&conn_manager);
        let expected: Vec<String> = vec![];

        assert!(listed_dirs.is_ok());
        assert_eq!(expected, listed_dirs.unwrap());
    }

    #[test]
    fn test_add_get_dirs() {
        let conn_manager = InMemoryConnectionManager::new("test_add_dirs").unwrap();
        conn_manager.migrate().unwrap();

        clear_dirs(&conn_manager).expect("Error clearing dirs");

        for dir in vec!["~/Downloads", "~/Documents"] {
            add_dir(&conn_manager, dir.to_string()).expect(&format!("Error adding {}", dir));
        }

        let dirs = list_dirs(&conn_manager);
        let expected = vec!["~/Documents".to_string(), "~/Downloads".to_string()];
        assert!(dirs.is_ok());
        assert_eq!(expected, dirs.unwrap());
    }

    fn clear_dirs(conn_manager: &InMemoryConnectionManager) -> Result<(), SettingsError> {
        let conn = conn_manager.get_connection()?;
        conn.execute("DELETE FROM directories", NO_PARAMS)?;

        Ok(())
    }
}

/// The `db` module wires the refinery migrations stored in `src-tauri/sql_migrations` into sqlite.
mod db {
    use refinery::embed_migrations;
    embed_migrations!("./sql_migrations");
}

/// `ConnectionManager` is a wrapper around a sqlite database, allowing callers to get a connection
/// or migrate the schema.
pub trait ConnectionManager: Send + Sync {
    /// Returns a connection to the database.  `Connection` automatically closes the connection
    /// when the variable is dropped, so callers don't need to manually close the connection.
    fn get_connection(&self) -> rusqlite::Result<Connection>;

    /// Migrates the schema by running the migrations in `src-tauri/sql_migrations`.
    fn migrate(&self) -> Result<(), SettingsError> {
        let mut conn = self.get_connection()?;
        db::migrations::runner().run(&mut conn)?;

        Ok(())
    }
}

/// `InMemoryConnectionManager` runs an in-memory SQLite DB that will run in the process's memory
/// for as long as the connection manager stays in scope.  Useful for testing.
pub struct InMemoryConnectionManager {
    /// In-memory DB stays alive as long as one connection is open.  Keep a connection
    /// for the lifetime of the InMemoryConnectionManager - the DB will be cleared once
    /// the connection manager is dropped.
    #[allow(dead_code)]
    conn: Connection,
    name: String,
}

impl InMemoryConnectionManager {
    /// Constructs a new `InMemoryConnectionManager` with the given unique name.
    /// If multiple databases need to exist (e.g. one per test), name distinguishes them.
    #[allow(dead_code)]
    pub fn new(name: &str) -> Result<InMemoryConnectionManager, SettingsError> {
        // rusqlite provides `Connection::open_in_memory`, but in-memory databases are allowed to
        // use shared cache when opened with a uri filename (file::memory: instead of :memory:)
        // and the open shared cache flag.  See https://sqlite.org/inmemorydb.html
        let conn = InMemoryConnectionManager::open_connection(name)?;

        Ok(InMemoryConnectionManager {
            conn,
            name: name.to_string()
        })
    }

    /// Opens a connection to the shared in-memory database.
    fn open_connection(name: &str) -> rusqlite::Result<Connection> {
        let flags = OpenFlags::default() | OpenFlags::SQLITE_OPEN_SHARED_CACHE;
        let path = format!("file:{}?mode=memory&cache=shared", name);

        Connection::open_with_flags(path, flags)
    }
}

/// `InMemoryConnectionManager`'s parent connection to the database isn't safe to use in multiple
/// threads, but it's open to give the DB the same lifetime as the connection manager.
/// Threads call `conn_manager.get_connection()` to get a thread-safe connection to the DB.
unsafe impl Sync for InMemoryConnectionManager {}

impl ConnectionManager for InMemoryConnectionManager {
    fn get_connection(&self) -> rusqlite::Result<Connection> {
        return InMemoryConnectionManager::open_connection(&self.name)
    }
}

/// `FileConnectionManager` is a persistent SQLite database stored in a file on disk.
pub struct FileConnectionManager {
    file: String
}

impl FileConnectionManager {
    #[allow(dead_code)]
    pub fn new(file: String) -> FileConnectionManager {
        FileConnectionManager { file }
    }
}

impl ConnectionManager for FileConnectionManager {
    fn get_connection(&self) -> rusqlite::Result<Connection> {
        // TODO: DB shows up as locked when stl-organizer is run on windows with the DB on the wsl
        //       filesystem.  SQLite allows specifying a 'vfs' virtual file system object -
        //       maybe that could help?  http://www.sqlite.org/c3ref/open.html
        return Connection::open(&self.file);
    }
}

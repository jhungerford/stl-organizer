use std::fmt::Formatter;

use crate::db::{ConnectionManager, DbError, InMemoryConnectionManager};
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
        SettingsError { message: err.to_string() }
    }
}

impl From<refinery::Error> for SettingsError {
    fn from(err: refinery::Error) -> Self {
        SettingsError { message: err.to_string() }
    }
}

impl From<DbError> for SettingsError {
    fn from(dbError: DbError) -> Self {
        SettingsError {
            message: dbError.message
        }
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

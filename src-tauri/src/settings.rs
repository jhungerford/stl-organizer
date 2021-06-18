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
        let settings = Settings::new(conn_manager.inner());
        settings.list_dirs()
    }

    /// `add_dir` adds a directory to the list that stl-organizer scans.
    #[command]
    pub fn add_dir(conn_manager: State<InMemoryConnectionManager>, dir: &str) -> Result<(), SettingsError> {
        let settings = Settings::new(conn_manager.inner());
        settings.add_dir(dir)
    }
}

pub struct Settings<'a, T: ConnectionManager> {
    conn_manager: &'a T
}

impl<'a, T: ConnectionManager> Settings<'a, T> {
    /// Creates a new Settings.
    pub fn new(conn_manager: &'a T) -> Self {
        Settings { conn_manager }
    }

    /// `list_dirs` returns a list of all of the directories that stl-organizer will scan,
    /// in alphabetical order.
    pub fn list_dirs(&self) -> Result<Vec<String>, SettingsError> {
        let conn = self.conn_manager.get_connection()?;
        let mut stmt = conn.prepare("SELECT name FROM directories ORDER BY name")?;
        let rows = stmt.query_map(NO_PARAMS, |row| row.get(0));

        let mut dirs = Vec::new();
        for row in rows? {
            dirs.push(row?);
        }

        Ok(dirs)
    }

    /// `add_dir` adds a directory to the list or directories that stl-organizer scans.
    pub fn add_dir(&self, dir: &str) -> Result<(), SettingsError> {
        let conn = self.conn_manager.get_connection()?;
        conn.execute("INSERT INTO directories (name) VALUES (?)", params![dir])?;

        Ok(())
    }

    /// `clear_dirs` removes all of the directories registered in settings, for testing.
    fn clear_dirs(&self) -> Result<(), SettingsError> {
        let conn = self.conn_manager.get_connection()?;
        conn.execute("DELETE FROM directories", NO_PARAMS)?;

        Ok(())
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
    fn from(err: DbError) -> Self {
        SettingsError {
            message: err.message
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_dirs_empty() {
        let conn_manager = create_db("test_get_dirs_empty");
        let settings = Settings::new(&conn_manager);

        settings.clear_dirs().expect("Error clearing dirs");

        let listed_dirs = settings.list_dirs();
        let expected: Vec<String> = vec![];

        assert!(listed_dirs.is_ok());
        assert_eq!(expected, listed_dirs.unwrap());
    }

    #[test]
    fn test_add_get_dirs() {
        let conn_manager = create_db("test_add_dirs");
        let settings = Settings::new(&conn_manager);

        settings.clear_dirs().expect("Error clearing dirs");

        for dir in vec!["~/Downloads", "~/Documents"] {
            settings.add_dir(dir).expect(&format!("Error adding {}", dir));
        }

        let dirs = settings.list_dirs();
        let expected = vec!["~/Documents".to_string(), "~/Downloads".to_string()];
        assert!(dirs.is_ok());
        assert_eq!(expected, dirs.unwrap());
    }

    #[test]
    fn test_clear_dirs() {
        let conn_manager = create_db("test_clear_dirs");
        let settings = Settings::new(&conn_manager);

        settings.clear_dirs().expect("Error clearing dirs");

        settings.add_dir("~/Downloads").expect("Error adding dir");
        assert_eq!(1, settings.list_dirs().unwrap().len());

        settings.clear_dirs().expect("Error clearing dirs");
        assert_eq!(0, settings.list_dirs().unwrap().len());
    }

    fn create_db(db_name: &str) -> InMemoryConnectionManager {
        let conn_manager = InMemoryConnectionManager::new(db_name)
            .expect("Error creating connection manager.");
        
        conn_manager.migrate().expect("Error migrating db.");

        conn_manager
    }
}

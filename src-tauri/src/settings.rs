use std::sync::Arc;

use crate::db::{ConnectionManager, InMemoryConnectionManager};
use crate::error::AppError;
use rusqlite::{NO_PARAMS, params};

/// Commands contains the settings tauri commands.  Testing behavior with tauri's State is tricky,
/// so these commands are thin wrappers around the functions in the settings module.
pub mod commands {
    use super::*;
    use tauri::{ command, State };

    /// `list_dirs` returns a list of all of the directories sql-organizer scans,
    /// in alphabetical order.
    #[command]
    pub fn list_dirs(settings: State<Settings<InMemoryConnectionManager>>) -> Result<Vec<String>, AppError> {
        settings.list_dirs()
    }

    /// `add_dir` adds a directory to the list that stl-organizer scans.
    #[command]
    pub fn add_dir(settings: State<Settings<InMemoryConnectionManager>>, dir: &str) -> Result<(), AppError> {
        settings.add_dir(dir)
    }
}

/// Settings stores user-specified values in the application, like the directories to scan for 3d printing files.
pub struct Settings<T: ConnectionManager> {
    conn_manager: Arc<T>
}

impl<T: ConnectionManager> Settings<T> {
    /// Creates a new Settings.
    pub fn new(conn_manager: Arc<T>) -> Self {
        Settings { conn_manager }
    }

    /// `list_dirs` returns a list of all of the directories that stl-organizer will scan,
    /// in alphabetical order.
    pub fn list_dirs(&self) -> Result<Vec<String>, AppError> {
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
    pub fn add_dir(&self, dir: &str) -> Result<(), AppError> {
        let conn = self.conn_manager.get_connection()?;
        conn.execute("INSERT INTO directories (name) VALUES (?)", params![dir])?;

        Ok(())
    }

    /// `clear_dirs` removes all of the directories registered in settings, for testing.
    fn clear_dirs(&self) -> Result<(), AppError> {
        let conn = self.conn_manager.get_connection()?;
        conn.execute("DELETE FROM directories", NO_PARAMS)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_dirs_empty() {
        let settings = create_test_settings("test_get_dirs_empty");

        settings.clear_dirs().expect("Error clearing dirs");

        let listed_dirs = settings.list_dirs();
        let expected: Vec<String> = vec![];

        assert!(listed_dirs.is_ok());
        assert_eq!(expected, listed_dirs.unwrap());
    }

    #[test]
    fn test_add_get_dirs() {
        let settings = create_test_settings("test_add_dirs");

        settings.clear_dirs().expect("Error clearing dirs");

        for dir in vec!["~/Downloads", "~/Documents"] {
            settings.add_dir(dir).expect(&format!("Error adding {}", dir));
        }

        let dirs = settings.list_dirs();
        let expected = vec!["~/Documents", "~/Downloads"];
        assert!(dirs.is_ok());
        assert_eq!(expected, dirs.unwrap());
    }

    #[test]
    fn test_clear_dirs() {
        let settings = create_test_settings("test_clear_dirs");

        settings.clear_dirs().expect("Error clearing dirs");

        settings.add_dir("~/Downloads").expect("Error adding dir");
        assert_eq!(1, settings.list_dirs().unwrap().len());

        settings.clear_dirs().expect("Error clearing dirs");
        assert_eq!(0, settings.list_dirs().unwrap().len());
    }

    /// Creates and migrates a database with the given name, returning Settings that will store data in the db.
    fn create_test_settings(db_name: &str) -> Settings<InMemoryConnectionManager> {
        let conn_manager = InMemoryConnectionManager::new(db_name)
            .expect("Error creating connection manager.");
        
        conn_manager.migrate().expect("Error migrating db.");

        Settings::new(Arc::new(conn_manager))
    }
}

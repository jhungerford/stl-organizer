#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use crate::db::{ConnectionManager, InMemoryConnectionManager};
use crate::settings::{Settings, SettingsError};

mod db;
mod scan;
mod settings;

/// Initializes and migrates the database, returning a ConnectionManager that can access it.
fn init_db() -> Result<InMemoryConnectionManager, SettingsError> {
  let conn_manager = InMemoryConnectionManager::new("stl-organizer")?;
  
  conn_manager.migrate()?;

  Ok(conn_manager)
}

fn main() {
  let conn_manager = init_db().expect("Error initializing DB.");
  let settings = Settings::new(conn_manager);

  settings.add_dir("~/Downloads").expect("Error adding sample directory.");

  tauri::Builder::default()
      .manage(settings)
      .invoke_handler(tauri::generate_handler![
        settings::commands::list_dirs,
        settings::commands::add_dir
      ])
      .run(tauri::generate_context!())
      .expect("error while running tauri application");
}

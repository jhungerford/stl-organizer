#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use crate::settings::{InMemoryConnectionManager, ConnectionManager};

mod settings;

fn main() {
  let conn_manager = InMemoryConnectionManager::new("stl-organizer")
      .expect("Error creating DB connection.");

  conn_manager.migrate().expect("Error migrating schema.");
  settings::add_dir(&conn_manager, "~/Downloads".to_string())
      .expect("Error adding sample directory.");

  tauri::Builder::default()
      .manage(conn_manager)
      .invoke_handler(tauri::generate_handler![
        settings::commands::list_dirs,
        settings::commands::add_dir
      ])
      .run(tauri::generate_context!())
      .expect("error while running tauri application");
}

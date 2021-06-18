// #![cfg_attr(
//   all(not(debug_assertions), target_os = "windows"),
//   windows_subsystem = "windows"
// )]

use crate::db::{ConnectionManager, InMemoryConnectionManager};
use crate::scan::Scanner;
use crate::settings::Settings;

mod db;
mod scan;
mod settings;

fn main() {
  let conn_manager = InMemoryConnectionManager::new("stl-organizer")
  .expect("Error connecting to db.");

  let scanner = Scanner::new();

  conn_manager.migrate().expect("Error initalizing db.");

  let settings = Settings::new(&conn_manager);
  settings.add_dir("~/Downloads").expect("Error adding sample directory.");

  tauri::Builder::default()
      .manage(scanner)
      .manage(conn_manager)
      .invoke_handler(tauri::generate_handler![
        settings::commands::list_dirs,
        settings::commands::add_dir
      ])
      .run(tauri::generate_context!())
      .expect("error while running tauri application");
}

// #![cfg_attr(
//   all(not(debug_assertions), target_os = "windows"),
//   windows_subsystem = "windows"
// )]

use std::sync::{Arc, Mutex};

use crate::db::{ConnectionManager, InMemoryConnectionManager};
use crate::scan::Scanner;
use crate::settings::Settings;

mod db;
mod error;
mod scan;
mod settings;

fn main() {
  let conn_manager = InMemoryConnectionManager::new("stl-organizer")
  .expect("Error connecting to db.");
  
  conn_manager.migrate().expect("Error initalizing db.");

  let conn_manager_arc = Arc::new(conn_manager);

  let settings = Settings::new(conn_manager_arc.clone());
  settings.add_dir("~/Downloads").expect("Error adding sample directory.");

  let scanner = Mutex::new(Scanner::new(conn_manager_arc.clone()).expect("Error initializing scanner."));

  tauri::Builder::default()
      .manage(settings)
      .manage(scanner)
      .invoke_handler(tauri::generate_handler![
        settings::commands::list_dirs,
        settings::commands::add_dir,
        scan::commands::scan_start,
        scan::commands::scan_progress,
      ])
      .run(tauri::generate_context!())
      .expect("error while running tauri application");
}

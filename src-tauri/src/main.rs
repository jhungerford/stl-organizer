#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

#[tauri::command]
fn settings_get_dirs() -> Result<Vec<String>, String> {
  Ok(vec![
    "~/Downloads".to_string()
  ])
}

#[tauri::command]
fn sample_command() {
  println!("Command invoked from JS");
}

fn main() {
  tauri::Builder::default()
      .invoke_handler(tauri::generate_handler![sample_command])
      .invoke_handler(tauri::generate_handler![settings_get_dirs])
      .run(tauri::generate_context!())
      .expect("error while running tauri application");
}

#[tauri::command]
pub fn get_dirs() -> Result<Vec<String>, String> {
    Ok(vec![
        "~/Downloads".to_string()
    ])
}
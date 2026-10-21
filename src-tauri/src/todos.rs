//! To-do list storage: one JSON file in ~/.config/linotch. The UI owns the format; Rust only reads and writes it.
use std::{fs, path::PathBuf};

fn file() -> PathBuf {
    PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".config/linotch/todos.json")
}

#[tauri::command]
pub fn todos_load() -> String {
    fs::read_to_string(file()).unwrap_or_else(|_| "[]".into())
}

#[tauri::command]
pub fn todos_save(data: String) -> Result<(), String> {
    let path = file();
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    // Write then rename, so a crash mid-write never leaves a half-written list.
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, data).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &path).map_err(|e| e.to_string())
}

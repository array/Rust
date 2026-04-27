// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::Path;
use tauri::{command, Emitter};

#[command]
fn read_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| e.to_string())
}

#[command]
fn list_markdown_files(dir: String) -> Result<Vec<String>, String> {
    let mut files = Vec::new();
    collect_md_files(Path::new(&dir), &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_md_files(dir: &Path, files: &mut Vec<String>) -> Result<(), String> {
    let entries = std::fs::read_dir(dir).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            collect_md_files(&path, files)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Some(p) = path.to_str() {
                files.push(p.to_string());
            }
        }
    }
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::DragDrop(tauri::DragDropEvent::Drop { paths, .. }) = &event {
                for path in paths {
                    if path.extension().and_then(|e| e.to_str()) == Some("md") {
                        let path_str = path.to_string_lossy().to_string();
                        window.emit("file-dropped", path_str).ok();
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![read_file, list_markdown_files])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod error;
mod task;
mod tracing_helper;
mod utils;

#[tauri::command]
fn echo(s: &str) -> &str {
    s
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![echo])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

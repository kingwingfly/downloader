// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use model::TaskBmc;
use std::{
    cell::RefCell,
    sync::{Mutex, OnceLock},
};

mod config;
mod model;
mod task;
mod tracing_helper;
mod utils;

static TASK_BMC: OnceLock<Mutex<RefCell<TaskBmc>>> = OnceLock::new();

#[tauri::command]
fn echo(s: &str) -> &str {
    s
}

#[tauri::command]
fn create(url: String) {
    let task_bmc = TASK_BMC.get().unwrap().lock().unwrap();
    task_bmc.borrow_mut().create(url).unwrap();
}

macro_rules! gen_tauri_task_handler {
    ($op: ident) => {
        #[tauri::command]
        fn $op(id: String) {
            let task_bmc = TASK_BMC.get().unwrap().lock().unwrap();
            let uuid = uuid::Uuid::parse_str(&id).unwrap();
            task_bmc.borrow_mut().$op(uuid).unwrap();
        }
    };
    ($($op: ident),+) => {
        $(gen_tauri_task_handler![$op];)+
    }
}

gen_tauri_task_handler![cancel, pause, continue_];

fn main() {
    crate::tracing_helper::init_tracing_subscriber();
    crate::config::config_init().unwrap();
    TASK_BMC.get_or_init(|| Mutex::new(RefCell::new(TaskBmc::new())));
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            echo, create, cancel, pause, continue_
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

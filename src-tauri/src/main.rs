// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(type_alias_impl_trait)]

use model::TaskBmc;
use std::{
    cell::RefCell,
    collections::HashMap,
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
fn create(url: String) -> String {
    let task_bmc = TASK_BMC.get().unwrap().lock().unwrap();
    let uuid = task_bmc.borrow_mut().create(url).unwrap();
    uuid.to_string()
}

macro_rules! gen_tauri_task_handler {
    ($op: ident) => {
        #[tauri::command]
        fn $op(id: String) {
            let task_bmc = TASK_BMC.get().unwrap().lock().unwrap();
            let uuid = uuid::Uuid::parse_str(&id).unwrap();
            task_bmc.borrow_mut().$op(uuid).ok();   // TODO error handling
        }
    };
    ($($op: ident),+) => {
        $(gen_tauri_task_handler![$op];)+
    }
}

gen_tauri_task_handler![cancel, pause, continue_, remove];

#[tauri::command]
fn show_config() -> HashMap<String, String> {
    crate::config::show_config().unwrap_or_default()
}

#[tauri::command]
fn upgrade_config(json: HashMap<String, String>) {
    crate::config::upgrade_config(json).ok();
}

#[tauri::command]
fn progress() -> Vec<(String, usize, usize, String, String)> {
    let task_bmc = TASK_BMC.get().unwrap().lock().unwrap();
    let ret = task_bmc.borrow().progress().unwrap();
    ret
}

fn main() {
    crate::tracing_helper::init_tracing_subscriber();
    TASK_BMC.get_or_init(|| Mutex::new(RefCell::new(TaskBmc::new())));
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            echo,
            create,
            cancel,
            pause,
            continue_,
            remove,
            progress,
            show_config,
            upgrade_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

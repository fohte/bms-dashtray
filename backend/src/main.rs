// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod config;
mod db_reader;
pub mod history_store;

use std::sync::Mutex;

use commands::{ConfigManagerState, HistoryStoreState};
use config::ConfigManager;
use history_store::HistoryStore;
use tauri::Manager as _;

fn main() {
    if let Err(e) = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;

            let config_path = app_data_dir.join("config.json");
            app.manage(ConfigManagerState(Mutex::new(ConfigManager::new(
                config_path,
            ))));

            let history_path = app_data_dir.join("history.json");
            let mut store = HistoryStore::new(history_path, "05:00");
            if let Err(e) = store.restore() {
                eprintln!("failed to restore history: {e}");
            }
            app.manage(HistoryStoreState(Mutex::new(store)));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::validate_and_save_config,
            commands::update_settings,
            commands::get_today_records,
            commands::reset_history,
        ])
        .run(tauri::generate_context!())
    {
        eprintln!("error while running tauri application: {e}");
        std::process::exit(1);
    }
}

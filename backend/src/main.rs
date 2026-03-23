// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};

use bms_dashtray::commands;
use bms_dashtray::commands::{ConfigManagerState, HistoryStoreState};
use bms_dashtray::config::ConfigManager;
use bms_dashtray::event_bridge::TauriEventEmitter;
use bms_dashtray::history_store::HistoryStore;
use bms_dashtray::pipeline;
use tauri::Manager as _;

fn main() {
    if let Err(e) = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;

            let config_path = app_data_dir.join("config.json");
            let config_manager = ConfigManager::new(config_path);
            let loaded_config = config_manager.load().ok().flatten();
            let reset_time = loaded_config
                .as_ref()
                .map(|c| c.reset_time.clone())
                .unwrap_or_else(|| "05:00".to_string());
            app.manage(ConfigManagerState(Mutex::new(config_manager)));

            let history_path = app_data_dir.join("history.json");
            let mut store = HistoryStore::new(history_path, &reset_time);
            if let Err(e) = store.restore() {
                eprintln!("failed to restore history: {e}");
            }
            let store = Arc::new(Mutex::new(store));
            app.manage(HistoryStoreState(Arc::clone(&store)));

            // Start the pipeline if config is available
            if let Some(ref config) = loaded_config {
                let emitter = Arc::new(TauriEventEmitter::new(app.handle().clone()));
                match pipeline::start_pipeline(config, Arc::clone(&store), emitter) {
                    Ok(handle) => {
                        // Store the handle so it lives as long as the app
                        app.manage(handle);
                    }
                    Err(e) => {
                        eprintln!("failed to start pipeline: {e}");
                    }
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::detect_players,
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

use std::sync::Mutex;

use tauri::State;

use crate::config::{AppConfig, ConfigManager};

pub struct ConfigManagerState(pub Mutex<ConfigManager>);

#[tauri::command]
pub fn get_config(state: State<'_, ConfigManagerState>) -> Result<AppConfig, String> {
    let manager = state
        .0
        .lock()
        .map_err(|e| format!("failed to acquire lock: {e}"))?;
    manager.load().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn validate_and_save_config(
    state: State<'_, ConfigManagerState>,
    beatoraja_root: String,
    player_name: String,
) -> Result<(), String> {
    let manager = state
        .0
        .lock()
        .map_err(|e| format!("failed to acquire lock: {e}"))?;
    manager
        .validate_and_save(&beatoraja_root, &player_name)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_settings(
    state: State<'_, ConfigManagerState>,
    reset_time: Option<String>,
    background_transparent: Option<bool>,
    font_size: Option<i32>,
) -> Result<(), String> {
    let manager = state
        .0
        .lock()
        .map_err(|e| format!("failed to acquire lock: {e}"))?;
    manager
        .update_settings(
            reset_time.as_deref(),
            background_transparent,
            font_size,
        )
        .map_err(|e| e.to_string())
}

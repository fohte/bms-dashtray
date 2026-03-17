use std::sync::Mutex;

use serde::Serialize;
use tauri::State;

use crate::config::{AppConfig, ConfigError, ConfigManager};

pub struct ConfigManagerState(pub Mutex<ConfigManager>);

#[derive(Debug, Serialize)]
#[serde(tag = "kind", content = "message")]
pub enum CommandError {
    Lock(String),
    Config(String),
}

impl From<ConfigError> for CommandError {
    fn from(err: ConfigError) -> Self {
        CommandError::Config(err.to_string())
    }
}

#[tauri::command]
pub fn get_config(state: State<'_, ConfigManagerState>) -> Result<AppConfig, CommandError> {
    let manager = state
        .0
        .lock()
        .map_err(|e| CommandError::Lock(e.to_string()))?;
    manager.load().map_err(Into::into)
}

#[tauri::command]
pub fn validate_and_save_config(
    state: State<'_, ConfigManagerState>,
    beatoraja_root: String,
    player_name: String,
) -> Result<(), CommandError> {
    let manager = state
        .0
        .lock()
        .map_err(|e| CommandError::Lock(e.to_string()))?;
    manager
        .validate_and_save(&beatoraja_root, &player_name)
        .map_err(Into::into)
}

#[tauri::command]
pub fn update_settings(
    state: State<'_, ConfigManagerState>,
    reset_time: Option<String>,
    background_transparent: Option<bool>,
    font_size: Option<i32>,
) -> Result<(), CommandError> {
    let manager = state
        .0
        .lock()
        .map_err(|e| CommandError::Lock(e.to_string()))?;
    manager
        .update_settings(reset_time.as_deref(), background_transparent, font_size)
        .map_err(Into::into)
}

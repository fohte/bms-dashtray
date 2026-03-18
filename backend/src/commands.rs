use std::sync::Mutex;

use tauri::State;

use crate::config::{AppConfig, ConfigError, ConfigManager};

pub struct ConfigManagerState(pub Mutex<ConfigManager>);

#[derive(Debug)]
pub enum CommandError {
    Lock(String),
    Config(String),
}

// Serialize as a plain string so the frontend receives a readable error message
// rather than a JSON object that would stringify as "[object Object]".
impl serde::Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let message = match self {
            CommandError::Lock(msg) => msg,
            CommandError::Config(msg) => msg,
        };
        serializer.serialize_str(message)
    }
}

impl From<ConfigError> for CommandError {
    fn from(err: ConfigError) -> Self {
        CommandError::Config(err.to_string())
    }
}

#[tauri::command]
pub fn get_config(state: State<'_, ConfigManagerState>) -> Result<Option<AppConfig>, CommandError> {
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
) -> Result<(), CommandError> {
    let manager = state
        .0
        .lock()
        .map_err(|e| CommandError::Lock(e.to_string()))?;
    manager
        .validate_and_save(&beatoraja_root)
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

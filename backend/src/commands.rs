use std::sync::Mutex;

use tauri::State;

use crate::config::{AppConfig, ConfigError, ConfigManager};
use crate::history_store::{HistoryStore, PlayRecord, StoreError};

pub struct ConfigManagerState(pub Mutex<ConfigManager>);
pub struct HistoryStoreState(pub Mutex<HistoryStore>);

#[derive(Debug)]
pub enum CommandError {
    Lock(String),
    Config(String),
    Store(String),
}

// Serialize as a plain string so the frontend receives a readable error message
// rather than a JSON object that would stringify as "[object Object]".
impl serde::Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let message = match self {
            CommandError::Lock(msg) | CommandError::Config(msg) | CommandError::Store(msg) => msg,
        };
        serializer.serialize_str(message)
    }
}

impl From<ConfigError> for CommandError {
    fn from(err: ConfigError) -> Self {
        CommandError::Config(err.to_string())
    }
}

impl From<StoreError> for CommandError {
    fn from(err: StoreError) -> Self {
        CommandError::Store(err.to_string())
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
    config_state: State<'_, ConfigManagerState>,
    history_state: State<'_, HistoryStoreState>,
    reset_time: Option<String>,
    background_transparent: Option<bool>,
    font_size: Option<i32>,
) -> Result<(), CommandError> {
    let manager = config_state
        .0
        .lock()
        .map_err(|e| CommandError::Lock(e.to_string()))?;
    manager.update_settings(reset_time.as_deref(), background_transparent, font_size)?;

    if let Some(ref rt) = reset_time {
        let mut store = history_state
            .0
            .lock()
            .map_err(|e| CommandError::Lock(e.to_string()))?;
        store.set_reset_time(rt);
    }

    Ok(())
}

#[tauri::command]
pub fn get_today_records(
    state: State<'_, HistoryStoreState>,
) -> Result<Vec<PlayRecord>, CommandError> {
    let store = state
        .0
        .lock()
        .map_err(|e| CommandError::Lock(e.to_string()))?;
    store.get_today_records().map_err(Into::into)
}

#[tauri::command]
pub fn reset_history(state: State<'_, HistoryStoreState>) -> Result<(), CommandError> {
    let mut store = state
        .0
        .lock()
        .map_err(|e| CommandError::Lock(e.to_string()))?;
    store.reset().map_err(Into::into)
}

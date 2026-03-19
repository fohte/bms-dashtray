use serde::Serialize;

use crate::history_store::PlayRecord;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoresUpdatedPayload {
    pub records: Vec<PlayRecord>,
    pub updated_at: String,
}

/// Trait abstracting the Tauri emit API for testability.
pub trait EventEmitter: Send + Sync {
    fn emit_scores_updated(&self, payload: ScoresUpdatedPayload) -> Result<(), String>;
}

/// Real implementation using Tauri's AppHandle.
pub struct TauriEventEmitter {
    app_handle: tauri::AppHandle,
}

impl TauriEventEmitter {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self { app_handle }
    }
}

impl EventEmitter for TauriEventEmitter {
    fn emit_scores_updated(&self, payload: ScoresUpdatedPayload) -> Result<(), String> {
        use tauri::Emitter as _;
        self.app_handle
            .emit("scores-updated", payload)
            .map_err(|e| e.to_string())
    }
}

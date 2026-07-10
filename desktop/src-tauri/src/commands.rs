//! Loon desktop IPC — config only. All library data comes from the backend API.

use serde::Serialize;
use tauri::{plugin::TauriPlugin, Runtime, State};

use crate::config::LoonDesktopConfig;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopConfigResponse {
    pub server_url: String,
    pub config_path: String,
}

#[tauri::command]
fn get_config(config: State<'_, LoonDesktopConfig>) -> DesktopConfigResponse {
    DesktopConfigResponse {
        server_url: config.server_url.clone(),
        config_path: config.config_path.display().to_string(),
    }
}

pub fn loon_plugin<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::new("loon")
        .invoke_handler(tauri::generate_handler![get_config])
        .build()
}

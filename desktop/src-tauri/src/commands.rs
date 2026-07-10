//! Loon desktop IPC — config and playback.

use serde::Serialize;
use tauri::plugin::TauriPlugin;
use tauri::{AppHandle, Emitter, Manager, Runtime, State};

use crate::config::LoonDesktopConfig;

const PLAYER_WINDOW_LABEL: &str = "player";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopConfigResponse {
    pub server_url: String,
    pub config_path: String,
    pub player_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayStreamResponse {
    pub stream_url: String,
    pub player_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PlayerLoadEvent {
    slug: String,
    title: String,
    stream_url: String,
}

fn show_player_window<R: Runtime>(
    app: &AppHandle<R>,
    slug: &str,
    title: &str,
    stream_url: &str,
) -> Result<(), String> {
    eprintln!(
        "loon-desktop: show player slug={slug} title={title} stream={stream_url}"
    );

    let window = app.get_webview_window(PLAYER_WINDOW_LABEL).ok_or_else(|| {
        let message = "player window is not registered in tauri.conf.json";
        eprintln!("loon-desktop: {message}");
        message.to_string()
    })?;

    let payload = PlayerLoadEvent {
        slug: slug.to_string(),
        title: title.to_string(),
        stream_url: stream_url.to_string(),
    };

    window
        .emit("player:load", &payload)
        .map_err(|error| format!("emit player:load failed: {error}"))?;
    window
        .set_title(title)
        .map_err(|error| format!("set player title failed: {error}"))?;
    window
        .show()
        .map_err(|error| format!("show player window failed: {error}"))?;
    window
        .set_focus()
        .map_err(|error| format!("focus player window failed: {error}"))?;

    eprintln!("loon-desktop: player window ready");
    Ok(())
}

#[tauri::command]
fn get_config(config: State<'_, LoonDesktopConfig>) -> DesktopConfigResponse {
    DesktopConfigResponse {
        server_url: config.server_url.clone(),
        config_path: config.config_path.display().to_string(),
        player_path: config.player_path.clone(),
    }
}

#[tauri::command]
fn play_stream<R: Runtime>(
    app: AppHandle<R>,
    config: State<'_, LoonDesktopConfig>,
    slug: String,
    title: Option<String>,
) -> Result<PlayStreamResponse, String> {
    let stream_url = format!(
        "{}/stream/{}",
        config.server_url.trim().trim_end_matches('/'),
        urlencoding::encode(&slug)
    );
    let display_title = title
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(slug.as_str())
        .to_string();

    show_player_window(&app, &slug, &display_title, &stream_url)?;

    Ok(PlayStreamResponse {
        stream_url,
        player_path: "tauri-window".to_string(),
    })
}

pub fn loon_plugin<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::new("loon")
        .invoke_handler(tauri::generate_handler![get_config, play_stream])
        .build()
}

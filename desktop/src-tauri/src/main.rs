#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod config;
mod config_host;

use nest_tauri::TauriApp;
use nest_theme::ThemeModule;

use crate::commands::loon_plugin;
use crate::config::LoonDesktopConfig;
use crate::config_host::require_config_path;

fn main() {
    let config_path = require_config_path();
    let desktop_config = match LoonDesktopConfig::load(&config_path) {
        Ok(config) => config,
        Err(error) => {
            eprintln!(
                "loon-desktop: failed to load {}: {error}",
                config_path.display()
            );
            std::process::exit(1);
        }
    };

    eprintln!(
        "loon-desktop: config={} api={}",
        desktop_config.config_path.display(),
        desktop_config.server_url
    );

    TauriApp::new("loon-admin")
        .with_config_path(&config_path)
        .module(ThemeModule::default())
        .with_builder(move |builder| {
            builder
                .manage(desktop_config)
                .plugin(loon_plugin())
                .plugin(tauri_plugin_opener::init())
        })
        .run(tauri::generate_context!());
}

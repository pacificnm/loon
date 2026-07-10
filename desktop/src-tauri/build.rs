const LOON_COMMANDS: &[&str] = &["get_config", "play_stream"];

fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new().plugin(
            "loon",
            tauri_build::InlinedPlugin::new()
                .commands(LOON_COMMANDS)
                .default_permission(tauri_build::DefaultPermissionRule::AllowAllCommands),
        ),
    )
    .expect("failed to run tauri build");
}

//! Launch ffplay, mpv, or another configured player for HTTP streams.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use nest_error::{NestError, NestResult};
use tracing::info;

/// Resolves the external player binary from config or `PATH`.
pub fn resolve_player_path(configured: Option<&str>) -> NestResult<PathBuf> {
    if let Some(path) = configured.map(str::trim).filter(|value| !value.is_empty()) {
        let candidate = PathBuf::from(path);
        if candidate.is_absolute() {
            if is_executable(&candidate) {
                return Ok(candidate);
            }
            return Err(NestError::config(format!(
                "configured player_path is not executable: {}",
                candidate.display()
            )));
        }
        if let Some(found) = find_in_path(path) {
            return Ok(found);
        }
        return Err(NestError::config(format!(
            "configured player_path not found in PATH: {path}"
        )));
    }

    for name in ["ffplay", "mpv", "mplayer"] {
        if let Some(found) = find_in_path(name) {
            info!(player = %found.display(), "using external video player");
            return Ok(found);
        }
    }

    Err(NestError::config(
        "no video player found: install ffmpeg (ffplay) or mpv, or set [loon-admin].player_path",
    ))
}

/// Spawns the external player detached for an HTTP stream URL.
pub fn launch_external_player(
    player: &Path,
    url: &str,
    title: Option<&str>,
) -> NestResult<()> {
    if url.trim().is_empty() {
        return Err(NestError::config("stream url must not be empty"));
    }

    let file_name = player
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or_default()
        .to_ascii_lowercase();

    let mut command = Command::new(player);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    if file_name.contains("ffplay") {
        command.arg("-autoexit");
        if let Some(title) = title.filter(|value| !value.is_empty()) {
            command.arg("-window_title").arg(title);
        }
        command.arg(url);
    } else if file_name.contains("mpv") {
        if let Some(title) = title.filter(|value| !value.is_empty()) {
            command.arg(format!("--title={title}"));
        }
        command.arg(url);
    } else {
        if let Some(title) = title.filter(|value| !value.is_empty()) {
            command.arg(title);
        }
        command.arg(url);
    }

    command.spawn().map_err(|error| {
        NestError::service(format!(
            "failed to launch {}: {error}",
            player.display()
        ))
    })?;

    info!(
        url,
        player = %player.display(),
        "launched external player"
    );
    Ok(())
}

/// Builds `{server_url}/stream/{slug}` and launches the configured player.
pub fn play_movie_stream(
    server_url: &str,
    slug: &str,
    title: Option<&str>,
    player_path: Option<&str>,
) -> NestResult<()> {
    let player = resolve_player_path(player_path)?;
    let encoded = urlencoding::encode(slug);
    let url = format!(
        "{}/stream/{}",
        server_url.trim().trim_end_matches('/'),
        encoded
    );
    launch_external_player(&player, &url, title)
}

fn is_executable(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        return path
            .metadata()
            .map(|meta| meta.permissions().mode() & 0o111 != 0)
            .unwrap_or(false);
    }
    #[cfg(not(unix))]
    {
        true
    }
}

fn find_in_path(name: &str) -> Option<PathBuf> {
    let paths = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&paths) {
        let candidate = dir.join(name);
        if is_executable(&candidate) {
            return Some(candidate);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_prefers_configured_absolute_path() {
        let path = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
        let resolved = resolve_player_path(Some(&path)).unwrap();
        assert_eq!(resolved, PathBuf::from(path));
    }

    #[test]
    fn play_url_encodes_slug() {
        let dir = tempfile::tempdir().unwrap();
        let script = dir.path().join("fake-ffplay");
        std::fs::write(&script, "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        let result = launch_external_player(&script, "http://127.0.0.1:3000/stream/a%20b", Some("Test"));
        assert!(result.is_ok());
    }
}

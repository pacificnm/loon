//! Loon desktop config location — `~/.config/loon/config.toml` only.

use std::path::PathBuf;

/// Returns `~/.config/loon`.
pub fn loon_config_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(|home| PathBuf::from(home).join(".config").join("loon"))
        .unwrap_or_else(|| PathBuf::from(".config").join("loon"))
}

/// Required config path: `~/.config/loon/config.toml`.
pub fn loon_config_path() -> PathBuf {
    loon_config_dir().join("config.toml")
}

/// Returns the config path or exits the process when missing.
pub fn require_config_path() -> PathBuf {
    let path = loon_config_path();
    if path.is_file() {
        return path;
    }
    eprintln!(
        "loon-desktop: configuration file not found: {}",
        path.display()
    );
    eprintln!("loon-desktop: create ~/.config/loon/config.toml with [loon-admin].server_url");
    std::process::exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn loon_config_path_under_home() {
        let dir = tempdir().unwrap();
        let home = dir.path().join("home");
        fs::create_dir_all(&home).unwrap();
        let original = std::env::var("HOME").ok();
        unsafe {
            std::env::set_var("HOME", &home);
        }
        assert_eq!(
            loon_config_path(),
            home.join(".config").join("loon").join("config.toml")
        );
        if let Some(value) = original {
            unsafe {
                std::env::set_var("HOME", value);
            }
        } else {
            unsafe {
                std::env::remove_var("HOME");
            }
        }
    }
}

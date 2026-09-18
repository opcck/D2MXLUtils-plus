use std::path::PathBuf;
use std::sync::OnceLock;

static APP_CONFIG_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Returns the portable configuration and data directory: `<exe_dir>/config`
pub fn get_app_dir() -> PathBuf {
    APP_CONFIG_DIR
        .get_or_init(|| {
            let dir = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| PathBuf::from("."))
                .join("config");

            if !dir.exists() {
                let _ = std::fs::create_dir_all(&dir);
            }
            dir
        })
        .clone()
}

use std::path::PathBuf;

use crate::CONFIG;

pub(crate) fn default_archive_dir() -> PathBuf {
    // Highest priority: user‑supplied (env or API setter)
    let config = CONFIG.read();
    if let Ok(cfg) = config {
        if let Some(path) = &cfg.archive_dir {
            return path.clone();
        }
    }

    // XDG on Unix, %LOCALAPPDATA% on Windows, ~/Library on macOS …
    dirs::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("loggit")
        .join("archives")
}

pub(crate) fn archive_dir() -> PathBuf {
    static DIR: once_cell::sync::Lazy<PathBuf> = once_cell::sync::Lazy::new(default_archive_dir);
    DIR.clone() // cheap Arc‑style clone of PathBuf
}

pub(crate) fn ensure_archivable_dir(path: &PathBuf) -> std::io::Result<()> {
    std::fs::create_dir_all(path)
}

pub(crate) fn ensure_archive_dir() -> std::io::Result<()> {
    std::fs::create_dir_all(archive_dir())
}

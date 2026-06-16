use log::{error, info};
use std::path::PathBuf;

use crate::public::constant::storage::get_data_path;
use crate::public::structure::config::APP_CONFIG;

/// Base directory relative `imagePath` entries are resolved against.
///
/// `UROCISSA_IMAGE_HOME`, if set, is used directly. Otherwise defaults to an
/// `images/` subdirectory of the resolved data home (`UROCISSA_DATA_HOME` —
/// see `constant::storage`), not the working directory: the cwd is
/// arbitrary depending on how the binary was launched (systemd unit, Docker
/// `WORKDIR`, a desktop shortcut), so it's not a reliable place to expect
/// media to already exist. The data home, by contrast, is already resolved
/// to a stable, predictable location. Multiple physical photo/video
/// libraries are expected to be aggregated under this one root at the
/// filesystem layer (bind mounts or symlinks) rather than configured as a
/// list here.
fn image_home_base() -> PathBuf {
    if let Ok(p) = std::env::var("UROCISSA_IMAGE_HOME") {
        return PathBuf::from(p);
    }

    let default_dir = get_data_path().join("images");
    if !default_dir.exists()
        && let Err(e) = std::fs::create_dir_all(&default_dir)
    {
        error!(
            "Failed to create default image directory {}: {e}",
            default_dir.display()
        );
    }
    info!(
        "UROCISSA_IMAGE_HOME not set. Defaulting to {}",
        default_dir.display()
    );
    default_dir
}

/// Resolve the configured `imagePath` to an absolute path, if set.
pub fn resolve_image_path(path: Option<PathBuf>) -> Option<PathBuf> {
    path.map(|p| {
        if p.is_relative() {
            let resolved = image_home_base().join(&p);
            resolved.canonicalize().unwrap_or(resolved)
        } else {
            p
        }
    })
}

/// Get the resolved image path from the current config.
pub fn get_resolved_image_path() -> Option<PathBuf> {
    let raw = APP_CONFIG
        .get()
        .unwrap()
        .read()
        .unwrap()
        .public
        .image_path
        .clone();
    resolve_image_path(raw)
}

use std::path::PathBuf;

use crate::public::structure::config::APP_CONFIG;

/// Get the image path from the current config.
/// After initialization, `imagePath` is always an absolute path stored in config.
pub fn get_resolved_image_home() -> Option<PathBuf> {
    APP_CONFIG
        .get()
        .expect("APP_CONFIG not initialized")
        .read()
        .expect("RwLock poisoned")
        .image_home
        .clone()
}

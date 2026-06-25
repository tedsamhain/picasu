use directories::ProjectDirs;
use log::{error, info};
use std::path::Path;
use std::path::PathBuf;
use std::sync::OnceLock;

pub static DATA_PATH: OnceLock<PathBuf> = OnceLock::new();
pub static CONFIG_DIR: OnceLock<PathBuf> = OnceLock::new();

const CONFIG_FILE_NAME: &str = "config.toml";

/// Resolves a root directory using a fixed precedence:
/// 1. `env_var`, if set — explicit override (e.g. `UROCISSA_DATA_HOME`).
/// 2. The legacy single-folder layout: if `./config.toml` already exists in
///    the working directory, pre-dates the split between config/data
///    locations below and must keep working unchanged.
/// 3. The OS-standard directory for `kind` (XDG on Linux, equivalent
///    conventions on Windows/macOS via the `directories` crate — which
///    itself honors `$XDG_CONFIG_HOME`/`$XDG_DATA_HOME` if set).
/// 4. The working directory, as a last resort if the OS directory can't be
///    determined at all.
pub fn resolve_root(
    env_var: &str,
    kind: &str,
    os_dir: impl FnOnce(&ProjectDirs) -> PathBuf,
) -> PathBuf {
    if let Ok(p) = std::env::var(env_var) {
        let dir = PathBuf::from(p);
        if let Err(e) = std::fs::create_dir_all(&dir) {
            error!(
                "Failed to create {env_var} directory {}: {e}",
                dir.display()
            );
        }
        info!(
            "{env_var} override detected. Using {kind} directory: {}",
            dir.display()
        );
        return dir;
    }

    // Legacy back-compat: every pre-existing install has `config.toml`
    // sitting next to its `db`/`object`/`upload` folders in the working
    // directory. Detecting that file is a more precise signal than the old
    // "does ./db or ./object exist" sniff, and covers both roots with one
    // check.
    if Path::new(CONFIG_FILE_NAME).exists() {
        info!("Legacy single-folder layout detected — using cwd for {kind}");
        return PathBuf::from(".");
    }

    if let Some(proj_dirs) = ProjectDirs::from("com", "urocissa", "urocissa") {
        let dir = os_dir(&proj_dirs);

        if !dir.exists()
            && let Err(e) = std::fs::create_dir_all(&dir)
        {
            error!("Failed to create {kind} directory {}: {e}", dir.display());
            return PathBuf::from(".");
        }

        info!("Using OS-standard {kind} directory: {}", dir.display());
        return dir;
    }

    info!("Could not determine system {kind} directory. Defaulting to cwd.");
    PathBuf::from(".")
}

/// Returns the path to `config.toml`. See [`get_config_dir`] for how the
/// containing directory is resolved.
pub fn get_config_path() -> PathBuf {
    get_config_dir().join(CONFIG_FILE_NAME)
}

/// Directory holding `config.toml`. Override with `UROCISSA_CONFIG_HOME`;
/// otherwise resolved independently of [`get_data_path`] (see [`resolve_root`]).
pub fn get_config_dir() -> &'static PathBuf {
    CONFIG_DIR.get_or_init(|| {
        resolve_root("UROCISSA_CONFIG_HOME", "config", |p| {
            p.config_dir().to_path_buf()
        })
    })
}

/// Directory holding `db/` and `object/compressed/` — derived index and
/// thumbnail/preview cache only. Override with `UROCISSA_DATA_HOME`.
/// Originals are never duplicated here; `IMAGE_HOME` is the single copy
/// (see `TODO.md` "Storage architecture fix"). This is still real,
/// back-up-worthy data — `db/index_v5.redb` is the only store of record for
/// tags/album-assignments/flags. (A handful of files under `db/` —
/// `cache_db.redb`, `temp_db.redb`, `expire_db.redb` — *are* safely
/// disposable; splitting those into a dedicated `UROCISSA_STATE_HOME` is a
/// possible follow-up, not yet done.)
pub fn get_data_path() -> &'static PathBuf {
    DATA_PATH
        .get_or_init(|| resolve_root("UROCISSA_DATA_HOME", "data", |p| p.data_dir().to_path_buf()))
}

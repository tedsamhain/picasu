use std::path::PathBuf;
use std::sync::{LazyLock, Mutex, RwLock};

use tempfile::TempDir;

use crate::model::config::{APP_CONFIG, AppConfig};
use crate::router::builder::build_rocket_with_config;
use crate::storage::db::DATA_TABLE;
use crate::storage::db::TREE;
use crate::storage::files::DATA_PATH;
use crate::storage::files::get_resolved_image_home;
use rocket::local::blocking::Client;

/// Holds the tempdir alive for the entire test binary run.
pub struct TestEnv {
    pub _dir: TempDir,
}

pub static TEST_ENV: LazyLock<TestEnv> = LazyLock::new(|| {
    let dir = tempfile::tempdir().expect("create tempdir");
    let data_path = dir.path().to_path_buf();

    DATA_PATH
        .set(data_path.clone())
        .expect("DATA_PATH already set");

    let mut test_config = AppConfig::default();
    let image_home = data_path.join("images");
    std::fs::create_dir_all(&image_home).unwrap();
    test_config.image_home = Some(image_home);
    APP_CONFIG
        .set(RwLock::new(test_config))
        .expect("APP_CONFIG already set");

    {
        let txn = TREE.in_disk.begin_write().expect("begin write txn");
        txn.open_table(DATA_TABLE).expect("create DATA_TABLE");
        txn.commit().expect("commit");
    }

    TestEnv { _dir: dir }
});

/// The image home path used by the test environment.
/// Derived from the test config that TEST_ENV sets up.
pub fn test_image_home() -> PathBuf {
    let _ = &*TEST_ENV;
    get_resolved_image_home().expect("IMAGE_HOME must be configured in test config")
}

/// Serialize the fields from `updates` into the config and write a
/// config.toml to the test directory for audit.  Affects the next call to
/// `make_client()` (the guard reads `APP_CONFIG` at request time).
pub fn write_config(updates: &serde_json::Value) {
    let mut config = APP_CONFIG.get().unwrap().write().unwrap();
    if let Some(obj) = updates.as_object() {
        if let Some(val) = obj.get("read_only_mode").and_then(|v| v.as_bool()) {
            config.read_only_mode = val;
        }
    }
    // Write a copy to disk for documentation/debugging.
    use serde::Serialize;
    #[derive(Serialize)]
    struct ConfigFile<'a> {
        picasu: &'a AppConfig,
    }
    let data_path = DATA_PATH.get().expect("DATA_PATH set");
    let config_path = data_path.join("config.toml");
    if let Ok(toml) = toml::to_string_pretty(&ConfigFile { picasu: &*config }) {
        let _ = std::fs::write(&config_path, toml);
    }
}

/// There is exactly one global album-index slot in `album_index.rs`.
/// All generated tests that call `POST /post/index/album` must hold this
/// lock for their entire body to get `202 Accepted` instead of `409 Conflict`.
pub static INDEX_SERIAL_GUARD: Mutex<()> = Mutex::new(());

/// Build a Rocket test client with the current APP_CONFIG.
pub fn make_client() -> Client {
    let _ = &*TEST_ENV;
    let config = APP_CONFIG.get().unwrap().read().unwrap().clone();
    Client::tracked(build_rocket_with_config(config)).expect("valid rocket instance")
}

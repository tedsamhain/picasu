use std::sync::{LazyLock, Mutex, RwLock};

use rocket::local::blocking::Client;
use tempfile::TempDir;

use crate::public::structure::config::AppConfig;
use crate::router::builder::build_rocket_with_config;

pub use std::collections::HashMap;
pub use std::path::{Path, PathBuf};

pub use arrayvec::ArrayString;
pub use redb::{ReadableDatabase, ReadableTable};
pub use rocket::http::{ContentType, Status};
pub use serde_json::Value;

pub use crate::operations::hash::{blake3_hasher, generate_random_hash};
pub use crate::public::constant::redb::DATA_TABLE;
pub use crate::public::constant::storage::DATA_PATH;
pub use crate::public::db::tree::TREE;
pub use crate::public::structure::abstract_data::AbstractData;
pub use crate::public::structure::common::file_modify::FileModify;
pub use crate::public::structure::config::APP_CONFIG;
pub use crate::public::structure::image::combined::ImageCombined;
pub use crate::public::structure::image::metadata::ImageMetadata;
pub use crate::public::structure::object::{ObjectSchema, ObjectType};
pub use crate::tasks::actor::album::album_task;
pub use crate::tasks::actor::folder_import::{
    FolderImportState, folder_import_status, start_image_home_scan,
};
pub use crate::workflow::index_for_watch;

// ─── One-time test environment ────────────────────────────────────────────

/// Holds the tempdir alive for the entire test binary run.
///
/// Scenario A's "initial empty state" assertions live here so they execute
/// exactly once before any test body has a chance to insert data.
pub struct TestEnv {
    pub _dir: TempDir,
    /// Assertions captured during init (None = passed, Some(msg) = failed)
    pub init_assertions: Vec<String>,
}

pub static TEST_ENV: LazyLock<TestEnv> = LazyLock::new(|| {
    let dir = tempfile::tempdir().expect("create tempdir");

    DATA_PATH
        .set(dir.path().to_path_buf())
        .expect("DATA_PATH already set");

    // No password → GuardAuth auto-succeeds; read_only_mode = false.
    APP_CONFIG
        .set(RwLock::new(AppConfig::default()))
        .expect("APP_CONFIG already set");

    // Create DATA_TABLE so read-only handlers don't fail on an empty DB.
    {
        let txn = TREE.in_disk.begin_write().expect("begin write txn");
        txn.open_table(DATA_TABLE).expect("create DATA_TABLE");
        txn.commit().expect("commit");
    }

    // ── Scenario A: verify initial empty state ────────────────────────────
    // Run here so no concurrent test can pollute the DB before we check.
    let client = Client::tracked(build_rocket_with_config(AppConfig::default())).expect("rocket");
    let mut failures = Vec::new();

    // Config endpoint: hasPassword=false, readOnlyMode=false, hasAuthKey=false
    let config_resp = client.get("/get/config").dispatch();
    if config_resp.status() != Status::Ok {
        failures.push(format!(
            "GET /get/config: expected 200, got {:?}",
            config_resp.status()
        ));
    } else {
        let body: Value = serde_json::from_str(&config_resp.into_string().unwrap()).unwrap();
        for (k, want) in [
            ("hasPassword", false),
            ("readOnlyMode", false),
            ("hasAuthKey", false),
        ] {
            if body[k] != want {
                failures.push(format!("config.{k} expected {want}, got {}", body[k]));
            }
        }
    }

    // Albums list is empty on a fresh DB
    let albums_resp = client.get("/get/get-albums").dispatch();
    if albums_resp.status() != Status::Ok {
        failures.push(format!(
            "GET /get/get-albums: expected 200, got {:?}",
            albums_resp.status()
        ));
    } else {
        let body: Value = serde_json::from_str(&albums_resp.into_string().unwrap()).unwrap();
        let arr = body.as_array().expect("expected JSON array");
        if !arr.is_empty() {
            failures.push(format!(
                "expected empty album list, got {} items",
                arr.len()
            ));
        }
    }

    // Tags list is empty on a fresh DB
    let tags_resp = client.get("/get/get-tags").dispatch();
    if tags_resp.status() != Status::Ok {
        failures.push(format!(
            "GET /get/get-tags: expected 200, got {:?}",
            tags_resp.status()
        ));
    } else {
        let body: Value = serde_json::from_str(&tags_resp.into_string().unwrap()).unwrap();
        let arr = body.as_array().expect("expected JSON array");
        if !arr.is_empty() {
            failures.push(format!("expected empty tag list, got {} items", arr.len()));
        }
    }

    TestEnv {
        _dir: dir,
        init_assertions: failures,
    }
});

/// `TREE_SNAPSHOT` keys snapshots by `Utc::now().timestamp_millis()`
/// (get_prefetch.rs). Two `prefetch` calls from different tests landing
/// in the same millisecond will silently overwrite each other's
/// snapshot, corrupting whichever test reads it second — a real
/// concurrency bug, not just a test artifact (see TODO.md). Tests that
/// call `prefetch_locate` must hold this guard for their whole body to
/// avoid tripping over it while running in parallel with each other.
pub static PREFETCH_SERIAL_GUARD: Mutex<()> = Mutex::new(());

/// Minimal photo fixture: fake file path, tags, optional EXIF date string.
pub struct PhotoSpec<'a> {
    pub path: &'a str,
    pub tags: &'a [&'a str],
    pub exif_date: Option<&'a str>,
}

mod api;
mod internal;
pub use api::*;
pub use internal::*;

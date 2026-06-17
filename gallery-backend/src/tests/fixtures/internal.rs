use std::path::Path;

use arrayvec::ArrayString;
use redb::{ReadableDatabase, ReadableTable};

use crate::operations::hash::generate_random_hash;
use crate::public::constant::redb::DATA_TABLE;
use crate::public::db::tree::TREE;
use crate::public::structure::abstract_data::AbstractData;
use crate::public::structure::common::file_modify::FileModify;
use crate::public::structure::image::combined::ImageCombined;
use crate::public::structure::image::metadata::ImageMetadata;
use crate::public::structure::object::{ObjectSchema, ObjectType};
use crate::public::structure::response::database_timestamp::DatabaseTimestamp;

use super::{PhotoSpec, TEST_ENV};

/// Write Image records to redb with the given fake paths and refresh
/// TREE.in_memory.  No actual files are needed. Returns the generated
/// hashes so callers can reference them without a reverse lookup.
pub fn insert_photos(photos: &[PhotoSpec]) -> Vec<ArrayString<64>> {
    let _ = &*TEST_ENV;
    let mut hashes = Vec::with_capacity(photos.len());
    let txn = TREE.in_disk.begin_write().expect("begin write");
    {
        let mut table = txn.open_table(DATA_TABLE).expect("open table");
        for spec in photos {
            let hash = generate_random_hash();
            hashes.push(hash);
            let mut obj = ObjectSchema::new(hash, ObjectType::Image);
            for &tag in spec.tags {
                obj.tags.insert(tag.to_owned());
            }
            let mut meta = ImageMetadata::new(hash, 4096, 1920, 1080, "jpg".into());
            meta.alias = vec![FileModify {
                file: spec.path.to_owned(),
                modified: 0,
                scan_time: 0,
            }];
            if let Some(date) = spec.exif_date {
                meta.exif_vec.insert("DateTimeOriginal".into(), date.into());
            }
            table
                .insert(
                    hash.as_str(),
                    &AbstractData::Image(ImageCombined {
                        object: obj,
                        metadata: meta,
                    }),
                )
                .expect("insert photo");
        }
    }
    txn.commit().expect("commit photos");
    refresh_in_memory();
    hashes
}

/// Re-read all redb records into TREE.in_memory.
/// Replicates the synchronous core of `update_tree_task` without async.
pub fn refresh_in_memory() {
    let txn = TREE.in_disk.begin_read().expect("begin read");
    let table = txn.open_table(DATA_TABLE).expect("open table");
    let priority_list = &["DateTimeOriginal", "filename", "modified", "scan_time"];
    let mut vec: Vec<DatabaseTimestamp> = table
        .iter()
        .expect("iter")
        .map(|entry| {
            let (_, v) = entry.expect("entry");
            DatabaseTimestamp::new(v.value(), priority_list)
        })
        .collect();
    vec.sort_by_key(|b| std::cmp::Reverse(b.timestamp));
    *TREE.in_memory.write().unwrap() = vec;
}

/// Insert a photo backed by an actual file on disk and return its hash.
pub fn insert_photo_with_real_file(file_path: &Path) -> ArrayString<64> {
    let _ = &*TEST_ENV;
    assert!(file_path.exists(), "source file must exist: {file_path:?}");
    let hash = generate_random_hash();
    let txn = TREE.in_disk.begin_write().expect("begin write");
    {
        let mut table = txn.open_table(DATA_TABLE).expect("open table");
        let obj = ObjectSchema::new(hash, ObjectType::Image);
        let mut meta = ImageMetadata::new(hash, 1, 1, 1, "jpg".into());
        meta.alias = vec![FileModify {
            file: file_path.to_string_lossy().into_owned(),
            modified: 0,
            scan_time: 0,
        }];
        table
            .insert(
                hash.as_str(),
                &AbstractData::Image(ImageCombined {
                    object: obj,
                    metadata: meta,
                }),
            )
            .expect("insert");
    }
    txn.commit().expect("commit");
    refresh_in_memory();
    hash
}

/// Like `insert_photo_with_real_file`, but with an explicit hash and
/// deliberately stale/placeholder metadata (`width`/`height` 1x1, no
/// tags) — simulating a pre-existing, incomplete index entry (e.g. one
/// written before a metadata-extraction feature existed, or by an
/// older/buggy version). The hash must match `file_path`'s real content
/// hash (computed with the same hasher `HashTask` uses) for a
/// force-reindex to recognise it as "already known" rather than new.
pub fn insert_stale_photo_record(file_path: &Path, hash: ArrayString<64>) {
    assert!(file_path.exists(), "source file must exist: {file_path:?}");
    let txn = TREE.in_disk.begin_write().expect("begin write");
    {
        let mut table = txn.open_table(DATA_TABLE).expect("open table");
        let obj = ObjectSchema::new(hash, ObjectType::Image);
        let mut meta = ImageMetadata::new(hash, 1, 1, 1, "jpg".into());
        meta.alias = vec![FileModify {
            file: file_path.to_string_lossy().into_owned(),
            modified: 0,
            scan_time: 0,
        }];
        table
            .insert(
                hash.as_str(),
                &AbstractData::Image(ImageCombined {
                    object: obj,
                    metadata: meta,
                }),
            )
            .expect("insert");
    }
    txn.commit().expect("commit");
    refresh_in_memory();
}

/// Scan `DATA_TABLE` for the record whose first alias entry is `path`.
/// Used after running the real indexing pipeline, where the hash is
/// content-derived (blake3) and not known to the test ahead of time.
pub fn find_hash_by_alias_path(path: &Path) -> ArrayString<64> {
    let target = path.to_string_lossy().into_owned();
    let txn = TREE.in_disk.begin_read().expect("begin read");
    let table = txn.open_table(DATA_TABLE).expect("open table");
    table
        .iter()
        .expect("iter")
        .flatten()
        .find_map(|(_, v)| {
            let data = v.value();
            data.alias()
                .iter()
                .any(|a| a.file == target)
                .then(|| data.hash())
        })
        .unwrap_or_else(|| panic!("no indexed record found with alias path {target}"))
}

# Database

Urocissa uses [redb](https://github.com/cberner/redb) (v4), an embedded
key-value store, for all persistence. There is no SQL or external database
server. The data directory (see `get_data_path()` in
`src/public/constant/storage.rs`) contains one persistent DB and three
disposable cache DBs.

---

## Database files

| File                | Purpose                                  | Persistence        |
| ------------------- | ---------------------------------------- | ------------------ |
| `db/index_v5.redb`  | Store of record for all media/album data | Persistent         |
| `db/temp_db.redb`   | Tree snapshot cache (flat sorted view)   | Deleted at startup |
| `db/cache_db.redb`  | Query prefetch cache                     | Deleted at startup |
| `db/expire_db.redb` | Snapshot expiration timestamps           | Deleted at startup |

The three cache databases are intentionally ephemeral — they are deleted at the
start of `initialize_file()` (`src/operations/initialization/redb.rs`) and
rebuilt on demand. Only `index_v5.redb` carries data that requires backup.

---

## index_v5.redb — store of record

### Table: `"database"`

Maps a 64-char hex hash key (`&str`) to an `AbstractData` value.

redb table definition (`src/public/constant/redb.rs`):

```
DATA_TABLE: TableDefinition<&str, AbstractData> = TableDefinition::new("database")
```

### AbstractData variants

```rust
pub enum AbstractData {
    Image(ImageCombined),
    Video(VideoCombined),
    Album(AlbumCombined),
}
```

Every variant shares a common `ObjectSchema` plus a type-specific metadata
struct.

#### ObjectSchema (common to all variants)

Source: `src/public/structure/object.rs`

| Field         | Type              | Description                  |
| ------------- | ----------------- | ---------------------------- |
| `id`          | `ArrayString<64>` | Unique hash identifier       |
| `obj_type`    | `ObjectType`      | `Image`, `Video`, or `Album` |
| `pending`     | `bool`            | Processing-in-progress flag  |
| `thumbhash`   | `Option<Vec<u8>>` | Binary thumbnail hash        |
| `description` | `Option<String>`  | User-written description     |
| `tags`        | `HashSet<String>` | User-applied tags            |
| `is_favorite` | `bool`            | Favorited flag               |
| `is_archived` | `bool`            | Archived flag                |
| `is_trashed`  | `bool`            | Trashed flag                 |
| `update_at`   | `i64`             | Last-updated timestamp (ms)  |

#### ImageMetadata

Source: `src/public/structure/image/metadata.rs`

| Field      | Type                       | Description                   |
| ---------- | -------------------------- | ----------------------------- |
| `id`       | `ArrayString<64>`          | Hash                          |
| `size`     | `u64`                      | File size in bytes            |
| `width`    | `u32`                      | Pixel width                   |
| `height`   | `u32`                      | Pixel height                  |
| `ext`      | `String`                   | File extension (e.g. `"jpg"`) |
| `phash`    | `Option<Vec<u8>>`          | Perceptual hash               |
| `album`    | `Option<ArrayString<64>>`  | Single album membership       |
| `exif_vec` | `BTreeMap<String, String>` | EXIF key-value pairs          |
| `alias`    | `Vec<FileModify>`          | Known file paths & timestamps |

#### VideoMetadata

Source: `src/public/structure/video/metadata.rs`

| Field      | Type                       | Description                   |
| ---------- | -------------------------- | ----------------------------- |
| `id`       | `ArrayString<64>`          | Hash                          |
| `size`     | `u64`                      | File size in bytes            |
| `width`    | `u32`                      | Pixel width                   |
| `height`   | `u32`                      | Pixel height                  |
| `ext`      | `String`                   | File extension (e.g. `"mp4"`) |
| `duration` | `f64`                      | Video duration in seconds     |
| `album`    | `Option<ArrayString<64>>`  | Single album membership       |
| `exif_vec` | `BTreeMap<String, String>` | EXIF key-value pairs          |
| `alias`    | `Vec<FileModify>`          | Known file paths & timestamps |

#### AlbumMetadata

Source: `src/public/structure/album/metadata.rs`

| Field                | Type                              | Description                       |
| -------------------- | --------------------------------- | --------------------------------- |
| `id`                 | `ArrayString<64>`                 | Hash                              |
| `title`              | `Option<String>`                  | Display title                     |
| `created_time`       | `i64`                             | Creation timestamp (ms)           |
| `start_time`         | `Option<i64>`                     | Earliest media timestamp (ms)     |
| `end_time`           | `Option<i64>`                     | Latest media timestamp (ms)       |
| `last_modified_time` | `i64`                             | Last metadata update (ms)         |
| `cover`              | `Option<ArrayString<64>>`         | Cover image hash                  |
| `item_count`         | `usize`                           | Number of member media items      |
| `item_size`          | `u64`                             | Total member file size            |
| `share_list`         | `HashMap<ArrayString<64>, Share>` | Named share configurations        |
| `dir_path`           | `Option<String>`                  | Filesystem path (dir-albums only) |

`dir_path` distinguishes two album types:

- **Dir-albums**: `dir_path` is set. Membership is derived from source file
  parent paths matching this path exactly.
- **User albums**: `dir_path` is `None`. Membership is stored explicitly in
  each media item's `album` field.

#### FileModify

Source: `src/public/structure/common/file_modify.rs`

| Field       | Type     | Description                      |
| ----------- | -------- | -------------------------------- |
| `file`      | `String` | Absolute file path               |
| `modified`  | `i64`    | File modification timestamp (ms) |
| `scan_time` | `i64`    | Last scan timestamp (ms)         |

Media items can have multiple `FileModify` entries when a file was renamed or
hard-linked — each known path is tracked under the same hash.

#### Share

Source: `src/public/structure/album/share.rs`

| Field           | Type              | Description                          |
| --------------- | ----------------- | ------------------------------------ |
| `url`           | `ArrayString<64>` | Unique share token                   |
| `description`   | `String`          | Human-readable description           |
| `password`      | `Option<String>`  | Optional access password             |
| `show_metadata` | `bool`            | Allow metadata view                  |
| `show_download` | `bool`            | Allow download                       |
| `show_upload`   | `bool`            | Allow upload                         |
| `exp`           | `i64`             | Expiration timestamp (ms, 0 = never) |

---

## temp_db.redb — tree snapshot

Caches the tree-sorted view of all media, partitioned by timestamp bucket.

Structure: one dynamic table per timestamp value (`i64` as string table name),
each mapping `u64` (sequential index) → `ReducedData`.

```rust
pub struct ReducedData {
    pub hash: ArrayString<64>,
    pub width: u32,
    pub height: u32,
    pub date: i64,
}
```

Source: `src/public/db/tree_snapshot/`

The on-disk DB acts as a backing store when the in-memory `DashMap` evicts
entries.

---

## cache_db.redb — query prefetch

Caches the "locate" result (scroll position + data length) for filtered
queries.

Structure: one dynamic table per `VERSION_COUNT_TIMESTAMP` (string table
name), each mapping `u64` (query hash) → `Prefetch`.

```rust
pub struct Prefetch {
    pub timestamp: i64,
    pub locate_to: Option<usize>,
    pub data_length: usize,
}
```

Source: `src/public/db/query_snapshot/`

The query hash is computed from the filter expression parameters combined
with the current version timestamp.

---

## expire_db.redb — snapshot expiration

Tracks when in-memory snapshots should be expired.

Source: `src/public/db/expire/`

### Table: `"expire_table"`

Maps `i64` (snapshot timestamp) → `Option<i64>` (expiration timestamp, `None`
= never expires).

The expire-check loop (24h cycle) reads this table and removes expired
snapshot entries from both the in-memory `DashMap` and the on-disk
`temp_db.redb`.

---

## In-memory structures

These supplement the persistent store with fast read-optimized views:

| Structure                  | Type                                  | Content                                                  |
| -------------------------- | ------------------------------------- | -------------------------------------------------------- |
| `TREE.in_memory`           | `Arc<RwLock<Vec<DatabaseTimestamp>>>` | All `AbstractData` records, sorted by computed timestamp |
| `TREE_SNAPSHOT.in_memory`  | `DashMap<i64, Vec<ReducedData>>`      | Bucketed tree views per timestamp                        |
| `QUERY_SNAPSHOT.in_memory` | `DashMap<u64, Prefetch>`              | Query prefetch results                                   |

`DatabaseTimestamp` pairs an `AbstractData` with its computed sort timestamp:

```rust
pub struct DatabaseTimestamp {
    pub abstract_data: AbstractData,
    pub timestamp: i64,
}
```

All three are lazily initialized as module-level statics and rebuilt every
startup (or on demand after config changes).

---

## Per-record schema versioning

`AbstractData` records in `index_v5.redb` are prefixed with a 2-byte header
`[0xFF, version]` (implemented via redb's `Value` trait in
`src/public/constant/ser_de.rs`).

`0xFF` is safe as a magic marker because `AbstractData` is a 3-variant enum;
bitcode encodes the discriminant in the lowest 2 bits of the first byte
(values 0, 1, 2). A first byte of `0xFF` has bits `[1:0] = 11` = discriminant
3, which is invalid — so no legitimately encoded `AbstractData` can start with
`0xFF`.

Current schema version: **3** (`SCHEMA_VERSION` in `ser_de.rs`).

### Version history

| Version     | Change                                   | Notes                                                    |
| ----------- | ---------------------------------------- | -------------------------------------------------------- |
| 1 (legacy)  | Original schema                          | No version prefix on disk; detected by absence of `0xFF` |
| 2           | `albums: HashSet` → `album: Option`      | Each media item belongs to at most one album             |
| 3 (current) | `AlbumMetadata.dir_path: Option<String>` | Enables filesystem-hierarchy albums                      |

### On-read migration

When a record is read via the `Value::from_bytes` impl, the version byte
selects the correct decoder:

```rust
match version {
    1 => AbstractData::from(decode::<AbstractDataV1>(payload)),
    2 => AbstractData::from(decode::<AbstractDataV2>(payload)),
    3 => decode::<AbstractData>(payload),  // current, no transform needed
    v => panic!("Unknown schema version {v}"),
}
```

Records with no `0xFF` prefix (pre-versioning) fall through to the v1 decoder.

Each old-version type has a `From` impl that converts it to the current
`AbstractData`:

- **v2 → current**: `HashSet<ArrayString<64>> albums` collapses to
  `Option<ArrayString<64>> album` (takes the first element; empty set becomes
  `None`).
- **v1 → current**: Album records without `dir_path` get `dir_path: None`;
  Image/Video records pass through v2 first.

Old frozen types are preserved in `ser_de.rs` under `#[derive(bitcode::Decode)]`
(no `Encode` except under `#[cfg(test)]`) so they serve as read-only migration
targets.

---

## Database migration history

Urocissa has gone through five on-disk database formats. The current code only
supports opening `index_v5.redb` directly.

| Format       | File            | Storage engine | Schema                                    | Migration |
| ------------ | --------------- | -------------- | ----------------------------------------- | --------- |
| V2           | `index.redb`    | redb 2.6.x     | Flat `Database`/`Album` structs per table | Removed   |
| V3           | `index.redb`    | redb 3.x       | `AbstractData` enum without `update_at`   | Removed   |
| V4           | `index_v4.redb` | redb 3.x       | `AbstractData` with `update_at`           | Removed   |
| V5 (current) | `index_v5.redb` | redb 4.x       | `AbstractData` schema v3                  | Current   |

### V2 → V4 migration (deleted code, commit `7abf4452`)

The old `src/migration/` directory (deleted) contained:

- **`v2_v3.rs`**: Read the old redb 2.6.x database using the `redb_old` crate.
  Opened two tables (`"database"` with `OldDatabase` entries,
  `"album"` with `OldAlbum` entries). Transformed each record:
  - Extracted underscore-prefixed pseudo-tags (`_favorite`, `_archived`,
    `_trashed`) into dedicated boolean fields.
  - Moved `_user_defined_description` from `exif_vec` to the `description`
    field.
  - Converted `u128` timestamps to `i64`.
  - Migrated `HashSet<ArrayString<64>> album` to `HashSet<ArrayString<64>>`
    (same type).
- **`v3_v4.rs`**: Same redb version (3.x), same `AbstractData` enum shape, but
  the `ObjectSchema` lacked `update_at`. The migration stamped
  `Utc::now().timestamp_millis()` on every record.

Both paths wrote into a fresh `index_v4.redb` using `redb::Database::create()`,
processing in batches of 5000 with Rayon parallelism.

### V4 → V5 rename (commit `640c14c5`)

No data transformation — just `std::fs::rename("index_v4.redb",
"index_v5.redb")`. The file path was updated in `tree/new.rs` and the rename
logic was removed along with the rest of the migration code
(commit `7abf4452`).

### Current startup guard

`src/lib.rs:32-51` checks for the existence of `db/index_v4.redb`. If found,
startup is blocked with instructions to downgrade to v1.2.2 first, which is the
last release that carried the old migration pipeline.

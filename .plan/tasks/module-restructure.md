---
status: open
type: chore
priority: medium
area: backend
---

## Module restructure: src/ layout cleanup

Consolidate and rename `backend/src/` modules for clarity. The current layout has
vague names (`public/`, `operations/`, `process/`) and unnecessary directory
nesting (single-file dirs, 3-file sub-modules).

### Call graph (key dependencies)

```
lib.rs ──┬── init::initialize()
          │       ├── model::config::AppConfig::init()
          │       ├── init::initialize_folder()
          │       ├── init::check_ffmpeg_and_ffprobe()
          │       └── init::initialize_file() → storage::db::TREE
          │
          ├── process::dir_album::init_dir_album_cache()
          │       ├── storage::db::open_data_table()
          │       ├── storage::db::TREE
          │       └── job::coordinator::BATCH_COORDINATOR
          │
          ├── job::runtime::ROCKET_RUNTIME    (spawn Rocket)
          └── job::runtime::INDEX_RUNTIME     (spawn worker)

workflow::index_image() ──→ job::coordinator::INDEX_COORDINATOR
          ├── job::actor::OpenFileTask
          ├── job::actor::HashTask → process::hash::blake3_hasher
          ├── job::actor::DeduplicateTask → storage::db::open_data_table
           ├── job::actor::IndexTask → process::index::process_image/video_info (orchestrator inline)
│       ├── process::exif::generate_exif_for_image/video
│       ├── process::xmp::extract_keywords_from_file
│       ├── process::misc::generate_dynamic_image
│       ├── process::misc::generate_image_width_height
│       │       └── process::video_ffprobe::video_width_height  (video only)
│       ├── process::misc::fix_image_orientation
│       ├── process::misc::generate_thumbhash / generate_phash
│       └── process::thumbnail::
│               └── process::misc::create_silent_ffmpeg_command
│               └── process::misc::small_width_height
          └── job::actor::VideoTask → process::generate_compressed_video::
                  ├── process::video_ffprobe::video_duration
                  ├── process::index::create_silent_ffmpeg_command
                  └── process::index::process_image_info (reuses image pipeline)

storage/db ──┬── TREE (in_disk + in_memory Vec)
             │       ├── new.rs — opens index_v5.redb
             │       ├── read_tags.rs — TagInfo, read_tags(), read_albums()
             │       └── open_data_table()

storage/cache ──┬── TREE_SNAPSHOT (in_disk + in_memory DashMap)  [temp_db.redb]
                │       ├── new.rs — opens temp_db.redb
                │       ├── read_rows.rs → model::response::Row
                │       ├── read_scrollbar.rs → model::response::ScrollBarData
                │       ├── read_tags.rs → TagInfo
                │       └── read_tree_snapshot.rs → MyCow
                │
                ├── QUERY_SNAPSHOT (in_disk + in_memory DashMap)  [cache_db.redb]
                │       ├── new.rs — opens cache_db.redb
                │       └── read_query_snapshots.rs → Prefetch
                │
                └── EXPIRE (in_disk)  [expire_db.redb]
                        ├── new.rs — opens expire_db.redb
                        └── expired_check.rs

router/builder ──┬── router/auth::* guards
                 ├── router/get/mod  → generate_get_routes()
                 ├── router/post/mod → generate_post_routes()
                 ├── router/put/mod  → generate_put_routes()
                 ├── router/delete   → generate_delete_routes()
                 └── router/cache    → cache_control_fairing()
```

### File sizes that matter for merging decisions

```
>200 lines:  abstract_data 654, ser_de 618, generate_filter 546, config 487,
             get_prefetch 336, album_index 321, guard_hash 276, dir_album 250,
             album/combined 234, auth_utils 232, get_page 229, error 228,
             post_upload 223, get_fs_completion 207,
             generate_filter_hide_metadata 194

100-200:     start_watcher 179, edit_album 176, edit_share 157, edit_config 152,
             assign_album 151, guard_timestamp 148, get_img 144, get_data 142,
             xmp 140, workflow/index_image 138, builder 135,
             rotate_image 124, edit_flags 122, process/extract 122,
             create_share 114, delete_data 109, reindex 108,
             regenerate_thumbnail 106, deduplicate 103,
             generate_compressed_video 103, edit_description 102,
             post/album_index 100

 50-100:     index_task 99, edit_tag 98, storage 98, thumbnail 93,
             transitor 93, exif 92, bootstrap 92,
             flush_query_snapshot 92, album_task 88, get_list 88,
             tree/read_tags 88, flush_tree_snapshot 87,
             post/create_dir_album 87, update_tree 85, expire/expired_check 80,
             expire_check 79, read_tree_snapshot 76, get_export 75, get_config 75,
             object 71, flush_tree 70, process/transitor 69 (bar)

 30-50:      guards (6 files, 24-48 lines each), fix_orientation 52,
             generate_dynamic_image 43, generate_width_height 23,
             generate_image_hash 15, generate_ffmpeg 9, hash 30, resize 35,
             image_path 15, init files (4-30 lines each)

<30:        various mod.rs, e2e.rs, embedded.rs, redb.rs, etc.
```

### Target hierarchy

```
backend/src/
├── bin/openapi.rs                          ← keep
├── main.rs                                 ← keep
├── lib.rs                                  ← keep
├── openapi.rs                              ← keep (auto-generated)
├── error.rs                                ← AppError, ErrorKind
│                                               (was public/error.rs + public/error_data.rs)
├── constant.rs                             ← ROW_BATCH_NUMBER, file extensions, etc.
│                                               (was public/constant/mod.rs re-exports)
├── frontend.rs                             ← FrontendAssets struct
│                                               (was public/embedded.rs)
├── init.rs                                 ← boot sequence: config init, folders,
│                                               ffmpeg check, logger, DB cache clear
│                                               (was operations/initialization/* +
│                                                process/initialization.rs)
├── model/                                  ← data model types*
│   ├── mod.rs
│   ├── abstract_data.rs                    ← was structure/abstract_data.rs (654 lines)
│   ├── album.rs                            ← was structure/album/* (4 files merged, ~270 lines)
│   ├── config.rs                           ← AppConfig (was structure/config.rs, 487 lines)
│   ├── expression.rs                       ← was structure/expression/* (3 files, ~740 total)
│   ├── image.rs                            ← was structure/image/* (3 files, ~50 lines)
│   ├── media.rs                            ← was public/media.rs (13 lines)
│   ├── object.rs                           ← was structure/object.rs (71 lines)
│   ├── response.rs                         ← was structure/response/* (4 files) +
│                                               structure/common/file_modify.rs (~80 lines)
│   └── video.rs                            ← was structure/video/* (3 files, ~55 lines)
├── storage/                                ← how data is *persisted*
│   ├── mod.rs
│   ├── db.rs                               ← permanent: TREE (index_v5.redb) + open_data_table
│   │                                          (was tree/ + open_db/ + redb.rs ~155 lines)
│   ├── cache.rs                            ← transient: TREE_SNAPSHOT (temp_db),
│   │                                          QUERY_SNAPSHOT (cache_db), EXPIRE (expire_db)
│   │                                          cleared on every startup
│   │                                          (was tree_snapshot/ + query_snapshot/ + expire/
│   │                                          ~375 lines total)
│   ├── files.rs                            ← path resolution (was storage.rs +
│   │                                          utils/image_path.rs, 113 lines)
│   └── ser_de.rs                           ← versioned codec (was ser_de.rs, 618 lines)
├── process/                                ← what the system *does*
│   ├── mod.rs
│   ├── dir_album.rs                        ← 250 lines
│   ├── hash.rs                             ← 30 lines (flattened)
│   ├── index.rs                            ← entry point + orchestrator:
│   │                                          workflow::index_image (138) + info.rs (122)
│   │                                          → ~260 lines total
│   ├── xmp.rs                              ← XMP keyword extraction (was extract_keywords, 140 lines)
│   ├── exif.rs                             ← EXIF metadata extraction (was generate_exif, 92 lines)
│   ├── generate_compressed_video.rs        ← 103 lines
│   ├── thumbnail.rs                        ← JPEG thumbnail generation (was generate_thumbnail, 93 lines)
│   ├── video_ffprobe.rs                    ← 62 lines
│   ├── transitor.rs                        ← transitor files merged (~110 lines)
│   └── misc.rs                             ← small helpers merged:
│                                              generate_ffmpeg (9), generate_image_hash (15),
│                                              generate_width_height (23), fix_orientation (52),
│                                              generate_dynamic_image (43), resize (35)
│                                              → ~175 lines total
├── router/                                 ← HTTP layer
│   ├── mod.rs                              ← AppResult, GuardResult, type aliases
│   ├── builder.rs                          ← Rocket assembly (135 lines)
│   ├── auth.rs                             ← claims/ + fairing/ merged (~680 lines:
│   │                                          4 claim types + 7 guards + auth utils)
│   ├── cache.rs                            ← cache_control_fairing (18 lines)
│   ├── delete.rs                           ← flattened (109 lines)
│   ├── get/                                ← 9 handlers (keep directory)
│   ├── post/                               ← 6 handlers (keep directory)
│   └── put/                                ← 11 handlers (keep directory)
├── job/                                    ← background tasks
│   ├── mod.rs                              ← INDEX_COORDINATOR, BATCH_COORDINATOR
│   ├── runtime.rs                          ← tokio runtimes + rayon pool
│   ├── actor.rs                            ← 8 task types merged (~750 lines)
│   ├── batcher.rs                          ← 7 batch tasks merged (~580 lines)
│   └── looper.rs                           ← periodic loop
└── tests/                                  ← unchanged
```

### Dependency analysis — who imports what

| Module | Imports from | Notes |
|---|---|---|
| `model/abstract_data` | `model::object` (ObjectType), `process::hash` (generate_random_hash) | circular: abstract_data → hash → ... |
| `model/config` | `storage::storage`, `job::batcher::reload_watcher` | config depends on store + job |
| `model/expression` | `process::dir_album` | generate_filter calls get_dir_path_for_album |
| `storage/db` | `model::*` (AbstractData, DatabaseTimestamp, etc.) | permanent DB: TREE, open_data_table |
| `storage/cache` | `model::*` (ReducedData, Row, Prefetch), `storage::db` (VERSION_COUNT_TIMESTAMP) | transient DBs cleared on restart |
| `storage/files` | nothing in crate | pure filesystem path logic |
| `storage/ser_de` | `model::*`, `router::get::prefetch::Prefetch` | knows all data structs + Prefetch |
| `process/index` | `process::dir_album`, `process::*` (xmp, exif, thumbnail), `storage::files`, `job::coordinator`, `job::actor::*` | entry point + orchestrator, calls all indexation |
| `job/actor` | `process::*`, `storage::db`, `job::coordinator` | dispatches domain work |
| `job/batcher` | `process::dir_album`, `storage::db`, `model::*`, `job::coordinator` | coordinates batch updates |
| `router/auth` | `model::*`, `router::*` | claims and guards |

Circular dependency to resolve:
- `model::abstract_data::generate_random_data()` calls `process::hash::generate_random_hash()`
  → model should not depend on process (domain logic calls model, not the other way)
  → Move `generate_random_hash` into `model/object.rs`

### Key design decisions

1. **`model/` depends on `process/` for `generate_random_hash`** — this is a layering
   violation. `generate_random_hash` (just `ArrayString::from(rng.gen::<u64>().to_string())`)
   should live in `model/object.rs` or `model/abstract_data.rs` since it's a data-aware
   utility, not domain logic. This removes the circular-ish dependency.

2. **`model/expression/` depends on `process/dir_album`** — Expression::generate_filter
   calls `get_dir_path_for_album` and `get_parent_album_id`. This is filter evaluation
   that needs the dir-album cache. The dir-album cache would stay in `process/` since it's
   domain logic (maintained by the indexer). The expression module's dependency is
   legitimate — filter evaluation at query time needs access to the cache.

3. **`storage/ser_de` depends on `router::get::prefetch::Prefetch`** — Prefetch is a
   serializable response type used in ser_de. Prefetch should be in `model/response.rs`
   instead of in the router layer.

4. **`model/config` depends on `job::batcher::reload_watcher`** — when config is saved
   via PUT, it reloads the filesystem watcher. This is a callback from config into job.
   This is legitimate — config update triggers a side effect in the job layer.

### Library re-exports (lib.rs)

Currently:
```rust
pub use storage::files::DATA_PATH;        // was public::constant::storage
pub use model::config::{APP_CONFIG, AppConfig};  // was public::structure::config
pub use router::builder::build_rocket_with_config;
```

These will stay the same, just with updated paths.

OpenAPI binary imports every `__path_*` symbol from route handlers. These will need
updated paths in the auto-generated `openapi.rs` (regenerated after restructure).

### Progress notes

- 2026-06-26: Initial mapping finalized. Added call graph, file sizes, dependency analysis.

# Local backlog (gitignored)

- standalone TLS deployment path: add `tls` Cargo feature gating `rocket/tls`, validate end-to-end with a real cert (certbot/acme.sh), document in CONFIG.md; only ship once tested
- review for license issues, SPDX label and OSSF score card
- review configuration options against docs
- add a proper documentation / generation system
- move the fork to github, compatible CI flows and Android app by Ted(?)
- enable the release/deployment flows in CI
- add android app 
- review code module by module

- 'just build' uses embed-frontend feature, raising different warnings from the default make check / make test
  - the default build/test should all use the same build configurations
  - CI should be expanded to test all supported configurations on PR and release

- tags assigned via api should be merged into DB as well as image repository on disk (idea borrowed from Immich)
  1. use xmp sidecar files to store all extracted data without modifying originals
  2. on indexing, if no xmp sidecar exists, create from image metadata
  3. on tag or other metadata changes, first store to db and then to xmp sidecar
  4. on album assignment (causing the original file to be moved), xmp sidecar files are moved together with the originals 
  5. we can provide a separate helper script to merge metadata from sidecar files back into originals. major photo apps probably also support such files.

- wrap image repo file operations and DB updates into a single transaction
  - the idea is to extend the DB consistency assurance to the photo repo
  - similar to journaling filesystems, a marker (mutex? file log) should record the planned transaction, perform the DB update, then file update, then release the transaction lock
  - if we crash before modifying files, we can detect the unfinished transaction and either rollback or complete, thus ensuring consistency
  - maybe there are better patterns and tooling to do this efficiently. but it should work on any (local) filesystem

## Github fork

- [ ] work on github fork for more compatible CI + contributing back
- [x] fork main
- [ ] push all changes to a dev branch
- [ ] refactor changes to first apply devops, tests, audit fixes, then merge / compress the album stuff?

## Docker / Deployment (assessed 2026-06-15)

The current setup has two pieces: `docker/Dockerfile` for building and `run_urocissa_docker.sh` for running the upstream-published image (`hsa00000/urocissa:latest`).

### Issues with current approach

- **Dockerfile in `docker/`** — non-standard; most tooling (GitHub Actions `docker/build-push-action`, Docker Hub auto-builds, `docker build .`) expects it at repo root. Requires `-f docker/Dockerfile` everywhere.
- **Run shell script (~230 lines)** — does what Docker Compose handles in ~20 lines: volume mounts, port mapping, env vars. Raw `sed`/`grep` JSON parsing of `syncPaths` and port is fragile.
- **Entrypoint `mv` pattern** — moves binaries from image into `UROCISSA_PATH` at runtime; non-standard and fragile. Normal pattern: mount host dirs into fixed image paths.

### Recommended improvements

- [ ] Add `docker-compose.yml` at repo root — replaces the shell script; `docker compose up -d` is the standard UX users expect. Highest value single improvement.
- [ ] Move `docker/Dockerfile` to repo root; keep `docker/` for compose, `.dockerignore`, and CI helpers only.
- [ ] Add a systemd `.service` file in `deploy/` or `contrib/` for users running the binary directly (non-Docker).
- [ ] Publish to `ghcr.io/codesam/urocissa` (GitHub Container Registry) — free for public images, co-located with source; complement or replace Docker Hub `hsa00000/urocissa`.

---

## Code conventions and tooling notes

### Pre-commit hook order
`just precommit` runs in this order: `cargo fmt --check` → `cargo clippy` → `cargo nextest` → `prettier --check` → `vue-tsc + eslint` → `vitest`. Run `cargo fmt` and `npx prettier --write` before staging to avoid the most common late failures. ESLint and vue-tsc run after prettier, so formatting errors mask type/lint errors until fixed.

### Error code policy (backend)
`ErrorKind::NotFound` maps to HTTP 404, which Rocket also returns for unregistered routes. Using `NotFound` for domain-level "entity not found" makes those errors indistinguishable from routing failures in tests and clients. Use `ErrorKind::InvalidInput` (→ 400) when the route succeeded but a referenced entity does not exist. Only use `NotFound` when a routing-level 404 is genuinely the right signal.

### ESLint strict mode (frontend)
`@typescript-eslint/strict-boolean-expressions` and `@typescript-eslint/no-non-null-assertion` are enforced. These only surface at commit time (pre-commit hook). When writing TypeScript: use `=== null` / `=== undefined` instead of truthiness checks on nullable strings; avoid `!` non-null assertions — use an explicit null/undefined guard branch instead.

---

## Album feature — open items

- [ ] **E2E test for `create_dir_album`** — endpoint has no scenario coverage. Should verify: subdir is created on disk, returns new album ID, parent album updates.
- [ ] **Stale `DIR_ALBUM_CACHE` scenario** — if a directory is deleted externally while the cache still holds its entry, `assign_album` will attempt to move a file into a non-existent path. Decide: detect and evict stale entries at startup, or return a clear 400 with a meaningful message at request time.
- [ ] **`assign_album` file-move verification in tests** — Scenario H checks album membership after assignment but does not verify the file moved to the correct directory on disk. Add a filesystem assertion to close this gap.

---

## Testing — best value items

### Backend unit tests (low effort, no DB needed)
- [x] `prettify_dir_name` — pure string transform, dir_album.rs
- [x] schema version round-trips — encode v1/v2, decode, check fields; ser_de.rs
- [x] `Expression` filter predicates — construct filter, run against fixture AbstractData; generate_filter.rs
- [x] `compute_timestamp` priority logic — abstract_data.rs
- [x] `belongs_to_album` path-prefix logic — combined.rs (dir vs manual branching)

### Backend integration tests (need tempdir redb — not as hard as it sounds)
- [ ] index → dedup → flush → album self-update round-trip
- [ ] dir album path-prefix membership (create album, check file in/out of subtree)
- [x] schema migration: write v1-encoded record, read back as v2 (through redb)

### Backend tooling (trivial)
- [x] deny(unsafe_code) — enforced at compile time via main.rs
- [x] cargo nextest — installed, replacing cargo test in justfile
- [x] cargo audit — in justfile (`just backend-audit`); known-unfixable advisories suppressed in `.cargo/audit.toml` with enforcement tests
- [x] cargo deny — deny.toml in place; wired into just audit
- [x] tighten clippy: unwrap_used = warn in Cargo.toml; excluded from -D warnings until call sites are cleaned up
- [ ] cargo geiger — include in release/audit reports (unsafe surface area of dep tree)
- [ ] semi-regular security audit CI job — run `just audit` on a schedule (monthly or on dep changes); options: Codeberg CI (Woodpecker), GitHub Actions on the upstream fork, or a local cron on the dev machine; the `/security-audit` skill covers the manual review workflow when issues are found

### Frontend unit tests
- [x] Vitest — installed; 34 lexer tests covering all atom types, compound operators, escaping, and lex errors
- [ ] Pinia store reducers — pure logic in stores, no DOM needed

### Frontend tooling
- [ ] knip — periodic dead-export sweep, not in precommit
- [x] npm audit — in justfile (`just frontend-audit`); all vulnerabilities resolved by bumping to latest deps
- [ ] dependency updates — establish a regular cadence (npm-check-updates, Dependabot, or Renovate); small frequent updates are cheaper than batched drift
- [ ] Zod (or valibot) — investigate for runtime validation of API responses; TypeScript types don't catch backend schema drift at runtime

### Known bugs
- [ ] **Grey placeholders on initial load** — `useElementSize` reports width 0 on first render; `usePrefetch` skips the fetch (guard: `windowWidth > 0`); the Home.vue watch then fires a row fetch before `prefetch()` sets the correct timestamp, so rows arrive stale and are discarded. A zoom or resize re-triggers the cycle cleanly and images appear. Root: `useElementSize` ResizeObserver fires after the first render tick, too late for the initial fetch path. Fix: ensure `prefetchStore.windowWidth` is set before the first `fetchRowInWorker` call, or delay the Home.vue watch fetch until after `processPrefetchChain` completes.
- [ ] **Info sidepane always shows "No album" until manually (re-)assigned** — the explicit per-photo `album` field (`img.metadata.album`, read by `ItemAlbum.vue` and `AssignAlbumModal`'s "current album" badge) is only ever set by `PUT /put/assign_album`. Normal indexing — including the filesystem-watcher path (`workflow::index_for_watch` → `process_image_info`) — never sets it for directory-hierarchy albums, even though `generate_filter.rs` correctly counts the file via path-based membership. Red test: `gallery-backend/src/tests/e2e.rs::scenario_l_dir_album_membership_not_set_at_index_time`. Fix: have indexing resolve and set the explicit field too (or make the sidepane/modal resolve "current album" via the same path-based rule `generate_filter.rs` uses, instead of reading the raw field).
- [ ] **Alias entry duplicates on every `assign_album` reassignment** — `assign_album` renames the file on disk, which the filesystem watcher observes as a `Create` event and re-indexes via `index_for_watch(path, None)`. Since the hash already exists, `DeduplicateTask` (`deduplicate.rs`) pushes another alias entry for the *same* path instead of recognising it already matches the current alias, so `alias` grows by one entry per reassignment. Red test: `scenario_m_watcher_reindex_after_assign_duplicates_alias`. Fix: skip the push in `deduplicate_task` when the incoming path already equals an existing alias entry.
- [ ] **Tags are never discovered at index time** — unlike the `album` field, this is a missing feature rather than a regression: nothing in the indexing pipeline extracts keyword metadata (IPTC/XMP `dc:subject`) embedded by tools like Lightroom/digiKam; tags are only ever set via `PUT /put/edit_tag`. `extract_keywords_from_xmp` (`operations/indexation/extract_keywords.rs`) is wired into both `process_image_info` and `process_video_info` but is a stub that always returns an empty set — implement XMP packet scanning there. Red tests: `extract_keywords::tests::*` (pure parser contract) and `scenario_n_tags_not_discovered_from_xmp_keywords_at_index_time` (full image pipeline; no equivalent video-pipeline test yet, see format-coverage item below). The parts of the tag feature that already work are locked in by green regression tests `scenario_o_tags_visible_via_get_data_sidebar_path` and `scenario_p_tags_modifiable_via_edit_tag_api`; `scenario_q_album_visible_via_get_data_after_assign` and `scenario_s_reindex_preserves_album_and_tags` lock in the equivalent already-working parts of the album feature (API-set field survives both a sidebar read and a full reindex) for contrast.
- [ ] **Stale alias entries accumulate when a tracked file is moved outside the app** — generalises the bug above: `DeduplicateTask` (`deduplicate.rs`) only ever *pushes* a new alias entry when it rediscovers a known hash at a new path; it never prunes entries whose `file` no longer exists on disk. Moving a tracked file with a file manager (not via `assign_album`) leaves the old, now-dead path in `alias` forever, and repeated moves grow the list unboundedly. Red test: `scenario_r_externally_moved_file_keeps_dead_alias_entry`. Fix: when discovering a hash with a new path, drop any existing alias entries whose `file` no longer exists.
- [ ] **`prefetch` snapshots can be deleted almost immediately instead of living for 1 hour** — found while stabilising the new e2e tests (was causing real, intermittent 500s on `/get/get-data` right after `/get/prefetch`, not just test flakiness). `Expire::expired_check` (`public/db/expire/expired_check.rs:77`, `None => true`) cannot distinguish "this timestamp was never recorded" from "this is the *current* `VERSION_COUNT_TIMESTAMP`, whose expiry hasn't been scheduled yet" (`update_expire_task` inserts `expire_table[current_timestamp] = None` deliberately, as a placeholder). Both collapse to the same `None` arm. Since `flush_query_snapshot_task` names each on-disk `QUERY_SNAPSHOT` table after the *current* `VERSION_COUNT_TIMESTAMP` at flush time, the very next version bump anywhere in the process (any concurrent indexing/edit/reindex) makes `expire_check_task` see `VERSION_COUNT_TIMESTAMP > that_table's_timestamp` and call `expired_check` on it — which returns `true` immediately via the `None` arm instead of waiting ~1 hour, deleting the query snapshot table and cascading (via the `RemoveTask` it schedules for every `Prefetch.timestamp` row inside) to delete the just-created `TREE_SNAPSHOT` table too. Net effect: under any concurrent write activity, a freshly-`prefetch`'d snapshot can vanish within milliseconds, and the next `/get/get-data`/`/get/get-rows` call against it 500s with "Failed to open tree snapshot table". Worked around in tests via `read_current_abstract_data` (re-prefetch-and-retry instead of reusing a timestamp); not worked around in the frontend. Fix: distinguish "unrecorded" from "active, not yet scheduled" — e.g. store expiry as a tri-state, or only ever insert the *current* version's row once its *own* successor exists, or check `entry.is_none()` separately from "row absent" before defaulting to "expired".
- [ ] **Enable metadata-extraction tests for all supported file formats** — `extract_keywords_from_xmp`'s current (stub) substring-scan approach is format-agnostic by construction, but only `scenario_n` (JPEG, hand-spliced APP1/XMP segment) exercises it end-to-end. Supported extensions today (`public/constant/mod.rs`): images `jpg, jpeg, jfif, jpe, png, tif, tiff, webp, bmp`; videos `gif, mp4, webm, mkv, mov, avi, flv, wmv, mpeg`. XMP/IPTC packet location and encoding differ by container — PNG can store text in compressed `zTXt` chunks (the naive scan would miss it), MP4/MOV embed XMP in a `uuid` box, TIFF has no `APPn` segment concept, and IPTC IIM (the older, non-XMP keyword mechanism some tools still write) is a separate binary format entirely. Once real extraction is implemented, extend coverage with one `scenario_n`-style test per representative container (at least: PNG with an uncompressed `iTXt` XMP packet, PNG with a compressed `zTXt` one, MP4 with a `uuid` XMP box, and a plain IPTC-IIM-only JPEG) rather than assuming the JPEG case generalises. Video-pipeline coverage additionally needs `ffmpeg`/`ffprobe` available in the test environment (process_video_info shells out to both), which `scenario_n`'s image-only path avoids.

### Deferred
- [ ] Playwright E2E — needs running backend; save for CI
- [ ] cargo bench / criterion — indexing pipeline regression benchmarks (file hashing, EXIF extraction, thumbnail generation, redb write throughput, serving latency under load)
- [ ] Admin status API — expose queue depth, active indexing jobs, recent errors, per-job progress as a JSON endpoint (foundation for a future admin panel)
- [ ] Frontend progress UI — poll the existing `/get/import/folder/status` pattern (or a future SSE stream) to show active indexing progress to authenticated users
- [ ] Structured logging — investigate `tracing` + `tracing-subscriber` as a replacement for `env_logger` to support spans, JSON output for log aggregators, and `tracing-journald` for native systemd/journald integration

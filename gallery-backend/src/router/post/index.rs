use rocket::http::Status;
use rocket::post;

use crate::router::fairing::guard_auth::GuardAuth;
use crate::router::fairing::guard_read_only_mode::GuardReadOnlyMode;
use crate::router::{AppResult, GuardResult};
use crate::tasks::actor::folder_import::start_image_home_index;

/// Start scanning `IMAGE_HOME` (or a subdirectory of it) for media files to
/// index.  The scanned root is always under the configured `imagePath`:
///
/// * `path` (optional) — a subdirectory relative to `imagePath` to scan
///   instead of the whole root.  Must exist before calling this endpoint.
/// * `force` (default `false`) — if `true`, re-run full metadata extraction
///   for files whose content hash is already indexed.
///
/// Status can be polled via `GET /get/import/folder/status`.
#[post("/post/index?<path>&<force>")]
#[allow(clippy::needless_pass_by_value)]
pub fn start_index_handler(
    _auth: GuardAuth,
    read_only: GuardResult<GuardReadOnlyMode>,
    path: Option<String>,
    force: Option<bool>,
) -> AppResult<Status> {
    let _ = read_only?;
    start_image_home_index(path.as_deref(), force.unwrap_or(false))?;
    Ok(Status::Accepted)
}

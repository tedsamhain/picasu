use rocket::http::Status;
use rocket::post;
use rocket::serde::json::Json;
use serde::Deserialize;

use crate::router::fairing::guard_auth::GuardAuth;
use crate::router::fairing::guard_read_only_mode::GuardReadOnlyMode;
use crate::router::{AppResult, GuardResult};
use crate::tasks::actor::folder_import::{
    cancel_folder_import, start_folder_import, start_image_home_scan,
};

#[derive(Deserialize)]
pub struct StartFolderImportRequest {
    path: String,
}

#[post("/post/import/folder", data = "<req>")]
pub fn start_folder_import_handler(
    _auth: GuardAuth,
    read_only: GuardResult<GuardReadOnlyMode>,
    req: Json<StartFolderImportRequest>,
) -> AppResult<Status> {
    let _ = read_only?;
    start_folder_import(&req.into_inner().path)?;
    Ok(Status::Accepted)
}

/// Scan the configured `imagePath` for files the watcher hasn't indexed yet
/// (e.g. pre-existing files dropped in before the app last started). Unlike
/// `start_folder_import_handler`, takes no path — always targets the
/// configured root, so albums/hierarchy are reliably discovered. Shares the
/// same job slot/status as a regular folder import.
#[post("/post/import/image-home")]
pub fn start_image_home_scan_handler(
    _auth: GuardAuth,
    read_only: GuardResult<GuardReadOnlyMode>,
) -> AppResult<Status> {
    let _ = read_only?;
    start_image_home_scan()?;
    Ok(Status::Accepted)
}

#[post("/post/import/folder/cancel")]
pub fn cancel_folder_import_handler(_auth: GuardAuth) -> AppResult<Status> {
    cancel_folder_import()?;
    Ok(Status::Ok)
}

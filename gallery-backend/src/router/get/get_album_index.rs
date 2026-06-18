use rocket::get;
use rocket::serde::json::Json;

use crate::router::fairing::guard_auth::GuardAuth;
use crate::tasks::actor::album_index::{AlbumIndexStatus, album_index_status};

#[get("/get/index/status")]
pub fn get_album_index_status(_auth: GuardAuth) -> Json<AlbumIndexStatus> {
    Json(album_index_status())
}

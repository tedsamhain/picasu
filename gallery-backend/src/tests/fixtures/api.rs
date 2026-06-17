use std::path::Path;

use image::{Rgb, RgbImage};
use rocket::http::{ContentType, Cookie, Status};
use rocket::local::blocking::Client;
use serde_json::Value;

use arrayvec::ArrayString;

use crate::operations::dir_album::get_or_create_dir_album;
use crate::public::structure::config::AppConfig;
use crate::router::builder::build_rocket_with_config;
use crate::tasks::BATCH_COORDINATOR;
use crate::tasks::batcher::flush_tree::FlushTreeTask;

use super::TEST_ENV;

// ─── HTTP client helpers ─────────────────────────────────────────────────

pub fn make_client() -> Client {
    let _ = &*TEST_ENV;
    Client::tracked(build_rocket_with_config(AppConfig::default())).expect("valid rocket instance")
}

pub fn auth_cookie(client: &Client) -> Cookie<'static> {
    let r = client
        .post("/post/authenticate")
        .header(ContentType::JSON)
        .body(r#""""#)
        .dispatch();
    let token = r.into_string().expect("token body");
    Cookie::new("jwt", token.trim_matches('"').to_owned())
}

pub fn json_get(client: &Client, path: &str) -> Value {
    let cookie = auth_cookie(client);
    let r = client.get(path).cookie(cookie).dispatch();
    assert_eq!(r.status(), Status::Ok, "GET {path} failed");
    serde_json::from_str(&r.into_string().unwrap()).expect("valid JSON")
}

/// Run the same prefetch flow the frontend uses before fetching row data:
/// POST /get/prefetch?locate=<hash>, returning (timestamp, index, bearer
/// token) for the matching item so the caller can hit /get/get-data or
/// /put/edit_tag exactly like a real client would.
pub fn prefetch_locate(client: &Client, hash: &str) -> (i64, usize, String) {
    let cookie = auth_cookie(client);
    let resp = client
        .post(format!("/get/prefetch?locate={hash}"))
        .cookie(cookie)
        .header(ContentType::JSON)
        .dispatch();
    assert_eq!(resp.status(), Status::Ok, "prefetch must succeed");
    let body: Value = serde_json::from_str(&resp.into_string().unwrap()).expect("valid JSON");
    let timestamp = body["prefetch"]["timestamp"]
        .as_i64()
        .expect("prefetch.timestamp");
    let index = usize::try_from(
        body["prefetch"]["locateTo"]
            .as_u64()
            .expect("prefetch.locateTo must be present for a known hash"),
    )
    .expect("index fits in usize");
    let token = body["token"].as_str().expect("token").to_owned();
    (timestamp, index, token)
}

/// `prefetch_locate` + `get_data_item`, with a fresh re-prefetch on
/// failure rather than blindly re-requesting the same (now possibly
/// invalid) timestamp.
///
/// A `prefetch` snapshot is meant to live for 1 hour
/// (`update_expire_task`), but `Expire::expired_check`
/// (`public/db/expire/expired_check.rs`) treats "no expiry entry
/// recorded yet" (`None`) the same as "already expired" — and a
/// brand-new snapshot's *own* `VERSION_COUNT_TIMESTAMP` slot is
/// recorded as exactly that `None` until the *next* version bump. So
/// as soon as anything else in the process bumps the global,
/// process-wide `VERSION_COUNT_TIMESTAMP` (any indexing or edit
/// anywhere — including other tests running concurrently),
/// `expire_check_task` immediately deletes the just-created query/tree
/// snapshot. This is a real bug (see TODO.md), not a test artifact —
/// but it's not what these particular tests are meant to catch, so we
/// route around it here by re-prefetching instead of trying to outwait
/// it.
pub fn read_current_abstract_data(client: &Client, hash: &str) -> Value {
    for attempt in 0..5 {
        let (timestamp, index, token) = prefetch_locate(client, hash);
        let resp = client
            .get(format!(
                "/get/get-data?timestamp={timestamp}&start={index}&end={}",
                index + 1
            ))
            .header(rocket::http::Header::new(
                "Authorization",
                format!("Bearer {token}"),
            ))
            .dispatch();
        if resp.status() == Status::Ok {
            let body: Value =
                serde_json::from_str(&resp.into_string().unwrap()).expect("valid JSON");
            return body.as_array().expect("array")[0]["abstractData"].clone();
        }
        if attempt == 4 {
            assert_eq!(resp.status(), Status::Ok, "get-data must succeed");
        }
    }
    unreachable!()
}

/// Fetch the original-file bytes via the real client flow: prefetch +
/// get-data (to obtain the hash-scoped, `allow_original` token embedded
/// in the response) + `GET /object/imported/<hash-prefix>/<hash>.<ext>`.
pub fn fetch_original_bytes(client: &Client, hash: &str, ext: &str) -> Vec<u8> {
    let (timestamp, index, ts_token) = prefetch_locate(client, hash);
    let resp = client
        .get(format!(
            "/get/get-data?timestamp={timestamp}&start={index}&end={}",
            index + 1
        ))
        .header(rocket::http::Header::new(
            "Authorization",
            format!("Bearer {ts_token}"),
        ))
        .dispatch();
    assert_eq!(resp.status(), Status::Ok, "get-data must succeed");
    let body: Value = serde_json::from_str(&resp.into_string().unwrap()).expect("valid JSON");
    let hash_token = body.as_array().expect("array")[0]["token"]
        .as_str()
        .expect("token")
        .to_owned();

    let resp = client
        .get(format!("/object/imported/{}/{hash}.{ext}", &hash[0..2]))
        .header(rocket::http::Header::new(
            "Authorization",
            format!("Bearer {hash_token}"),
        ))
        .dispatch();
    assert_eq!(
        resp.status(),
        Status::Ok,
        "GET /object/imported must succeed"
    );
    resp.into_bytes().expect("response body bytes")
}

// ─── Filesystem / album helpers ──────────────────────────────────────────

/// Create a dir album for `dir_path` and return its album ID.
pub fn make_dir_album(dir_path: &Path) -> ArrayString<64> {
    get_or_create_dir_album(dir_path.to_path_buf()).expect("create dir album")
}

/// Write a tiny real (decodable) JPEG to `path` — needed for tests that
/// exercise the real indexing pipeline (`process_image_info` decodes
/// pixels), unlike the `b"\xff\xd8\xff fake jpeg"` fixtures used by
/// tests that only exercise `assign_album` (no decode involved).
pub fn write_real_jpeg(path: &Path, color: [u8; 3]) {
    let img = RgbImage::from_pixel(4, 4, Rgb(color));
    img.save(path).expect("encode real jpeg");
}

/// Write a real decodable JPEG with an embedded XMP packet declaring
/// `dc:subject` keywords, as Lightroom/digiKam/exiftool would write
/// them. The packet is spliced into an APP1 segment right after the
/// SOI marker; JPEG decoders skip unrecognised APPn segments, so the
/// image still decodes normally.
pub fn write_real_jpeg_with_xmp_keywords(path: &Path, color: [u8; 3], keywords: &[&str]) {
    let img = RgbImage::from_pixel(4, 4, Rgb(color));
    let mut jpeg_bytes: Vec<u8> = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut jpeg_bytes),
        image::ImageFormat::Jpeg,
    )
    .expect("encode jpeg to memory");

    let items: String = keywords
        .iter()
        .map(|k| format!("<rdf:li>{k}</rdf:li>"))
        .collect();
    let xmp_packet = format!(
        r#"<x:xmpmeta xmlns:x="adobe:ns:meta/"><rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"><rdf:Description xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:subject><rdf:Bag>{items}</rdf:Bag></dc:subject></rdf:Description></rdf:RDF></x:xmpmeta>"#
    );

    let mut segment: Vec<u8> = b"http://ns.adobe.com/xap/1.0/\0".to_vec();
    segment.extend_from_slice(xmp_packet.as_bytes());
    let segment_len = u16::try_from(segment.len() + 2).expect("xmp segment too large");
    let mut app1: Vec<u8> = vec![0xFF, 0xE1];
    app1.extend_from_slice(&segment_len.to_be_bytes());
    app1.extend_from_slice(&segment);

    // jpeg_bytes[0..2] is the SOI marker (FF D8); splice APP1 right after it.
    let mut spliced = jpeg_bytes[..2].to_vec();
    spliced.extend_from_slice(&app1);
    spliced.extend_from_slice(&jpeg_bytes[2..]);

    std::fs::write(path, &spliced).expect("write spliced jpeg");
}

/// Block until every `FlushTreeTask` queued before this call has been
/// written to disk. `index_task`/`video_task` enqueue their writes via
/// `execute_batch_detached`, which returns immediately; submitting an
/// empty flush with `execute_batch_waiting` guarantees (per
/// `mini_executor`'s ordering contract) that everything submitted
/// earlier on the same queue has already run by the time this resolves.
pub async fn wait_for_flush() {
    BATCH_COORDINATOR
        .execute_batch_waiting(FlushTreeTask::insert(Vec::new()))
        .await
        .expect("flush sync");
}

// ─── Upload helpers ──────────────────────────────────────────────────────

/// Build a `multipart/form-data` body for `POST /upload`: one `file`
/// part (binary, with the given filename/content-type) and one
/// `lastModified` text part, matching what the frontend sends.
pub fn build_upload_multipart_body(
    filename: &str,
    file_bytes: &[u8],
    content_type: &str,
    last_modified_ms: u64,
) -> (String, Vec<u8>) {
    let boundary = "e2eUploadBoundary".to_string();
    let mut body = Vec::new();
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\nContent-Type: {content_type}\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(file_bytes);
    body.extend_from_slice(b"\r\n");
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"lastModified\"\r\n\r\n{last_modified_ms}\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
    (boundary, body)
}

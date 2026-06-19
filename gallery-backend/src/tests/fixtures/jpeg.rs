use std::path::Path;

use xtask::test_image::{PhotoSpec, generate_photo_file};

pub fn path_color(path: &str) -> [u8; 3] {
    let h = path.bytes().fold(0u32, |mut h, b| {
        h = h.wrapping_add(b as u32);
        h = h.wrapping_add(h << 10);
        h ^= h >> 6;
        h
    });
    let h = {
        let mut h = h;
        h = h.wrapping_add(h << 3);
        h ^= h >> 11;
        h = h.wrapping_add(h << 15);
        h
    };
    [(h >> 16) as u8, (h >> 8) as u8, h as u8]
}

fn photo_spec(color: [u8; 3], tags: Option<Vec<String>>, exif_date: Option<String>) -> PhotoSpec {
    PhotoSpec {
        output: None,
        format: Some("jpeg".into()),
        width: Some(4),
        height: Some(4),
        color: Some(color),
        tags,
        exif_date,
        seed: Some(1),
    }
}

pub fn write_real_jpeg(path: &Path, color: [u8; 3]) {
    let spec = photo_spec(color, None, None);
    generate_photo_file(&spec, path).expect("write real jpeg");
}

pub fn write_real_jpeg_with_xmp_keywords(path: &Path, color: [u8; 3], keywords: &[&str]) {
    let tags: Vec<String> = keywords.iter().map(|s| s.to_string()).collect();
    let spec = photo_spec(color, Some(tags), None);
    generate_photo_file(&spec, path).expect("write jpeg with keywords");
}

pub fn write_real_jpeg_with_exif(path: &Path, color: [u8; 3], exif_date: &str) {
    let spec = photo_spec(color, None, Some(exif_date.to_string()));
    generate_photo_file(&spec, path).expect("write jpeg with exif");
}

pub fn write_real_jpeg_with_xmp_and_exif(
    path: &Path,
    color: [u8; 3],
    keywords: &[&str],
    exif_date: Option<&str>,
) {
    let tags: Vec<String> = keywords.iter().map(|s| s.to_string()).collect();
    let spec = photo_spec(color, Some(tags), exif_date.map(|d| d.to_string()));
    generate_photo_file(&spec, path).expect("write jpeg with metadata");
}

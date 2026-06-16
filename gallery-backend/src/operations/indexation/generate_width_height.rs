use super::video_ffprobe::video_width_height;
use crate::public::structure::abstract_data::AbstractData;
use anyhow::{Context, Result};
use image::DynamicImage;

/// Return `(width, height)` for an already‑decoded **image**.
/// Pure function ‑ no fallible operations.
pub fn generate_image_width_height(dynamic_image: &DynamicImage) -> (u32, u32) {
    (dynamic_image.width(), dynamic_image.height())
}

/// Probe a video file using `ffprobe` (through `video_width_height`) to
/// obtain `(width, height)`, adding explicit context to every `?` site.
pub fn generate_video_width_height(abstract_data: &AbstractData) -> Result<(u32, u32)> {
    let source = abstract_data.source_path_string();

    let width = video_width_height("width", source)
        .context(format!("failed to obtain video width for {source:?}"))?;
    let height = video_width_height("height", source)
        .context(format!("failed to obtain video height for {source:?}"))?;

    Ok((width, height))
}

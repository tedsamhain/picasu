use log::{error, info};
use std::process::Command;

pub fn check_ffmpeg_and_ffprobe() {
    for command in &["ffmpeg", "ffprobe"] {
        match Command::new(command).arg("-version").output() {
            Ok(output) if output.status.success() => {
                let version_info = String::from_utf8_lossy(&output.stdout);
                let version_number = version_info
                    .lines()
                    .next()
                    .unwrap_or("Unknown version")
                    .split_whitespace()
                    .nth(2) // Get the third word
                    .unwrap_or("Unknown");
                info!("{command} version: {version_number}");
            }
            Ok(_) => {
                error!(
                    "`{command}` command was found, but it returned an error. Please ensure it's correctly installed."
                );
            }
            Err(_) => {
                error!(
                    "`{command}` is not installed or not available in PATH. Please install it before running the application."
                );
            }
        }
    }
}

//==============================================================================
// lib.rs
// Description: Shared library containing common FFmpeg logic and helper functions
// Version: 0.1.0
//==============================================================================

use std::process::Command; // Needed for has_encoder
/// [LIB-01] Path used to check if the file exists before it is processed by FFmpeg
// used by: trim-clip
use std::path::Path;

/// [LIB-02] Represents basic metadata about a media file.
// used by: trim-clip
pub struct MediaInfo {
    pub stem: String,
    pub extension: String,
}

/// [LIB-03] Extracts the file stem (filename without extension) and the lowercase extension.
// used by: trim-clip
pub fn get_media_info(path_str: &str) -> MediaInfo {
    let path = Path::new(path_str);
    MediaInfo {
        stem: path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output")
            .to_string(),
        extension: path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase(),
    }
}

/// [LIB-04] Converts sexagesimal timestamps (HH:MM:SS.mmm or MM:SS) or plain second strings into total seconds.
// used by: trim-clip
pub fn parse_to_seconds(timestamp: &str) -> f64 {
    let parts: Vec<&str> = timestamp.split(':').collect();
    match parts.len() {
        3 => {
            let h: f64 = parts[0].parse().unwrap_or(0.0);
            let m: f64 = parts[1].parse().unwrap_or(0.0);
            let s: f64 = parts[2].parse().unwrap_or(0.0);
            (h * 3600.0) + (m * 60.0) + s
        }
        2 => {
            let m: f64 = parts[0].parse().unwrap_or(0.0);
            let s: f64 = parts[1].parse().unwrap_or(0.0);
            (m * 60.0) + s
        }
        _ => timestamp.parse().unwrap_or(0.0),
    }
}

/// [LIB-05] Formats a float of total seconds back into a sexagesimal string (HH:MM:SS or HH:MM:SS.mmm).
// used by: trim-clip
pub fn format_seconds(total_sec: f64) -> String {
    let h = (total_sec / 3600.0).floor() as u32;
    let m = ((total_sec % 3600.0) / 60.0).floor() as u32;
    let s = (total_sec % 60.0).floor() as u32;
    let ms = (total_sec.fract() * 1000.0).round() as u32;

    if ms > 0 {
        format!("{:02}:{:02}:{:02}.{:03}", h, m, s, ms)
    } else {
        format!("{:02}:{:02}:{:02}", h, m, s)
    }
}

/// [LIB-06] Checks if a specific encoder is available in the current FFmpeg installation.
// used by: trim-clip
pub fn has_encoder(encoder_name: &str) -> bool {
    let output = Command::new("ffmpeg")
        .args(["-hide_banner", "-h", &format!("encoder={}", encoder_name)])
        .output();
    
    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            !stdout.contains("not recognized")
        },
        Err(_) => false,
    }
}

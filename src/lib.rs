use std::process::Command; // Needed for has_encoder
use std::path::Path;

pub struct MediaInfo {
    pub stem: String,
    pub extension: String,
}

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

/// This was missing from your upload!
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



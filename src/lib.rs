//================================================================================
// lib.rs
// Description: Shared library containing common FFmpeg logic and helper functions
// Version: 0.1.0
//================================================================================

use std::process::Command; // Needed for [LIB-06]
/// [LIB-01] Path used to check if the file exists before it is processed by FFmpeg
// used by: 
// - audio-silence
// - chapter-add
// - chapter-csv
// - chapter-extract
// - clip-time
// - combine-clips
// - contact-sheet
// - ebu-meter
// - extract-frame
// - fade-clip
// - img2video
// - overlay-clip
// - overlay-pip
// - pan-scan
// - scene-cut
// - scene-cut-to
// - scene-detect
// - scene-detect-auto
// - scene-images
// - scopes
// - subtitle-add
// - trim-clip
// - trim-clip-to
// - trim-short
// - trim-remote-clip
// - waveform
// - webp
// - vid2gif
// - xfade
// - zoompan
use std::path::Path;

/// [LIB-02] Represents basic metadata about a media file.
// used by: 
// - extract-frame
// - trim-clip
// - trim-clip-to
pub struct MediaInfo {
    pub stem: String,
    pub extension: String,
}

/// [LIB-03] Extracts the file stem (filename without extension) and the lowercase extension.
// used by: 
// - audio-silence
// - chapter-add
// - clip-time
// - combine-clips
// - contact-sheet
// - extract-frame
// - fade-clip
// - img2video
// - pan-scan
// - overlay-clip
// - overlay-pip
// - scene-cut
// - scene-cut-to
// - scene-detect
// - scene-detect-auto
// - scene-images
// - subtitle-add
// - trim-clip
// - trim-clip-to
// - trim-short
// - waveform
// - webp
// - vid2gif
// - xfade
// - zoompan
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
// used by: 
// - chapter-csv
// - contact-sheet
// - fade-clip
// - img2video
// - overlay-clip
// - overlay-pip
// - pan-scan
// - scene-cut
// - scene-detect-auto
// - scene-time
// - scene-images
// - sexagesimal-time
// - trim-clip
// - trim-short
// - xfade
// - zoompan
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
// used by: 
// - contact-sheet
// - sexagesimal-time
// - trim-clip
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
// used by: 
// - trim-clip
// - trim-clip-to
// - scene-cut-to
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

/// [LIB-07] Subtracts the start time from the end time to calculate duration in seconds.
// used by: 
// - sexagesimal-time
// - trim-clip-to
pub fn calculate_duration(start: &str, end: &str) -> f64 {
    let start_sec = parse_to_seconds(start);
    let end_sec = parse_to_seconds(end);
    end_sec - start_sec
}

/// [LIB-08] Uses ffprobe to get the total duration of a video file in seconds.
// used by: 
// - combine-clips
// - contact-sheet
// - scene-detect
// - xfade
pub fn get_video_duration(path: &str) -> f64 {
    let output = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-show_entries", "format=duration",
            "-of", "default=noprint_wrappers=1:nokey=1",
            path,
        ])
        .output()
        .expect("Failed to execute ffprobe");
    
    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<f64>()
        .unwrap_or(0.0)
}

/// [LIB-09] Formats total seconds into HH:MM:SS.mmm (with mandatory milliseconds)
// used by: 
// - chapter-extract
// - combine-clips
// - fade-clip
// - img2video
// - overlay-clip
// - overlay-pip
// - pan-scan
// - scene-cut
// - scene-detect
// - scene-detect-auto
// - scene-time
// - scene-images
// - trim-short
// - xfade
// - zoompan
pub fn format_seconds_ms(total_sec: f64) -> String {
    let h = (total_sec / 3600.0) as u32;
    let m = ((total_sec % 3600.0) / 60.0) as u32;
    let s = (total_sec % 60.0) as u32;
    let ms = ((total_sec.fract()) * 1000.0).round() as u32;

    format!("{:02}:{:02}:{:02}.{:03}", h, m, s, ms)
}

/// [LIB-10] Formats a time string for use in a filename by replacing colons with dashes.
// used by: 
// - combine-clips
// - contact-sheet
// - extract-frame
// - fade-clip
// - img2video
// - scene-cut
// - scene-detect
// - scene-detect-auto
// - scene-images
// - trim-short
/// Automatically keeps colons for Linux/Unix and uses dashes for Windows.
pub fn format_time_for_filename(time: &str) -> String {
    if cfg!(target_os = "windows") {
        // Windows compatibility: replace ":" with "-"
        time.replace(':', "-")
    } else {
        // Linux, NixOS, macOS, FreeBSD: keep the ":"
        time.to_string()
    }
}

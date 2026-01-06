//==============================================================================
// scene-detect-auto
// Description: Automated scene detection, timing, and cutting in one command
// References: [LIB-01], [LIB-03], [LIB-04], [LIB-09], [LIB-10]
//==============================================================================

use clap::Parser;
use std::process::{Command, Stdio};
use std::path::Path;
use std::fs::{File, write};
use std::io::{BufRead, BufReader};
use ffmpeg_rust_scripts::{get_media_info, parse_to_seconds, format_seconds_ms};

#[derive(Parser, Debug)]
#[command(
    author, 
    version,
    about = "Automated scene detection and video splitting",
    after_help = "Example:\n  scene-detect-auto -i input.mp4\n\nDependencies:\n  ffmpeg, ffprobe: https://www.ffmpeg.org/\n\nNotes:\n  Creates detection.txt and cutlist.txt automatically.",
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// input video file
    #[arg(short = 'i', required = true)]
    input: String,

    /// detection threshold (0.0 to 1.0)
    #[arg(short = 't', default_value = "0.3")]
    threshold: String,

    /// Print help
    #[arg(short = 'h', long = "help", action = clap::ArgAction::Help)]
    help: Option<bool>,

    /// Print version
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    version: Option<bool>,
}

fn main() {
    let args = Args::parse();
    if !Path::new(&args.input).exists() {
        eprintln!("Error: Input file '{}' not found.", args.input);
        std::process::exit(1);
    }

    let info = get_media_info(&args.input);
    let detect_file = "detection.txt";
    let cutlist_file = "cutlist.txt";

    // STEP 1: SCENE DETECTION
    let output = Command::new("ffmpeg")
        .args([
            "-hide_banner", "-i", &args.input,
            "-filter_complex", &format!("select='gt(scene,{})',showinfo", args.threshold),
            "-f", "null", "-",
        ])
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to run ffmpeg detection");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut timestamps = Vec::new();
    timestamps.push("0.000000".to_string()); 

    for line in stderr.lines() {
        if line.contains("pts_time:") {
            if let Some(pts_part) = line.split("pts_time:").nth(1) {
                if let Some(ts) = pts_part.split_whitespace().next() {
                    timestamps.push(ts.to_string());
                }
            }
        }
    }
    write(detect_file, timestamps.join("\n")).expect("Unable to write detection.txt");

    // STEP 2: CREATE CUTLIST
    let mut cutlist_content = String::new();
    for i in 0..timestamps.len() {
        let start_sec: f64 = timestamps[i].parse().unwrap_or(0.0);
        let end_sec: f64 = if i + 1 < timestamps.len() {
            timestamps[i + 1].parse().unwrap_or(0.0)
        } else {
            let dur_output = Command::new("ffprobe")
                .args(["-v", "error", "-show_entries", "format=duration", "-of", "default=noprint_wrappers=1:nokey=1", &args.input])
                .output()
                .expect("Failed to get duration");
            String::from_utf8_lossy(&dur_output.stdout).trim().parse().unwrap_or(0.0)
        };

        let duration = end_sec - start_sec;
        if duration > 0.1 { 
            cutlist_content.push_str(&format!("{},{}\n", format_seconds_ms(start_sec), format_seconds_ms(duration)));
        }
    }
    write(cutlist_file, cutlist_content).expect("Unable to write cutlist.txt");

    // STEP 3: CUT SCENES
    let file = File::open(cutlist_file).expect("Failed to open cutlist");
    let reader = BufReader::new(file);

    for (index, line) in reader.lines().enumerate() {
        let line = line.expect("Failed to read line");
        let parts: Vec<&str> = line.split(',').collect();

        if parts.len() == 2 {
            let start_raw = parts[0].trim();
            let dur_raw = parts[1].trim();

            let start_sec = parse_to_seconds(start_raw);
            let duration_sec = parse_to_seconds(dur_raw);
            let end_ts = format_seconds_ms(start_sec + duration_sec);

            let output_name = format!(
                "{}-scene-{:03}-[{}–{}].mp4",
                info.stem, index + 1, start_raw, end_ts
            );

            // Using Input Seeking (-ss and -t BEFORE -i) for speed
            Command::new("ffmpeg")
                .args([
                    "-hide_banner", "-v", "error", "-stats",
                    "-ss", start_raw,
                    "-t", dur_raw,
                    "-i", &args.input,
                    "-c:a", "aac", "-c:v", "libx264", "-profile:v", "high",
                    "-pix_fmt", "yuv420p", "-movflags", "+faststart", "-y",
                    &output_name,
                ])
                .status()
                .expect("Failed to execute FFmpeg");
        }
    }
}

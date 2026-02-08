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
use ffmpeg_rust_scripts::{get_media_info, parse_to_seconds, format_seconds_ms, format_time_for_filename};

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

/// check if the nvenc code is available
fn has_nvenc() -> bool {
    let output = Command::new("ffmpeg").args(["-encoders"]).output().expect("ffmpeg check failed");
    String::from_utf8_lossy(&output.stdout).contains("hevc_nvenc")
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

    // 1. Determine Encoder once at the start (Print ONLY once)
    let use_nvenc = has_nvenc();
    if use_nvenc {
        println!("+ Using High-Fidelity Hardware Encoding (NVENC)");
    } else {
        println!("+ NVENC not found. Falling back to libx264 (CRF 18)");
    }

    // STEP 2: SCENE DETECTION
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

    // STEP 3: CREATE CUTLIST
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

    // STEP 4: CUT SCENES
    let file = File::open(cutlist_file).expect("Failed to open cutlist");
    let reader = BufReader::new(file);

    for (index, line) in reader.lines().enumerate() {
        let line = line.expect("Failed to read line");
        let parts: Vec<&str> = line.split(',').collect();

        if parts.len() != 2 { continue; }
            let start_raw = parts[0].trim();
            let dur_raw = parts[1].trim();

            let start_sec = parse_to_seconds(start_raw);
            let duration_sec = parse_to_seconds(dur_raw);
            let end_sec = start_sec + duration_sec;

            // LIB-09: HH:MM:SS.mmm (Preserve milliseconds)
            let start_filename_raw = format_seconds_ms(start_sec);
            let end_filename_raw = format_seconds_ms(end_sec);

            // 5. Apply LIB-10 OS check
            let start_fs = format_time_for_filename(&start_filename_raw);
            let end_fs = format_time_for_filename(&end_filename_raw);

            let output_name = format!(
                "{}-scene-{:03}-[{}–{}].mp4",
                info.stem, index + 1, start_fs, end_fs
            );

            println!("Cutting Scene {}: {} -> {}", index + 1, start_filename_raw, end_filename_raw);

        let mut cmd = Command::new("ffmpeg");
        cmd.args([
            "-hide_banner", "-v", "error", "-stats",
            "-ss", start_raw,
            "-t", dur_raw,
            "-i", &args.input,
        ]);

        // 3. Encoder Logic (No printing inside the loop)
        if use_nvenc {
            cmd.args([
                "-c:v", "hevc_nvenc",
                "-tune", "hq",
                "-preset", "p7",
                "-rc", "vbr",
                "-multipass", "fullres",
                "-rc-lookahead", "32",
                "-spatial-aq", "1"
                "-cq", "20",
                "-b:v", "0",
            ]);
        } else {
            cmd.args(["-c:v", "libx264", "-crf", "18"]);
        }

        cmd.args([
            "-pix_fmt", "yuv420p",
            "-c:a", "aac",
            "-movflags", "+faststart",
            "-y",
            &output_name
        ]);

        let status = cmd.status().expect("Failed to execute FFmpeg");

        if !status.success() {
            eprintln!("Error: FFmpeg failed on scene {}.", index + 1);
        }
    }
}

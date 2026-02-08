//==============================================================================
// scene-cut-to
// Description: Split video into clips using start time and duration (End-point seeking)
// References: [LIB-01], [LIB-03], [LIB-06], [LIB-10]
//==============================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path;
use std::fs::File;
use std::io::{BufRead, BufReader};
use ffmpeg_rust_scripts::{get_media_info, parse_to_seconds, format_seconds_ms, format_time_for_filename};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "split video into clips using a start,duration cutlist by calculating end-point",
    after_help = "Example:\n  scene-cut-to -i input.mp4 -c cutlist.txt\n\nDependencies:\n  ffmpeg: https://www.ffmpeg.org/",
    override_usage = "scene-cut-to -i <INPUT> -c <CUTLIST> [OPTIONS]"
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// input video file
    #[arg(short = 'i', required = true, value_name = "INPUT")]
    input: String,

    /// cutlist file comma-separated start,duration
    #[arg(short = 'c', required = true, value_name = "CUTLIST")]
    cutlist: String,

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
    let file = File::open(&args.cutlist).expect("Failed to open cutlist");
    let reader = BufReader::new(file);

    // 1. Determine Encoder once at the start
    let use_nvenc = has_nvenc();
    if use_nvenc {
        println!("+ Using High-Fidelity Hardware Encoding (NVENC)");
    } else {
        println!("+ NVENC not found. Falling back to libx264 (CRF 18)");
    }

    // 2. PROCESS CUTLIST
    for (index, line) in reader.lines().enumerate() {
        let line = line.expect("Failed to read line");
        let parts: Vec<&str> = line.split(',').collect();

            if parts.len() != 2 { continue; }
            let start_raw = parts[0].trim();
            let duration_raw = parts[1].trim();

            // 3. Prepare raw timestamps for calculation
            let start_sec = parse_to_seconds(start_raw);
            let duration_sec = parse_to_seconds(duration_raw);
            let end_sec = start_sec + duration_sec;

            // 4. LIB-09: Get full HH:MM:SS.mmm (Preserving milliseconds)
            let start_filename_raw = format_seconds_ms(start_sec);
            let end_filename_raw = format_seconds_ms(end_sec);

            // 5. Apply LIB-10 OS check for the filename
            let start_ts = format_time_for_filename(&start_filename_raw);
            let end_ts = format_time_for_filename(&end_filename_raw);

            // Filename with original colons preserved
            let output_name = format!(
                "{}-scene-{:03}-[{}-{}].mp4",
                info.stem,
                index + 1,
                start_ts,
                end_ts
            );

            println!("Processing Scene {}: {} -> {}", index + 1, start_raw, format_seconds_ms(end_sec));

        let mut cmd = Command::new("ffmpeg");
        cmd.args([
            "-hide_banner",
            "-v", "error",
            "-stats",
            "-i", &args.input,     // Input Seeking (Frame Accurate)
            "-ss", start_raw,
            "-to", &end_filename_raw,    // Use the calculated ms timestamp
        ]);

        // 6. High-Fidelity Encoder Logic
        if use_nvenc {
            cmd.args([
                "-c:v", "hevc_nvenc",
                "-preset", "p7",
                "-tune", "hq",
                "-rc", "vbr",
                "-multipass", "fullres",
                "-rc-lookahead", "32",
                "-spatial-aq", "1",
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
            &output_name,
        ]);

        let status = cmd.status().expect("Failed to execute FFmpeg");

        if !status.success() {
            eprintln!("Error processing scene {}", index + 1);
        }
    }
}

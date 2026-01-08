//==============================================================================
// scene-cut
// Description: Read a cutlist and split a video into individual scene clips
// References: [LIB-01], [LIB-03], [LIB-04], [LIB-09], [LIB-10]
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
    about = "Split a video into individual scenes based on a cutlist",
    after_help = "Example:\n  scene-cut -i input.mp4 -c cutlist.txt\n\nDependencies:\n  ffmpeg: https://www.ffmpeg.org/",
    override_usage = "scene-cut -i <INPUT> -c <CUTLIST> [OPTIONS]"
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// Input video file
    #[arg(short = 'i', required = true)]
    input: String,

    /// Cutlist file (comma-separated start,duration)
    #[arg(short = 'c', required = true)]
    cutlist: String,

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
        eprintln!("Error: Input video '{}' not found.", args.input);
        std::process::exit(1);
    }
    if !Path::new(&args.cutlist).exists() {
        eprintln!("Error: Cutlist file '{}' not found.", args.cutlist);
        std::process::exit(1);
    }

    let info = get_media_info(&args.input);
    let file = File::open(&args.cutlist).expect("Failed to open cutlist");
    let reader = BufReader::new(file);

    for (index, line) in reader.lines().enumerate() {
        if let Ok(l) = line {
            let parts: Vec<&str> = l.split(',').collect();
            if parts.len() != 2 { continue; }

            let start_raw = parts[0].trim();
            let duration_raw = parts[1].trim();

            let start_sec = parse_to_seconds(start_raw);
            let duration_sec = parse_to_seconds(duration_raw);
            let end_sec = start_sec + duration_sec;

            // Extract HH:MM:SS with colons
            let start_filename_raw = format_seconds_ms(start_sec).split('.').next().unwrap_or("00:00:00").to_string();
            let end_filename_raw = format_seconds_ms(end_sec).split('.').next().unwrap_or("00:00:00").to_string();

            // Apply LIB-10 OS check to replace colons with dashes if on Windows
            let start_ts = format_time_for_filename(&start_filename_raw);
            let end_ts = format_time_for_filename(&end_filename_raw);

            // FIXED: Single dash between timestamps
            let output_name = format!("{}-scene-{:03}-[{}-{}].mp4", 
                info.stem, index + 1, start_ts, end_ts);

            println!("Processing Scene {}: {} -> {}", index + 1, start_raw, format_seconds_ms(end_sec));

            let status = Command::new("ffmpeg")
                .args([
                    "-hide_banner", "-loglevel", "error", "-stats",
                    "-ss", start_raw,
                    "-t", duration_raw,
                    "-i", &args.input,
                    "-c:v", "libx264",      // re-encode video
                    "-profile:v", "high",   // high profile
                    "-pix_fmt", "yuv420p",  // pixel format
                    "-c:a", "aac",          // re-encode audio
                    "-movflags", "+faststart", // faststart for web
                    "-f", "mp4",            // force mp4 format
                    &output_name
                ])
                .status()
                .expect("Failed to execute FFmpeg");

            if !status.success() {
                eprintln!("Error: FFmpeg failed on scene {}", index + 1);
            }
        }
    }
}

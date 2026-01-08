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

fn main() {
    let args = Args::parse();

    if !Path::new(&args.input).exists() {
        eprintln!("Error: Input file '{}' not found.", args.input);
        std::process::exit(1);
    }

    let info = get_media_info(&args.input);
    let file = File::open(&args.cutlist).expect("Failed to open cutlist");
    let reader = BufReader::new(file);

    for (index, line) in reader.lines().enumerate() {
        let line = line.expect("Failed to read line");
        let parts: Vec<&str> = line.split(',').collect();

        if parts.len() == 2 {
            let start_raw = parts[0].trim();
            let duration_raw = parts[1].trim();

            // 1. Prepare raw timestamps for calculation
            let start_sec = parse_to_seconds(start_raw);
            let duration_sec = parse_to_seconds(duration_raw);
            let end_sec = start_sec + duration_sec;

            // 2. Get HH:MM:SS strings (without milliseconds) for the filename
            let end_ts_raw = format_seconds_ms(end_sec);
            let start_clean = start_raw.split('.').next().unwrap_or("00:00:00");
            let end_clean = end_ts_raw.split('.').next().unwrap_or("00:00:00");

            // 3. Apply LIB-10 OS check for the filename
            let start_filename = format_time_for_filename(start_clean);
            let end_filename = format_time_for_filename(end_clean);

            // Filename with original colons preserved
            let output_name = format!(
                "{}-scene-{:03}-[{}-{}].mp4",
                info.stem,
                index + 1,
                start_filename,
                end_filename
            );

            // Output seeking: -i FIRST, then -ss and -to (using calculated end_ts)
            let status = Command::new("ffmpeg")
                .args([
                    "-hide_banner",
                    "-v", "error",
                    "-stats",
                    "-i", &args.input,
                    "-ss", start_raw,
                    "-to", &end_ts_raw,
                    "-c:a", "aac",
                    "-c:v", "libx264",
                    "-profile:v", "high",
                    "-pix_fmt", "yuv420p",
                    "-movflags", "+faststart",
                    "-y",
                    &output_name,
                ])
                .status()
                .expect("Failed to execute FFmpeg");

            if !status.success() {
                eprintln!("Error processing scene {}", index + 1);
            }
        }
    }
}

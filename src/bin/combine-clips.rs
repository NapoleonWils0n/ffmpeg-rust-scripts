//==============================================================================
// combine-clips
// Description: Combine an audio file and video together (remux)
// References: [LIB-01], [LIB-03], [LIB-08], [LIB-09]
//==============================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path;
use ffmpeg_scripts_rust::{get_media_info, get_video_duration, format_seconds_ms};

#[derive(Parser, Debug)]
#[command(
    author, 
    version, 
    about = "Combine audio and video files",
    after_help = "Dependencies:\nffmpeg, ffprobe: https://www.ffmpeg.org/",
    override_usage = "combine-clips -i <INPUT> -a <AUDIO> [OPTIONS]"
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// Input video file (-i)
    #[arg(short = 'i', required = true)]
    input: String,

    /// Input audio file (-a)
    #[arg(short = 'a', required = true)]
    audio: String,

    /// Output file (optional)
    #[arg(short = 'o')]
    outfile: Option<String>,

    /// Print version
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    version: Option<bool>,

    /// Print help
    #[arg(short = 'h', long = "help", action = clap::ArgAction::Help)]
    help: Option<bool>,
}

fn main() {
    let args = Args::parse();

    // Check if input files exist 
    if !Path::new(&args.input).exists() || !Path::new(&args.audio).exists() {
        eprintln!("Error: Input video or audio file not found.");
        std::process::exit(1);
    }

    // Get metadata for naming 
    let info = get_media_info(&args.input);
    let duration_secs = get_video_duration(&args.input);
    let full_ts = format_seconds_ms(duration_secs);
    let timestamp = full_ts.split('.').next().unwrap_or("00:00:00");

    // Default output filename: video-stem-combined-[duration].mp4 
    let final_output = args.outfile.unwrap_or_else(|| {
        format!("./{}-combined-[{}].mp4", info.stem, timestamp)
    });

    // Execute FFmpeg remux 
    // -c:v copy -c:a copy remuxes without re-encoding 
    // -map 0:v:0 -map 1:a:0 ensures we take video from first input and audio from second 
    let status = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel", "error",
            "-stats",
            "-i", &args.input,
            "-i", &args.audio,
            "-c:v", "copy",
            "-c:a", "copy",
            "-map", "0:v:0",
            "-map", "1:a:0",
            "-pix_fmt", "yuv420p",
            "-movflags", "+faststart",
            "-f", "mp4",
            "-y",
            &final_output,
        ])
        .status()
        .expect("Failed to execute FFmpeg");

    if !status.success() {
        std::process::exit(1);
    }
}

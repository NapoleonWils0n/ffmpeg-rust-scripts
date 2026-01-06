//==============================================================================
// img2video
// Description: Convert a static image (png, jpg, jpeg) to a video file
// References: [LIB-01] Path validation, [LIB-03] get_media_info, 
//             [LIB-04] parse_to_seconds, [LIB-09] format_seconds_ms
//==============================================================================

use clap::Parser;
use std::process::{Command, Stdio};
use std::path::Path;
use ffmpeg_rust_scripts::{get_media_info, format_seconds_ms, parse_to_seconds};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Convert a static image to a video file with a specified duration",
    after_help = "Example:\n  img2video -i input.png -d 00:00:10 -o output.mp4\n\n  \
                  Dependencies:\n  \
                  ffmpeg: https://www.ffmpeg.org/",
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// Input image file (png, jpg, jpeg)
    #[arg(short = 'i', required = true)]
    infile: String,

    /// Duration (e.g., 10 or 00:00:10)
    #[arg(short = 'd', required = true)]
    duration: String,

    /// Output file (optional)
    #[arg(short = 'o')]
    outfile: Option<String>,

    /// Print help
    #[arg(short = 'h', long = "help", action = clap::ArgAction::Help)]
    help: Option<bool>,

    /// Print version
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    version: Option<bool>,
}

fn main() {
    let args = Args::parse();

    // 1. Validate input exists [LIB-01]
    if !Path::new(&args.infile).exists() {
        eprintln!("Error: Input image '{}' not found.", args.infile);
        std::process::exit(1);
    }

    // 2. Parse duration and determine output filename
    let duration_secs = parse_to_seconds(&args.duration); // [LIB-04]
    if duration_secs <= 0.0 {
        eprintln!("Error: Invalid duration '{}'.", args.duration);
        std::process::exit(1);
    }

    let info = get_media_info(&args.infile); // [LIB-03]
    
    // Format duration to HH:MM:SS for the filename [LIB-09]
    let full_ts = format_seconds_ms(duration_secs);
    let timestamp = full_ts.split('.').next().unwrap_or("00:00:00");
    
    let final_output = args.outfile.unwrap_or_else(|| {
        format!("{}-[{}].mp4", info.stem, timestamp)
    });

    // 3. Run FFmpeg to convert image to video
    let status = Command::new("ffmpeg")
        .args([
            "-loglevel", "error",
            "-loop", "1",
            "-i", &args.infile,
            "-c:v", "libx264",
            "-t", &duration_secs.to_string(),
            "-r", "30",
            "-pix_fmt", "yuv420p",
            "-movflags", "+faststart",
            "-y",
            &final_output,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("Failed to execute FFmpeg");

    if !status.success() {
        eprintln!("Error: FFmpeg failed to convert image to video.");
        std::process::exit(1);
    }
}

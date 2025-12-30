//==============================================================================
// overlay-clip
// Description: Overlay a video onto a background clip at a specific time
// References: [LIB-01], [LIB-03], [LIB-04], [LIB-09]
//==============================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path;
use ffmpeg_scripts_rust::{get_media_info, parse_to_seconds, format_seconds_ms};

#[derive(Parser, Debug)]
#[command(
    author, 
    version, 
    about = "Overlay one video clip on top of another video clip",
    override_usage = "overlay-clip -a <INPUT> -b <OVERLAY> -p <POSITION> [-o <OUTPUT>]"
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// Bottom video (-a)
    #[arg(short = 'a', required = true)]
    input: String,

    /// Overlay video (-b)
    #[arg(short = 'b', required = true)]
    overlay: String,

    /// Time to start the overlay (e.g., 5 or 00:00:05)
    #[arg(short = 'p', required = true)]
    position: String,

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

    if !Path::new(&args.input).exists() || !Path::new(&args.overlay).exists() {
        eprintln!("Error: One or more input files not found.");
        std::process::exit(1);
    }

    let start_secs = parse_to_seconds(&args.position);
    let info = get_media_info(&args.input);
    let fg_info = get_media_info(&args.overlay);
    
    // Format the position for the filename (e.g., 00:00:10)
    let full_ts = format_seconds_ms(start_secs);
    let timestamp = full_ts.split('.').next().unwrap_or("00:00:00");
    
    let final_output = args.outfile.unwrap_or_else(|| {
        format!("{}-overlay-{}-[{}].mp4", info.stem, fg_info.stem, timestamp)
    });

    // Filter: setpts delays the foreground, overlay=eof_action=pass keeps bg after fg ends
    let filter = format!("[1:v]setpts=PTS+{}/TB[fg]; [0:v][fg]overlay=eof_action=pass", start_secs);

    let status = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel", "error",
            "-stats",
            "-i", &args.input,
            "-i", &args.overlay,
            "-filter_complex", &filter,
            "-map", "0:a?", 
            "-c:v", "libx264",
            "-crf", "18",
            "-pix_fmt", "yuv420p",
            "-y",
            &final_output,
        ])
        .status()
        .expect("Failed to execute FFmpeg");

    if !status.success() {
        std::process::exit(1);
    }
}

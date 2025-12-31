//==============================================================================
// fade-in
// Description: Apply a fade-in effect to both video and audio with timestamped output
// References: [LIB-01], [LIB-03], [LIB-04], [LIB-09]
//==============================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path;
use ffmpeg_scripts_rust::{get_media_info, format_seconds_ms, parse_to_seconds};

#[derive(Parser, Debug)]
#[command(
    author, 
    version, 
    about = "Fade in a video and audio clip",
    after_help = "Example:\n  fade-clip -i input.mp4 -d 00:00:02\n\nDependencies:\n  ffmpeg: https://www.ffmpeg.org/",
    override_usage = "fade-clip [OPTIONS] -i <INFILE>"
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// Input video file
    #[arg(short = 'i', required = true)]
    infile: String,

    /// Fade duration (e.g., 2 or 00:00:02)
    #[arg(short = 'd', default_value = "00:00:00.500")]
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

    if !Path::new(&args.infile).exists() {
        eprintln!("Error: Input file '{}' not found.", args.infile);
        std::process::exit(1);
    }

    let dur_secs = parse_to_seconds(&args.duration);
    let info = get_media_info(&args.infile);
    
    // Format the duration for the filename (e.g., 00:00:02)
    let full_ts = format_seconds_ms(dur_secs);
    let timestamp = full_ts.split('.').next().unwrap_or("00:00:00");
    
    let final_output = args.outfile.unwrap_or_else(|| {
        format!("{}-faded-in-[{}].mp4", info.stem, timestamp)
    });

    // Filters
    let v_filter = format!("fade=t=in:st=0:d={}", dur_secs);
    let a_filter = format!("afade=t=in:st=0:d={}", dur_secs);

    let status = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel", "error",
            "-stats",
            "-i", &args.infile,
            "-vf", &v_filter,
            "-af", &a_filter,
            "-c:v", "libx264",
            "-crf", "18",
            "-pix_fmt", "yuv420p",
            "-c:a", "aac",
            "-y",
            &final_output,
        ])
        .status()
        .expect("Failed to execute FFmpeg");

    if !status.success() {
        eprintln!("Error: FFmpeg execution failed.");
        std::process::exit(1);
    }
}

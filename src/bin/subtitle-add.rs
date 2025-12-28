//==============================================================================
// subtitle-add
// Description: Mux external subtitles into a video file as a toggleable track
// References: [LIB-01] Path validation, [LIB-03] get_media_info
//==============================================================================

use clap::Parser;
use std::process::{Command, Stdio};
use std::path::Path;
use ffmpeg_scripts_rust::get_media_info;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Add SRT/VTT subtitles to a video as a track you can toggle on and off",
    after_help = "Example:\n  subtitle-add -i input.mp4 -s input.srt -l eng -o output.mp4\n\n  \
                  Dependencies:\n  \
                  ffmpeg: https://www.ffmpeg.org/",
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// Input video file
    #[arg(short = 'i', required = true)]
    infile: String,

    /// Subtitle file (SRT or VTT)
    #[arg(short = 's', required = true)]
    subfile: String,

    /// Language code (e.g., eng, ita, fra)
    #[arg(short = 'l', default_value = "eng")]
    lang: String,

    /// Output file (optional, defaults to input-subs.ext)
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

    // 1. Validate inputs exist
    if !Path::new(&args.infile).exists() {
        eprintln!("Error: Video file '{}' not found.", args.infile);
        std::process::exit(1);
    }
    if !Path::new(&args.subfile).exists() {
        eprintln!("Error: Subtitle file '{}' not found.", args.subfile);
        std::process::exit(1);
    }

    // 2. Determine output filename and subtitle codec
    let info = get_media_info(&args.infile);
    let final_output = args.outfile.unwrap_or_else(|| {
        format!("{}-subs.{}", info.stem, info.extension)
    });

    // Choose codec based on container
    // MP4/MOV use 'mov_text', MKV uses 'srt' or 'ass'
    let sub_codec = match info.extension.as_str() {
        "mp4" | "m4v" | "mov" => "mov_text",
        _ => "srt",
    };

    // 3. Run FFmpeg to mux the subtitles
    let status = Command::new("ffmpeg")
        .args([
            "-loglevel", "error",
            "-i", &args.infile,
            "-i", &args.subfile,
            "-map", "0",              // Map all streams from video
            "-map", "1",              // Map subtitle stream
            "-c", "copy",             // Copy existing video/audio
            "-c:s", sub_codec,        // Set specific subtitle codec
            &format!("-metadata:s:s:0"), &format!("language={}", args.lang),
            "-y",
            &final_output,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("Failed to execute FFmpeg");

    if !status.success() {
        eprintln!("Error: FFmpeg failed to add subtitles.");
        std::process::exit(1);
    }
}

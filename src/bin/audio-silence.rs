//==============================================================================
// audio-silence
// Description: Replace or add a silent audio track to a video file
// References: [LIB-01] Path validation, [LIB-03] get_media_info
//==============================================================================

use clap::Parser;
use std::process::{Command, Stdio};
use std::path::Path;
use ffmpeg_rust_scripts::get_media_info;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Replaces or adds a silent audio track to a video file",
    after_help = "Example:\n  audio-silence -i input.mp4 -c stereo -r 48000 -o output.mp4\n\n  \
                  Dependencies:\n  \
                  ffmpeg: https://www.ffmpeg.org/\n\n",
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// Input video file
    #[arg(short = 'i', required = true)]
    infile: String,

    /// Audio channels (mono or stereo)
    #[arg(short = 'c', default_value = "mono")]
    channels: String,

    /// Sample rate (e.g., 44100, 48000)
    #[arg(short = 'r', default_value = "44100")]
    rate: String,

    /// Output file (optional, defaults to input-silence.ext)
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

    // 1. Validate input exists
    if !Path::new(&args.infile).exists() {
        eprintln!("Error: Input file '{}' not found.", args.infile);
        std::process::exit(1);
    }

    // 2. Determine output filename
    let info = get_media_info(&args.infile);
    let final_output = args.outfile.unwrap_or_else(|| {
        format!("{}-silence.{}", info.stem, info.extension)
    });

    // Translate channel string to ffmpeg layout
    let layout = match args.channels.as_str() {
        "stereo" => "stereo",
        _ => "mono",
    };

    // 3. Run FFmpeg to replace audio with silence
    // Uses anullsrc to generate silent audio matching video duration
    let status = Command::new("ffmpeg")
        .args([
            "-loglevel", "error",
            "-i", &args.infile,
            "-f", "lavfi",
            "-i", &format!("anullsrc=channel_layout={}:sample_rate={}", layout, args.rate),
            "-c:v", "copy",           // Copy video without re-encoding
            "-c:a", "aac",            // Encode silence to AAC
            "-map", "0:v:0",          // Use first video stream from first input
            "-map", "1:a:0",          // Use first audio stream from silent source
            "-shortest",              // Ensure audio doesn't outlast video
            "-y",                     // Overwrite output
            &final_output,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("Failed to execute FFmpeg");

    if !status.success() {
        eprintln!("Error: FFmpeg failed to create silent audio track.");
        std::process::exit(1);
    }
}

//==============================================================================
// vid2gif
// Description: Convert video to high-quality GIF using a custom color palette
// References: [LIB-01], [LIB-03]
//==============================================================================

use clap::Parser;
use std::process::Command;
// [LIB-01] Path import used for file existence check
use std::path::Path;
// [LIB-03] Shared logic for extracting file stems and extensions
use ffmpeg_rust_scripts::get_media_info;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "convert video to high quality gif",
    after_help = "Example:\n  vid2gif -i input.mp4 -w 480 -f 15 -o animation.gif\n\nDependencies:\n  ffmpeg, ffprobe: https://www.ffmpeg.org/",
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// input video file
    #[arg(short = 'i', help = "input file")]
    infile: String,

    /// output width (maintains aspect ratio)
    #[arg(short = 'w', default_value = "320", help = "width")]
    width: i32,

    /// output frame rate
    #[arg(short = 'f', default_value = "10", help = "fps")]
    fps: i32,

    /// output file
    #[arg(short = 'o', help = "output file")]
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

    let info = get_media_info(&args.infile);
    let out = args.outfile.clone().unwrap_or_else(|| {
        format!("{}.gif", info.stem)
    });

    // FFmpeg filter chain:
    // 1. Scale and set FPS
    // 2. Split stream: one for palettegen, one for paletteuse
    // 3. Generate palette from the first stream
    // 4. Use palette on the second stream
    let filter = format!(
        "fps={},scale={}:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse",
        args.fps, args.width
    );

    let status = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-v", "warning",
            "-stats",
            "-i", &args.infile,
            "-vf", &filter,
            "-y", // Overwrite output without asking
            &out,
        ])
        .status()
        .expect("Failed to execute FFmpeg");

    if !status.success() {
        eprintln!("FFmpeg process exited with an error.");
    }
}

//==============================================================================
// webp
// Description: Convert video to an animated WebP file
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
    about = "convert video to an animated webp",
    after_help = "Example:\n  webp -i input.mp4 -w 480 -f 15 -o animation.webp\n\nDependencies:\n  ffmpeg, ffprobe: https://www.ffmpeg.org/\n\n",
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
    #[arg(short = 'h', long = "version", action = clap::ArgAction::Help)]
    help: Option<bool>,

    /// Print version
    #[arg(short = 'v', long = "help", action = clap::ArgAction::Version)]
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
        format!("{}.webp", info.stem)
    });

    // Fix for FFmpeg protocol handling
    let out_path = format!("./{}", out);

    // Filter: set fps, scale width while keeping aspect ratio (-1)
    let filter = format!("fps={},scale={}:-1:flags=lanczos", args.fps, args.width);

    let status = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-v", "warning",
            "-stats",
            "-i", &args.infile,
            "-vf", &filter,
            // loop 0 = infinite loop
            "-loop", "0",
            "-y",
            &out_path,
        ])
        .status()
        .expect("Failed to execute FFmpeg");

    if !status.success() {
        eprintln!("FFmpeg process exited with an error.");
    }
}

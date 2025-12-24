//==============================================================================
// waveform
// Description: Create a waveform image from a video or audio file
// References: [LIB-01], [LIB-03]
//==============================================================================

use clap::Parser;
use std::process::Command;
// [LIB-01] Path import used for file existence check
use std::path::Path;
// [LIB-03] Shared logic for extracting file stems and extensions
use ffmpeg_scripts_rust::get_media_info; 

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "create a waveform image from a video or audio file",
    after_help = "Example:\n  waveform -i input.mp4 -c orange -o waveform.png\n\nColors: https://ffmpeg.org/ffmpeg-utils.html#Color\n\nDependencies:\n  ffmpeg: https://www.ffmpeg.org/",
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// input file
    #[arg(short = 'i', help = "input file")]
    infile: String,

    /// waveform color
    #[arg(short = 'c', default_value = "white", help = "waveform color")]
    color: String,

    /// output width
    #[arg(short = 'w', default_value = "1280", help = "width")]
    width: i32,

    /// output height
    #[arg(short = 'e', default_value = "420", help = "height")]
    height: i32,

    /// output file
    #[arg(short = 'o', help = "output file")]
    outfile: Option<String>,

    /// Print help
    #[arg(short = 'h', action = clap::ArgAction::Help)]
    help: Option<bool>,

    /// Print version
    #[arg(short = 'v', action = clap::ArgAction::Version)]
    version: Option<bool>,
}

fn main() {
    let args = Args::parse();

    // Ensure the source file exists before calling FFmpeg
    if !Path::new(&args.infile).exists() {
        eprintln!("Error: Input file '{}' not found.", args.infile);
        std::process::exit(1);
    }

    let info = get_media_info(&args.infile);
    
    // Default naming: input-waveform.png
    let out = args.outfile.clone().unwrap_or_else(|| {
        format!("{}-waveform.png", info.stem)
    });

    // Use ./ prefix to ensure FFmpeg doesn't treat colons in the filename as a protocol
    let out_path = format!("./{}", out);

    // Build the showwavespic filter string
    let filter = format!(
        "showwavespic=s={}x{}:colors={}", 
        args.width, args.height, args.color
    );

    println!("Generating waveform...");

    let status = Command::new("ffmpeg")
        .args([
            "-hide_banner", 
            "-v", "fatal", // Suppress non-fatal header parsing warnings (e.g., Opus packets)
            "-stats",
            "-i", &args.infile,
            "-filter_complex", &filter,
            "-frames:v", "1",
            &out_path
        ])
        .status()
        .expect("Failed to execute FFmpeg");

    if status.success() {
        println!("Waveform saved to: {}", out);
    }
}

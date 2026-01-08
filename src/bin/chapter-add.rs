//==============================================================================
// chapter-add
// Description: Mux FFmpeg metadata chapters into a video or audio file
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
    about = "Mux FFmpeg metadata chapters into a video or audio file without re-encoding",
    after_help = "Example:\n  chapter-add -i input.mp4 -m metadata.txt -o output.mp4\n\n  \
                  Dependencies:\n  \
                  ffmpeg: https://www.ffmpeg.org/",
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// Input video or audio file
    #[arg(short = 'i', required = true)]
    infile: String,

    /// Metadata text file (FFMPEG METADATA format)
    #[arg(short = 'm', required = true)]
    metafile: String,

    /// Output file (optional, defaults to input-chapters.ext)
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
        eprintln!("Error: Input file '{}' not found.", args.infile);
        std::process::exit(1);
    }
    if !Path::new(&args.metafile).exists() {
        eprintln!("Error: Metadata file '{}' not found.", args.metafile);
        std::process::exit(1);
    }

    // 2. Determine output filename
    let info = get_media_info(&args.infile);
    let final_output = args.outfile.unwrap_or_else(|| {
        format!("{}-chapters.{}", info.stem, info.extension)
    });

    // 3. Run FFmpeg quietly to mux the metadata
    let status = Command::new("ffmpeg")
        .args([
            "-loglevel", "error",
            "-i", &args.infile,
            "-f", "ffmetadata", 
            "-i", &args.metafile,
            "-map", "0",            // Map all streams from original file
            "-map_metadata", "1",   // Use metadata from the text file
            "-codec", "copy",       // Fast muxing without re-encoding
            "-y",                   // Overwrite output
            &final_output,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("Failed to execute FFmpeg");

    if !status.success() {
        eprintln!("Error: FFmpeg failed to add chapters.");
        std::process::exit(1);
    }
}

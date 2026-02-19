//==================================================================================
// lut-create
// Description: Generate a Hald CLUT and a reference video frame stacked horizontally
// References: [LIB-01], [LIB-03], [LIB-10]
//==================================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path;
use ffmpeg_rust_scripts::{get_media_info, format_time_for_filename};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Generate a Hald CLUT and a reference video frame for color grading",
    after_help = "Example:\n  lut-create -i input.mp4 -s 00:00:30 -o output.png\n\nDependencies:\n  ffmpeg: https://www.ffmpeg.org/\n\nNotes:\n  Defaults to PNG format.\n  If -o is not provided, defaults to: input-lut-[timestamp].png"
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// Input video file
    #[arg(short = 'i', required = true, help = "input video file")]
    infile: String,

    /// Timestamp (HH:MM:SS)
    #[arg(short = 's', required = true, help = "timestamp to extract frame")]
    start: String,

    /// Optional output image file
    #[arg(short = 'o', help = "optional output file")]
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

    // Check if input file exists [cite: 11]
    if !Path::new(&args.infile).exists() {
        eprintln!("Error: Input file '{}' not found.", args.infile);
        std::process::exit(1);
    }

    let info = get_media_info(&args.infile);
    
    // Apply OS-safe filename check to the timestamp 
    let safe_ts = format_time_for_filename(&args.start);

    // Determine output name: input-lut-[timestamp].png [cite: 5, 6]
    let out = args.outfile.clone().unwrap_or_else(|| {
        format!("{}-lut-[{}].png", info.stem, safe_ts)
    });

    // Dynamic Filter: Scale reference frame to 512px height and stack with LUT 
    // The LUT (haldclutsrc=8) is 512x512, so we scale the video to match its height.
    let filter = "[1]scale=-1:512[b];[0][b]hstack";

    // Build FFmpeg command using a Vec 
    let ffmpeg_args = vec![
        "-hide_banner",
        "-v", "error",
        "-stats",
        "-f", "lavfi", "-i", "haldclutsrc=8", // [cite: 4]
        "-ss", &args.start,
        "-i", &args.infile,
        "-frames:v", "1",
        "-filter_complex", filter, // 
        &out,
    ];

    let status = Command::new("ffmpeg")
        .args(&ffmpeg_args)
        .status()
        .expect("Failed to execute FFmpeg");

    if status.success() {
        println!("LUT and reference frame created: {}", out);
    } else {
        std::process::exit(1);
    }
}

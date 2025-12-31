//==============================================================================
// extract-frame
// Description: Extract a single frame with custom scaling and format options
// References: [LIB-01], [LIB-02], [LIB-03]
//==============================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path;
use ffmpeg_scripts_rust::get_media_info; 

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "extract a single frame from a video",
    after_help = "Example:\n  extract-frame -s 00:00:15 -i input.mp4 -x 1280 -f jpg\n\nDependencies:\n  ffmpeg: https://www.ffmpeg.org/\n\nNotes:\n  If width/height is omitted, original size is used.\n  If -o is not provided, defaults to: input-frame-[timestamp].ext",
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// timestamp (HH:MM:SS.mmm)
    #[arg(short = 's', help = "timestamp to extract")]
    start: String,

    /// input video file
    #[arg(short = 'i', help = "input file")]
    infile: String,

    /// output format (png or jpg)
    #[arg(short = 'f', default_value = "png", help = "output format")]
    format: String,

    /// custom width
    #[arg(short = 'x', help = "output width")]
    width: Option<i32>,

    /// custom height
    #[arg(short = 'y', help = "output height")]
    height: Option<i32>,

    /// optional output image file
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

    // [LIB-01] Check if input file exists
    if !Path::new(&args.infile).exists() {
        eprintln!("Error: Input file '{}' not found.", args.infile);
        std::process::exit(1);
    }

    let info = get_media_info(&args.infile);
    let ext = args.format.to_lowercase();
    
    // Updated naming convention: input-[00:00:05].jpg
    let out = args.outfile.clone().unwrap_or_else(|| {
        format!("{}-frame-[{}].{}", info.stem, args.start, ext)
    });

    // Fix for FFmpeg protocol handling of colons in filenames
    let ffmpeg_output_path = format!("./{}", out);

    // Build scaling filter: -2 ensures even dimensions for compatibility
    let scale_filter = match (args.width, args.height) {
        (Some(w), Some(h)) => format!("scale={}:{}", w, h),
        (Some(w), None)    => format!("scale={}:-2", w),
        (None, Some(h))    => format!("scale=-2:{}", h),
        (None, None)       => "".to_string(),
    };

    let mut ffmpeg_args = vec![
        "-hide_banner", "-v", "error", "-stats",
        "-ss", &args.start,
        "-i", &args.infile,
    ];

    if !scale_filter.is_empty() {
        ffmpeg_args.push("-vf");
        ffmpeg_args.push(&scale_filter);
    }

    ffmpeg_args.extend_from_slice(&[
        "-frames:v", "1",
        "-q:v", "2", 
        "-update", "1",
        &ffmpeg_output_path,
    ]);

    let status = Command::new("ffmpeg")
        .args(&ffmpeg_args)
        .status()
        .expect("Failed to execute FFmpeg");

    if status.success() {
        println!("Frame extracted to: {}", out);
    }
}

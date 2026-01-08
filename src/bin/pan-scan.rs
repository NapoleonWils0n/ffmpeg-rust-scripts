//==============================================================================
// pan-scan
// Description: Create a pan animation using scale and crop math from shell script
// References: [LIB-01] Path validation, [LIB-03] get_media_info, 
//             [LIB-04] parse_to_seconds, [LIB-09] format_seconds_ms
//             [LIB-10]
//==============================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path;
use ffmpeg_rust_scripts::{get_media_info, format_seconds_ms, parse_to_seconds, format_time_for_filename};

#[derive(Parser, Debug)]
#[command(
    author, 
    version, 
    about = "Pan scan over an image using scale/crop math",
    after_help = "Example:\n  pan-scan -i photo.jpg -d 00:00:10 -p l\n\nDependencies:\n  ffmpeg: https://www.ffmpeg.org/\n\n",
    override_usage = "pan-scan [OPTIONS] -i <INFILE> -d <DURATION> -p <POSITION>"
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// Input image file
    #[arg(short = 'i', required = true)]
    infile: String,

    /// Duration (e.g., 10 or 00:00:10)
    #[arg(short = 'd', required = true)]
    duration: String,

    /// Position: l (left), r (right), u (up), d (down)
    #[arg(short = 'p', required = true)]
    position: String,

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

fn get_image_dimensions(path: &str) -> (u32, u32) {
    let output = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-select_streams", "v:0",
            "-show_entries", "stream=width,height",
            "-of", "csv=s=x:p=0",
            path,
        ])
        .output()
        .expect("Failed to execute ffprobe");
    
    let res = String::from_utf8_lossy(&output.stdout);
    let dims: Vec<u32> = res.trim().split('x').filter_map(|s| s.parse().ok()).collect();
    if dims.len() == 2 { (dims[0], dims[1]) } else { (1920, 1080) }
}

fn main() {
    let args = Args::parse();

    if !Path::new(&args.infile).exists() {
        eprintln!("Error: Input image '{}' not found.", args.infile);
        std::process::exit(1);
    }

    let (iw, ih) = get_image_dimensions(&args.infile);
    let dur = parse_to_seconds(&args.duration);
    
    let info = get_media_info(&args.infile);
    let full_ts = format_seconds_ms(dur);
    let timestamp_raw = full_ts.split('.').next().unwrap_or("00:00:00");

    // Apply LIB-10 OS check
    let timestamp = format_time_for_filename(timestamp_raw);
    
    let pos_full = match args.position.as_str() {
        "l" => "left", "r" => "right", "u" => "up", "d" => "down",
        _ => &args.position,
    };

    let final_output = args.outfile.unwrap_or_else(|| {
        format!("{}-pan-{}-[{}].mp4", info.stem, pos_full, timestamp)
    });

    let filter = match args.position.as_str() {
        "l" => format!("scale=w=-2:h=3*{},crop=w=3*{}/1.05:h=3*{}/1.05:x=t*(in_w-out_w)/{}:y=(in_h-out_h)/2,scale=w={}:h={},setsar=1", ih, iw, ih, dur, iw, ih),
        "r" => format!("scale=w=-2:h=3*{},crop=w=3*{}/1.05:h=3*{}/1.05:x=(in_w-out_w)-t*(in_w-out_w)/{}:y=(in_h-out_h)/2,scale=w={}:h={},setsar=1", ih, iw, ih, dur, iw, ih),
        "u" => format!("scale=w=-2:h=3*{},crop=w=3*{}/1.2:h=3*{}/1.2:x=(in_w-out_w)/2:y=t*(in_h-out_h)/{},scale=w={}:h={},setsar=1", ih, iw, ih, dur, iw, ih),
        "d" => format!("scale=w=-2:h=3*{},crop=w=3*{}/1.2:h=3*{}/1.2:x=(in_w-out_w)/2:y=(in_h-out_h)-t*(in_h-out_h)/{},scale=w={}:h={},setsar=1", ih, iw, ih, dur, iw, ih),
        _ => {
            eprintln!("Error: Use l, r, u, or d for position.");
            std::process::exit(1);
        }
    };

    let status = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel", "error", // Suppress errors and banner
            "-stats",             // Enable real-time progress
            "-r", "30",
            "-loop", "1",
            "-i", &args.infile,
            "-t", &dur.to_string(),
            "-filter_complex", &filter,
            "-c:v", "libx264",
            "-crf", "18",
            "-profile:v", "high",
            "-pix_fmt", "yuv420p",
            "-movflags", "+faststart",
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

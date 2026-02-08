//==============================================================================
// trim-short
// Description: Create vertical 9:16 clips for YouTube Shorts or TikTok
// References: [LIB-01], [LIB-03], [LIB-04], [LIB-09], [LIB-10]
//==============================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path;
use std::env;
use ffmpeg_rust_scripts::{get_media_info, parse_to_seconds, format_seconds_ms, format_time_for_filename};

#[derive(Parser, Debug)]
#[command(
    author, 
    version, 
    about = "Create a 9:16 vertical clip for YouTube Shorts or TikTok",
    after_help = "Example:\n  trim-short -i input.mp4 -s 00:00:10 -x 75\n\nDependencies:\n  ffmpeg: https://www.ffmpeg.org/",
    override_usage = "trim-short -i <INPUT> -s <START> [OPTIONS]"
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// Input file
    #[arg(short = 'i', required = true)]
    infile: String,

    /// Start time (HH:MM:SS.mmm)
    #[arg(short = 's', required = true)]
    start: String,

    /// End time (optional, defaults to +60s)
    #[arg(short = 't')]
    end: Option<String>,

    /// X-position percentage (0, 25, 50, 75, 100)
    #[arg(short = 'x', default_value = "50")]
    x_pos: String,

    /// Optional output file
    #[arg(short = 'o')]
    outfile: Option<String>,

    /// Print help
    #[arg(short = 'h', long = "help", action = clap::ArgAction::Help)]
    help: Option<bool>,

    /// Print version
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    version: Option<bool>,
}

/// check if the nvenc code is available
fn has_nvenc() -> bool {
    let output = Command::new("ffmpeg").args(["-encoders"]).output().expect("ffmpeg check failed");
    String::from_utf8_lossy(&output.stdout).contains("hevc_nvenc")
}

fn main() {
    let args = Args::parse();

    if !Path::new(&args.infile).exists() {
        eprintln!("Error: Input file '{}' not found.", args.infile);
        std::process::exit(1);
    }

    // 1. TIMING LOGIC
    let start_sec = parse_to_seconds(&args.start);
    let end_sec = match &args.end {
        Some(t) => parse_to_seconds(t),
        None => start_sec + 60.0, // Default to 60s duration for Shorts
    };

    // 2. NAMING LOGIC
    let info = get_media_info(&args.infile);
    
    // 1. Get full timestamps (preserving milliseconds)
    let start_full = format_seconds_ms(start_sec);
    let end_full = format_seconds_ms(end_sec);
    
    // 2. LIB-10: Use the library's built-in OS check
    let start_fs = format_time_for_filename(&start_full);
    let end_fs = format_time_for_filename(&end_full);

    let x_was_specified = env::args().any(|arg| arg == "-x");

    // Unified naming format: [start–end] using en-dash
    let mut name_suffix = format!("-short-[{}–{}]", start_fs, end_fs);
    if args.x_pos != "50" || x_was_specified {
        name_suffix = format!("-x-{}-short-[{}–{}]", args.x_pos, start_fs, end_fs);
    }

    let final_output = args.outfile.unwrap_or_else(|| {
        format!("{}.mp4", info.stem + &name_suffix)
    });

    // 3. CROP LOGIC (Vertical 9:16)
    let x_offset = match args.x_pos.as_str() {
        "0"   => "(in_w-9/16*in_h)*0.0",
        "25"  => "(in_w-9/16*in_h)*0.25",
        "50"  => "(in_w-9/16*in_h)*0.5",
        "75"  => "(in_w-9/16*in_h)*0.75",
        "100" => "(in_w-9/16*in_h)*1.0",
        _     => "(in_w-9/16*in_h)*0.5",
    };

    let filter_string = format!("crop=ih*9/16:ih:{}:0,scale=1080:1920", x_offset);

    // 4. EXECUTE FFMPEG (Output Seeking: -i before -ss and -to)
    let mut cmd = Command::new("ffmpeg");
    cmd.args([
        "-hide_banner", "-v", "error", "-stats",
        "-i", &args.infile,
    ]);

    // Position and Duration
    cmd.args([
        "-ss", &args.start,
        "-to", &format_seconds_ms(end_sec),
    ]);

    // Video Filter (9:16 Crop and Scale)
    cmd.args(["-vf", &filter_string]);

    // Video Encoder Settings
    if has_nvenc() {
        println!("+ Using High-Fidelity Hardware Encoding (NVENC)");
        cmd.args([
            "-c:v", "hevc_nvenc",
            "-tune", "hq",
            "-preset", "p7",
            "-rc", "vbr",
            "-multipass", "fullres",
            "-rc-lookahead", "32",
            "-spatial-aq", "1",
            "-cq", "20",
            "-b:v", "0",
        ]);
    } else {
        println!("+ NVENC not found. Falling back to libx264 (CRF 18)");
        cmd.args(["-c:v", "libx264", "-crf", "18", "-preset", "medium"]);
    }

    // Audio and Final Output Arguments
    cmd.args([
        "-c:a", "aac",
        "-pix_fmt", "yuv420p",
        "-movflags", "+faststart",
        "-y",
        &final_output,
    ]);

    let status = cmd.status().expect("Failed to execute FFmpeg");

    if !status.success() {
        eprintln!("FFmpeg failed to create the vertical clip.");
        std::process::exit(1);
    }
}

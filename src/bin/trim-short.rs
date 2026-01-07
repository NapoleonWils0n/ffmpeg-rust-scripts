//==============================================================================
// trim-short
// Description: Create vertical 9:16 clips for YouTube Shorts or TikTok
// References: [LIB-01], [LIB-03], [LIB-04], [LIB-09], [LIB-10]
//==============================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path;
use std::env;
use ffmpeg_rust_scripts::{get_media_info, parse_to_seconds, format_seconds_ms};

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

    /// X-position percentage (0, 25, 50, 75, 100) [default: 50]
    #[arg(short = 'x', default_value = "50")]
    x_pos: String,

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
        None => start_sec + 60.0,
    };
    
    // 2. CROP & SCALE LOGIC
    let x_percent: f64 = args.x_pos.parse().unwrap_or(50.0);
    let x_offset = format!("(iw-ow)*({}/100)", x_percent);
    let filter = format!("crop=ih*(9/16):ih:{}:0,scale=1080:1920,setsar=1/1", x_offset);

    // 3. NAMING LOGIC
    let info = get_media_info(&args.infile);
    let start_ts_raw = format_seconds_ms(start_sec).split('.').next().unwrap_or("00:00:00").to_string();
    let end_ts_raw = format_seconds_ms(end_sec).split('.').next().unwrap_or("00:00:00").to_string();
    
    // Check if -x was explicitly passed in the arguments
    let x_was_specified = env::args().any(|arg| arg == "-x");

    // Logic: Use start_ts_raw and end_ts_raw directly to keep the colons
    let mut name_suffix = format!("-short-[{}-{}]", start_ts_raw, end_ts_raw);
    if args.x_pos != "50" || x_was_specified {
        name_suffix = format!("-x-{}-short-[{}-{}]", args.x_pos, start_ts_raw, end_ts_raw);
    }

    let final_output = args.outfile.unwrap_or_else(|| {
        format!("./{}{}.mp4", info.stem, name_suffix)
    });

    // 4. EXECUTE FFMPEG
    let status = Command::new("ffmpeg")
        .args([
            "-hide_banner", "-loglevel", "error", "-stats",
            "-ss", &args.start,
            "-to", &end_sec.to_string(),
            "-i", &args.infile,
            "-vf", &filter,
            "-c:v", "libx264", "-crf", "18", "-preset", "veryfast",
            "-pix_fmt", "yuv420p", 
            "-c:a", "aac", "-b:a", "192k",
            "-y", &final_output,
        ])
        .status()
        .expect("Failed to execute FFmpeg");

    if !status.success() {
        std::process::exit(1);
    }
}

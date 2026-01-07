//==============================================================================
// zoompan
// Description: Ken Burns style zoom in/out with real-time progress
// References: [LIB-01], [LIB-03], [LIB-04], [LIB-09], [LIB-10]
//==============================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path;
use ffmpeg_rust_scripts::{get_media_info, format_seconds_ms, parse_to_seconds, format_time_for_filename};

#[derive(Parser, Debug)]
#[command(
    author, 
    version, 
    about = "Ken Burns style zoom animation",
    after_help = "Example:\n  zoompan -i image.jpg -d 10 -z in -p c\n\nDependencies:\n  ffmpeg: https://www.ffmpeg.org/",
    override_usage = "zoompan [OPTIONS] -i <INPUT> -d <DURATION>"
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// Input image (png, jpg, jpeg)
    #[arg(short = 'i', required = true)]
    infile: String,

    /// Duration (e.g., 10 or 00:00:10)
    #[arg(short = 'd', required = true)]
    duration: String,

    /// Zoom direction: in, out
    #[arg(short = 'z', default_value = "in")]
    zoom: String,

    /// Position: tl, tc, tr, c, bl, bc, br
    #[arg(short = 'p', default_value = "c")]
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

fn get_image_height(path: &str) -> u32 {
    let output = Command::new("ffprobe")
        .args(["-v", "error", "-select_streams", "v:0", "-show_entries", "stream=height", "-of", "csv=p=0", path])
        .output().expect("ffprobe failed");
    String::from_utf8_lossy(&output.stdout).trim().parse().unwrap_or(720)
}

fn main() {
    let args = Args::parse();

    if !Path::new(&args.infile).exists() {
        eprintln!("Error: Input file not found.");
        std::process::exit(1);
    }

    let dur = parse_to_seconds(&args.duration);
    let img_h = get_image_height(&args.infile);
    let total_frames = (dur * 30.0) as u32;

    // Zoom Math
    let z_expr = if args.zoom == "in" {
        "min(zoom+0.0015,1.5)"
    } else {
        "if(lte(zoom,1.0),1.5,max(1.001,zoom-0.0015))"
    };

    // Position Math
    let (x, y) = match args.position.as_str() {
        "tl" => ("0", "0"),
        "tc" => ("iw/2-(iw/zoom/2)", "0"),
        "tr" => ("iw-iw/zoom", "0"),
        "c"  => ("iw/2-(iw/zoom/2)", "ih/2-(ih/zoom/2)"),
        "bl" => ("0", "ih-ih/zoom"),
        "bc" => ("iw/2-(iw/zoom/2)", "ih-ih/zoom"),
        "br" => ("iw-iw/zoom", "ih-ih/zoom"),
        _    => ("iw/2-(iw/zoom/2)", "ih/2-(ih/zoom/2)"),
    };

    // 1. Get media info
    let info = get_media_info(&args.infile);

    // 2. Format the duration (strip milliseconds for filename)
    let full_ts = format_seconds_ms(dur);
    let timestamp_raw = full_ts.split('.').next().unwrap_or("00:00:00");
    
    // 3. LIB-10: Convert colons to dashes for Windows compatibility
    let timestamp_fs = format_time_for_filename(timestamp_raw);

    let final_output = args.outfile.unwrap_or_else(|| {
        format!("{}-zoom-{}-{}-[{}].mp4", info.stem, args.zoom, args.position, timestamp_fs)
    });

    // High-res scaling to prevent jitter
    let filter = format!(
        "scale=-2:10*ih,zoompan=z='{}':x='{}':y='{}':d={}:s=1280x720,scale=-2:{}", 
        z_expr, x, y, total_frames, img_h
    );

    

    // Run FFmpeg with progress enabled
    let status = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel", "error", // Suppress errors
            "-stats",             // Force progress stats even with low loglevel
            "-loop", "1",
            "-i", &args.infile,
            "-t", &dur.to_string(),
            "-vf", &filter,
            "-c:v", "libx264",
            "-crf", "18",
            "-pix_fmt", "yuv420p",
            "-y",
            &final_output
        ])
        .status()
        .expect("Failed to execute FFmpeg");

    if !status.success() {
        std::process::exit(1);
    }
}

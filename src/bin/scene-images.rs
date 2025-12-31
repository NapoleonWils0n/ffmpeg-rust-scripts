//==============================================================================
// scene-images
// Description: Create thumbnails from scene detection timestamps
// References: [LIB-01], [LIB-03], [LIB-04], [LIB-09], [LIB-10]
// Dependencies: ffmpeg
//==============================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path;
use std::fs::File;
use std::io::{BufRead, BufReader};
use ffmpeg_scripts_rust::{get_media_info, parse_to_seconds, format_seconds_ms};

#[derive(Parser, Debug)]
#[command(
    author, 
    version, 
    about = "Create thumbnails from scene detection timestamps",
    after_help = "Example:\n  scene-images -i input.mp4 -c cutlist.txt -x 1280 -t jpg\n\nDependencies:\n  ffmpeg: https://www.ffmpeg.org/",
    override_usage = "scene-images -i <INPUT> -c <CUTLIST> [OPTIONS]"
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// Input video file
    #[arg(short = 'i', required = true)]
    input: String,

    /// Cutlist file (comma-separated start,duration)
    #[arg(short = 'c', required = true)]
    cutlist: String,

    /// Image format (png or jpg)
    #[arg(short = 't', default_value = "jpg")]
    format: String,

    /// Width of the output image
    #[arg(short = 'x')]
    width: Option<i32>,

    /// Height of the output image
    #[arg(short = 'y')]
    height: Option<i32>,

    /// Print help
    #[arg(short = 'h', long = "help", action = clap::ArgAction::Help)]
    help: Option<bool>,

    /// Print version
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    version: Option<bool>,
}

fn main() {
    let args = Args::parse();

    if !Path::new(&args.input).exists() {
        eprintln!("Error: Input video '{}' not found.", args.input);
        std::process::exit(1);
    }
    if !Path::new(&args.cutlist).exists() {
        eprintln!("Error: Cutlist file '{}' not found.", args.cutlist);
        std::process::exit(1);
    }

    let info = get_media_info(&args.input);
    let file = File::open(&args.cutlist).expect("Failed to open cutlist");
    let reader = BufReader::new(file);

    // 1. DETERMINE SCALE FILTER AND FILENAME SUFFIX
    let mut suffix = String::new();
    let scale_filter = match (args.width, args.height) {
        (Some(w), Some(h)) => {
            suffix = format!("-x{}-y{}", w, h);
            Some(format!("scale={}:{}", w, h))
        },
        (Some(w), None) => {
            suffix = format!("-x{}", w);
            Some(format!("scale={}:-1", w))
        },
        (None, Some(h)) => {
            suffix = format!("-y{}", h);
            Some(format!("scale=-1:{}", h))
        },
        (None, None) => None,
    };

    // 2. PROCESS CUTLIST
    for (index, line) in reader.lines().enumerate() {
        if let Ok(l) = line {
            let start_raw = l.split(',').next().unwrap_or("00:00:00").trim();
            let start_sec = parse_to_seconds(start_raw);
            
            let time_fs = format_seconds_ms(start_sec)
                .split('.')
                .next()
                .unwrap_or("00:00:00")
                .to_string();

            // Inject the width/height suffix into the name
            let output_name = format!("{}-scene-{:03}{}-[{}].{}", 
                info.stem, index + 1, suffix, time_fs, args.format);

            println!("Extracting frame {}: {}", index + 1, start_raw);

            // 3. EXECUTE FFMPEG (Optimized Seek)
            let mut cmd = Command::new("ffmpeg");
            cmd.args([
                "-hide_banner", "-loglevel", "error",
                "-ss", start_raw,
                "-i", &args.input,
                "-vframes", "1",
                "-q:v", "2",
            ]);

            if let Some(ref filter) = scale_filter {
                cmd.args(["-vf", filter]);
            }

            cmd.arg(&output_name);

            let status = cmd.status().expect("Failed to execute FFmpeg");

            if !status.success() {
                eprintln!("Error: FFmpeg failed on image {}", index + 1);
            }
        }
    }
}

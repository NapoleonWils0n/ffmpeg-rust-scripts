//==============================================================================
// tile-thumbnails
// Description: Create tiled thumbnails covering the video duration
// References: [LIB-01], [LIB-03], [LIB-04], [LIB-08]
//==============================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path;
use chrono::Local;
use ffmpeg_scripts_rust::{get_media_info, parse_to_seconds, get_video_duration}; 

#[derive(Parser, Debug)]
#[command(
    author, version,
    about = "create an image with thumbnails from a video",
    after_help = "Example:\n  tile-thumbnails -i input.mp4 -s 00:00:00.000 -w 160 -t 4x3 -j jpg\n\nDependencies:\n  ffmpeg: https://www.ffmpeg.org/\n\nNotes:\n  -x on enables timestamps. -j sets image format (png/jpg).",
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    #[arg(short = 'i')]
    infile: String,

    #[arg(short = 's', default_value = "00:00:05")]
    seek: String,

    #[arg(short = 'w', default_value = "160")]
    width: String,

    #[arg(short = 't', default_value = "4x3")]
    layout: String,

    #[arg(short = 'p', default_value = "7")]
    padding: String,

    #[arg(short = 'm', default_value = "2")]
    margin: String,

    #[arg(short = 'c', default_value = "black")]
    color: String,

    #[arg(short = 'f', default_value = "white")]
    fontcolor: String,

    #[arg(short = 'b', default_value = "black")]
    boxcolor: String,

    #[arg(short = 'x', default_value = "off")]
    timestamps: String,

    /// image format (png or jpg)
    #[arg(short = 'j', default_value = "png")]
    format: String,

    #[arg(short = 'o')]
    outfile: Option<String>,

    #[arg(short = 'h', action = clap::ArgAction::Help)]
    help: Option<bool>,

    #[arg(short = 'v', action = clap::ArgAction::Version)]
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
    
    // Normalize format and generate unique filename
    let ext = args.format.to_lowercase();
    let now = Local::now().format("%Y-%m-%d-%H-%M-%S");
    let out = args.outfile.clone().unwrap_or_else(|| {
        format!("{}-tile-{}.{}", info.stem, now, ext)
    });
    
    let seek_seconds = parse_to_seconds(&args.seek);
    let duration = get_video_duration(&args.infile);
    let remaining_duration = duration - seek_seconds;

    let parts: Vec<&str> = args.layout.split('x').collect();
    let cols: f64 = parts[0].parse().unwrap_or(4.0);
    let rows: f64 = parts.get(1).map_or(3.0, |v| v.parse().unwrap_or(3.0));
    let total_tiles = cols * rows;

    let fps_val = total_tiles / remaining_duration;
    let w_val = if args.width == "on" { "iw".to_string() } else { args.width };

    // Set sampling rate and scaling
    let mut vf = format!("fps={},scale={}:-2", fps_val, w_val);
    
    if args.timestamps.to_lowercase() == "on" {
        // x=(w-tw)/2 centers the text horizontally
        // y=h-th-10 places the text at the bottom with a 10px margin
        // fontsize=h/10 ensures text size scales with thumbnail height
        vf.push_str(&format!(
            ",drawtext=text='%{{pts\\:hms}}':x=(w-tw)/2:y=h-th-10:fontsize=h/10:fontcolor={}:box=1:boxcolor={}@0.5",
            args.fontcolor, args.boxcolor
        ));
    }
    
    vf.push_str(&format!(
        ",tile={}:padding={}:margin={}:color={}",
        args.layout, args.padding, args.margin, args.color
    ));

    // Execute FFmpeg with high quality for jpg outputs
    let status = Command::new("ffmpeg")
        .args([
            "-hide_banner", "-v", "error", "-stats",
            "-ss", &seek_seconds.to_string(),
            "-i", &args.infile,
            "-vf", &vf,
            "-frames:v", "1",
            "-q:v", "2", 
            "-update", "1",
            &format!("./{}", out),
        ])
        .status()
        .expect("Failed to execute FFmpeg");

    if status.success() {
        println!("Tile thumbnails created: {}", out);
    }
}

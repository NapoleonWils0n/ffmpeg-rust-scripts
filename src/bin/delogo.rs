//==============================================================================
// delogo
// Description: remove a logo from video footage
// References: [LIB-01] [LIB-03] 
//==============================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path;
use ffmpeg_rust_scripts::{get_media_info};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "remove a logo from video footage",
    after_help = "Example:\n  delogo -i input.mp4 -x 590 -y 670 -w 120 -h 49 -p 1\n\nDependencies: ffmpeg, ffplay\n\nNotes:\n The -p 1 option previews with a green box. -p 0 previews without the box. Omit -p to record.",
    override_usage = "delogo [OPTIONS] -i <INFILE> -x <X> -y <Y> -w <W> -h <H>"
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    #[arg(short = 'i', help = "input file", required = true)]
    infile: String,

    /// x coordinate
    #[arg(short = 'x', help = "x coordinate", required = true)]
    x: u32,

    /// y coordinate
    #[arg(short = 'y', help = "y coordinate", required = true)]
    y: u32,

    /// filter width
    #[arg(short = 'w', help = "filter width", required = true)]
    width: u32,

    /// filter height
    #[arg(short = 'h', help = "filter height", required = true)]
    height: u32,

    /// preview mode: 1=box, 0=no box
    #[arg(short = 'p', help = "preview mode: 1=box, 0=no box")]
    preview: Option<u32>,

    #[arg(short = 'o', help = "optional output file")]
    outfile: Option<String>,

    #[arg(short = 'H', long = "help", help = "Print help", action = clap::ArgAction::Help)]
    help: Option<bool>,

    #[arg(short = 'v', long = "version", help = "Print version", action = clap::ArgAction::Version)]
    version: Option<bool>,
}

fn main() {
    let args = Args::parse();

    if !Path::new(&args.infile).exists() {
        eprintln!("! Error: Input file '{}' does not exist.", args.infile);
        std::process::exit(1);
    }

    // 1. Get Video Dimensions using ffprobe
    let output = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-select_streams", "v:0",
            "-show_entries", "stream=width,height",
            "-of", "csv=s=x:p=0",
            &args.infile,
        ])
        .output()
        .expect("Failed to execute ffprobe");

    let dims = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = dims.trim().split('x').collect();
    
    if parts.len() == 2 {
        let video_w: u32 = parts[0].parse().unwrap_or(0);
        let video_h: u32 = parts[1].parse().unwrap_or(0);

        // Validation: delogo requires 1 pixel of space around the box
        if args.x + args.width >= video_w || args.y + args.height >= video_h {
            eprintln!("! Error: Delogo area is out of bounds or too close to edge.");
            eprintln!("  Video: {}x{}, Delogo ends at: {}x{}", video_w, video_h, args.x + args.width, args.y + args.height);
            std::process::exit(1);
        }
    }

    // [LIB-03] Get file info for naming
    let info = get_media_info(&args.infile);
    let out_path = args.outfile.unwrap_or_else(|| {
        format!("{}-delogo.{}", info.stem, info.extension)
    });

    // Construct the filter string
    // show=1 draws the green box, show=0 does not
    let filter = format!("delogo=x={}:y={}:w={}:h={}:show={}", 
        args.x, args.y, args.width, args.height, args.preview.unwrap_or(0));

    if args.preview.is_some() {
        // PREVIEW MODE (ffplay)
        Command::new("ffplay")
            .args(["-hide_banner", "-stats", "-v", "error", "-i", &args.infile, "-vf", &filter])
            .status()
            .expect("Failed to execute ffplay");
    } else {
        // RECORD MODE (ffmpeg)
        let status = Command::new("ffmpeg")
            .args([
                "-hide_banner", "-stats", "-v", "error",
                "-i", &args.infile,
                "-vf", &filter,
                "-c:a", "copy", // preserve audio
                &out_path,
            ])
            .status()
            .expect("Failed to execute ffmpeg");

        if !status.success() {
            std::process::exit(1);
        }
    }
}

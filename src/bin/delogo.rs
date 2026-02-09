//==============================================================================
// delogo
// Description: remove a logo from video footage
// References: [LIB-01] [LIB-03], [LIB-11]
//==============================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path;
use ffmpeg_rust_scripts::{get_media_info, hardware_encoding};

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

    /// optional argument: output file
    #[arg(short = 'o', help = "optional output file")]
    outfile: Option<String>,

    /// Print help
    #[arg(short = 'H', long = "help", help = "Print help", action = clap::ArgAction::Help)]
    help: Option<bool>,

    /// Print version
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

    // Determine Encoder once at the start of main
    let (v_codec, v_params) = if hardware_encoding() {
        println!("+ using hardware acceleration.");
        (
            "hevc_nvenc",
            vec![
                "-tune", "hq",
                "-preset", "p7",
                "-rc", "vbr",
                "-multipass", "fullres",
                "-rc-lookahead", "32",
                "-spatial-aq", "1",
                "-cq", "20",
                "-b:v", "0",
            ],
        )
    } else {
        println!("+ using software encoding.");
        (
            "libx264",
            vec![
                "-crf", "18",
                "-preset", "medium",
            ],
        )
    };

    // 4. Execution Logic
    if args.preview.is_some() {
        // PREVIEW MODE (ffplay)
        let mut cmd = Command::new("ffplay");
        cmd.args([
            "-hide_banner",
            "-stats",
            "-v", "error",
            "-i", &args.infile,
            "-vf", &filter,
        ]);
        cmd.status().expect("failed to execute ffplay");
    } else {
        // RECORD MODE (ffmpeg)
        let mut cmd = Command::new("ffmpeg");
        
        // Construct the unified argument vector
        let mut ffmpeg_args = vec![
            "-hide_banner",
            "-v", "error",
            "-stats",
            "-i", &args.infile,
            "-vf", &filter,
            "-c:v", v_codec,
        ];

        // Append encoder-specific parameters
        ffmpeg_args.extend(v_params);

        // Finalize with audio and output
        ffmpeg_args.extend(vec![
            "-c:a", "copy",           // Preserve original audio
            "-pix_fmt", "yuv420p",
            "-movflags", "+faststart",
            &out_path,                // Non-destructive (no -y)
        ]);

        let status = cmd.args(ffmpeg_args)
            .status()
            .expect("failed to execute ffmpeg");

        if !status.success() {
            eprintln!("! error: ffmpeg failed to process delogo.");
            std::process::exit(1);
        }
    }
}

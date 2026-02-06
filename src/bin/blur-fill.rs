//==============================================================================
// blur-fill
// Description: Fill pillarboxes/letterboxes with a blurred version of the video
// References: [LIB-01] [LIB-03] 
//==============================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path;
use ffmpeg_rust_scripts::{get_media_info};

#[derive(Parser, Debug)]
#[command(
    author, version,
    about = "Fill pillarboxes with a blurred version of the input video",
    after_help = "Example:\n  blur-fill -i vertical.mp4 -b 50 -o output.mp4\n\nNotes:\n - Targets 1920x1080 (16:9).\n - Uses NVENC p7 (Highest Quality) or libx264 CRF 18.",
    override_usage = "blur-fill [OPTIONS] -i <INFILE>"
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// input file
    #[arg(short = 'i', help = "input file", required = true)]
    infile: String,

    /// blur strength (default: 40)
    #[arg(short = 'b', help = "blur strength", default_value = "40")]
    blur: u32,

    /// optional output file
    #[arg(short = 'o', help = "optional output file")]
    outfile: Option<String>,

    /// Print help
    #[arg(short = 'h', long = "help", help = "Print help", action = clap::ArgAction::Help)]
    help: Option<bool>,

    /// Print version
    #[arg(short = 'v', long = "version", help = "Print version", action = clap::ArgAction::Version)]
    version: Option<bool>,
}

fn has_nvenc() -> bool {
    let output = Command::new("ffmpeg").args(["-encoders"]).output().expect("ffmpeg check failed");
    String::from_utf8_lossy(&output.stdout).contains("hevc_nvenc")
}

fn main() {
    let args = Args::parse();

    if !Path::new(&args.infile).exists() {
        eprintln!("! Error: Input file '{}' does not exist.", args.infile);
        std::process::exit(1);
    }

    let info = get_media_info(&args.infile);
    let out_path = args.outfile.unwrap_or_else(|| format!("{}-blurfill.mp4", info.stem));

    // Filter Logic:
    // Scale background to 'increase' (fill 1080p area), crop excess, blur.
    // Scale foreground to match 1080p height.
    let filter = format!(
        "[0:v]split=2[main][bg]; \
         [bg]scale=1920:1080:force_original_aspect_ratio=increase,crop=1920:1080,boxblur={}:10[blurred]; \
         [main]scale=-1:1080[foreground]; \
         [blurred][foreground]overlay=(W-w)/2:(H-h)/2",
        args.blur
    );

    let mut cmd = Command::new("ffmpeg");
    cmd.args(["-hide_banner", "-stats", "-v", "error"]);
    cmd.args(["-i", &args.infile]);
    cmd.args(["-filter_complex", &filter]);

    if has_nvenc() {
        println!("+ Using Hardware Encoding (NVENC p7)");
        // Using p7 for highest quality as suggested
        cmd.args(["-c:v", "hevc_nvenc", "-cq", "20", "-preset", "p7"]);
    } else {
        println!("+ NVENC not found. Falling back to libx264 (CRF 18)");
        // CRF 18 for visually lossless fallback
        cmd.args(["-c:v", "libx264", "-crf", "18", "-preset", "medium"]);
    }

    cmd.args([
        "-map", "0:a?", 
        "-c:a", "aac",
        "-pix_fmt", "yuv420p",
        "-movflags", "+faststart",
        &out_path
    ]);

    let status = cmd.status().expect("ffmpeg failed");

    if !status.success() {
        std::process::exit(1);
    }

    println!("+ Done! Saved to: {}", out_path);
}

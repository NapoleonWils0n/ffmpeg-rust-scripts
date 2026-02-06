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
    after_help = "Example:\n  blur-fill -i vertical.mp4 -b 20 -o output.mp4\n\nNotes:\n - Targets 1920x1080 (16:9).\n - Uses High-Quality NVENC VBR or libx264 CRF 18.\n - Smart Audio: Copies AAC, transcodes others to AAC.",
    override_usage = "blur-fill [OPTIONS] -i <INFILE>"
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// input file
    #[arg(short = 'i', help = "input file", required = true)]
    infile: String,

    /// blur strength (default: 20)
    #[arg(short = 'b', help = "blur strength", default_value = "10")]
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

fn get_audio_codec(path: &str) -> String {
    let output = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-select_streams", "a:0",
            "-show_entries", "stream=codec_name",
            "-of", "default=noprint_wrappers=1:nokey=1",
            path
        ])
        .output()
        .expect("ffprobe audio check failed");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn main() {
    let args = Args::parse();

    if !Path::new(&args.infile).exists() {
        eprintln!("! Error: Input file '{}' does not exist.", args.infile);
        std::process::exit(1);
    }

    let info = get_media_info(&args.infile);
    let out_path = args.outfile.unwrap_or_else(|| format!("{}-blurfill.mp4", info.stem));

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

    // 1. Video Encoder Settings
    if has_nvenc() {
        println!("+ Using High-Quality Hardware Encoding (NVENC)");
        cmd.args([
            "-c:v", "hevc_nvenc",
            "-tune", "hq",
            "-preset", "p7",
            "-rc", "vbr",
            "-multipass", "fullres",
            "-cq", "20",
            "-b:v", "0",
            "-rc-lookahead", "32",
            "-spatial-aq", "1"
        ]);
    } else {
        println!("+ NVENC not found. Falling back to libx264 (CRF 18)");
        cmd.args(["-c:v", "libx264", "-crf", "18", "-preset", "medium"]);
    }

    // 2. Smart Audio Settings
    let codec = get_audio_codec(&args.infile);
    if codec == "aac" {
        println!("+ Audio is AAC: Using stream copy");
        cmd.args(["-c:a", "copy"]);
    } else {
        println!("+ Audio is {}: Transcoding to AAC", codec);
        cmd.args(["-c:a", "aac"]);
    }

    // Final mapping and output
    cmd.args([
        "-pix_fmt", "yuv420p",
        "-movflags", "+faststart",
        &out_path
    ]);

    let status = cmd.status().expect("ffmpeg failed");

    if !status.success() {
        std::process::exit(1);
    }
}

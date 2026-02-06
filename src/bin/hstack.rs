//==============================================================================
// hstack
// Description: stack two videos side-by-side with auto-scaling and nvenc fallback
// References: [LIB-01] [LIB-03] 
//==============================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path;
use ffmpeg_rust_scripts::{get_media_info};

#[derive(Parser, Debug)]
#[command(
    author, version,
    about = "Stack two videos side-by-side (hstack)",
    after_help = "Example:\n  hstack -l left.mp4 -r right.mp4 -a r -o comparison.mp4\n\nNotes:\n - Auto-scales to match heights (max 1080p).\n - Defaults to NVENC, falls back to libx264.",
    override_usage = "hstack [OPTIONS] -l <LEFT> -r <RIGHT>"
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// Left video input
    #[arg(short = 'l', long = "left", required = true)]
    left: String,

    /// Right video input
    #[arg(short = 'r', long = "right", required = true)]
    right: String,

    /// Audio source: l (left) or r (right)
    #[arg(short = 'a', long = "audio", default_value = "l")]
    audio: String,

    #[arg(short = 'o', long = "outfile")]
    outfile: Option<String>,

    #[arg(short = 'h', long = "help", action = clap::ArgAction::Help)]
    help: Option<bool>,

    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    version: Option<bool>,
}

fn get_dims(path: &str) -> (u32, u32) {
    let output = Command::new("ffprobe")
        .args(["-v", "error", "-select_streams", "v:0", "-show_entries", "stream=width,height", "-of", "csv=s=x:p=0", path])
        .output().expect("ffprobe failed");
    let s = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<u32> = s.trim().split('x').filter_map(|p| p.parse().ok()).collect();
    if parts.len() == 2 { (parts[0], parts[1]) } else { (0, 0) }
}

fn has_nvenc() -> bool {
    let output = Command::new("ffmpeg").args(["-encoders"]).output().expect("ffmpeg check failed");
    String::from_utf8_lossy(&output.stdout).contains("hevc_nvenc")
}

fn main() {
    let args = Args::parse();

    if !Path::new(&args.left).exists() || !Path::new(&args.right).exists() {
        eprintln!("! Error: One or both input files do not exist.");
        std::process::exit(1);
    }

    let (_w1, h1) = get_dims(&args.left);
    let (_w2, h2) = get_dims(&args.right);

    let mut target_h = h1.max(h2);
    if target_h > 1080 { target_h = 1080; }

    // Updated Filter: Added shortest=1 directly into hstack
    let filter = format!(
        "[0:v]scale=-1:{}:flags=lanczos[l]; [1:v]scale=-1:{}:flags=lanczos[r]; [l][r]hstack=inputs=2:shortest=1[v]",
        target_h, target_h
    );

    let audio_map = if args.audio.to_lowercase() == "r" { "1:a" } else { "0:a" };
    let info = get_media_info(&args.left);
    let out_path = args.outfile.unwrap_or_else(|| format!("{}-hstack.mp4", info.stem));

    let mut cmd = Command::new("ffmpeg");
    cmd.args(["-hide_banner", "-stats", "-v", "error"]);
    cmd.args(["-i", &args.left]);
    cmd.args(["-i", &args.right]);
    cmd.args(["-filter_complex", &filter]);
    cmd.args(["-map", "[v]", "-map", audio_map]);

    if has_nvenc() {
        println!("+ Using Hardware Encoding (NVENC)");
        cmd.args(["-c:v", "hevc_nvenc", "-cq", "20", "-preset", "p4"]);
    } else {
        println!("+ NVENC not found. Falling back to Software Encoding (libx264)");
        cmd.args(["-c:v", "libx264", "-crf", "16", "-preset", "medium"]);
    }

    cmd.args(["-c:a", "aac", "-pix_fmt", "yuv420p", "-movflags", "+faststart", &out_path]);

    let status = cmd.status().expect("ffmpeg failed");

    if !status.success() { std::process::exit(1); }
}

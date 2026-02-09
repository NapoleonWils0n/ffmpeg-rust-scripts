//==============================================================================
// hstack
// Description: stack two videos side-by-side with auto-scaling and nvenc fallback
// References: [LIB-01] [LIB-03], [LIB-11]
//==============================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path;
use ffmpeg_rust_scripts::{get_media_info, hardware_encoding};

#[derive(Parser, Debug)]
#[command(
    author, version,
    about = "Stack two videos side-by-side (hstack)",
    after_help = "Example:\n  hstack -l left.mp4 -r right.mp4 -a r -o comparison.mp4\n\nNotes:\n - Auto-scales to match heights (max 1080p).\n - Uses High-Quality NVENC VBR or libx264 CRF 16.\n - Audio: Transcoded to AAC to ensure duration sync with shortest video.",
    override_usage = "hstack [OPTIONS] -l <LEFT> -r <RIGHT>"
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// Left video input
    #[arg(short = 'l', help = "left video input", required = true)]
    left: String,

    /// Right video input
    #[arg(short = 'r', help = "right video input", required = true)]
    right: String,

    /// Audio source: l (left) or r (right)
    #[arg(short = 'a', help = "audio source: l (left) or r (right)", default_value = "l")]
    audio: String,

    /// optional argument: output file
    #[arg(short = 'o', help = "optional output file")]
    outfile: Option<String>,

    /// Print help
    #[arg(short = 'h', long = "help", action = clap::ArgAction::Help)]
    help: Option<bool>,

    /// Print version
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

fn main() {
    let args = Args::parse();

    // 1. Validate inputs
    if !Path::new(&args.left).exists() || !Path::new(&args.right).exists() {
        eprintln!("! Error: One or both input files do not exist.");
        std::process::exit(1);
    }

    // 2. Determine Logic and Naming
    let (_w1, h1) = get_dims(&args.left);
    let (_w2, h2) = get_dims(&args.right);

    let mut target_h = h1.max(h2);
    if target_h > 1080 { target_h = 1080; }

    // Filter: Scale both to target_h while maintaining aspect ratio, then stack
    let filter = format!(
        "[0:v]scale=-1:{}:flags=lanczos[l]; [1:v]scale=-1:{}:flags=lanczos[r]; [l][r]hstack=inputs=2:shortest=1[v]",
        target_h, target_h
    );

    // 3. Naming and Audio Mapping
    let info = get_media_info(&args.left);
    let out_path = args.outfile.unwrap_or_else(|| format!("{}-hstack.mp4", info.stem));
    let audio_map = if args.audio.to_lowercase() == "r" { "1:a" } else { "0:a" };

// 4. Encoder Selection Logic
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
                "-crf", "16",
                "-preset", "medium",
            ],
        )
    };

    // 5. EXECUTE FFMPEG (Unified Vec)
    let mut cmd = Command::new("ffmpeg");
    
    let mut ffmpeg_args = vec![
        "-hide_banner",
        "-v", "error",
        "-stats",
        "-i", &args.left,
        "-i", &args.right,
        "-filter_complex", &filter,
        "-map", "[v]",
        "-map", audio_map,
        "-c:v", v_codec,
    ];

    ffmpeg_args.extend(v_params);

    ffmpeg_args.extend(vec![
        "-c:a", "aac",
        "-pix_fmt", "yuv420p",
        "-shortest",
        "-movflags", "+faststart",
        &out_path,
    ]);

    let status = cmd.args(ffmpeg_args)
        .status()
        .expect("failed to execute ffmpeg");

    if !status.success() {
        eprintln!("! error: ffmpeg failed to process hstack.");
        std::process::exit(1);
    }
}

//==============================================================================
// overlay-clip
// Description: Overlay a video onto a background clip at a specific time
// References: [LIB-01], [LIB-03], [LIB-04], [LIB-09], [LIB-10]
//==============================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path;
use ffmpeg_rust_scripts::{get_media_info, parse_to_seconds, format_seconds_ms, format_time_for_filename};

#[derive(Parser, Debug)]
#[command(
    author, 
    version, 
    about = "Overlay one video clip on top of another video clip",
    after_help = "Example:\n  overlay-clip -a bottom-video.mp4 -b overlay.mp4 -p 00:00:05\n\nDependencies:\n  ffmpeg: https://www.ffmpeg.org/",
    override_usage = "overlay-clip -a <INPUT> -b <OVERLAY> -p <POSITION> [OPTIONS]"
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// Bottom video (-a)
    #[arg(short = 'a', required = true)]
    input: String,

    /// Overlay video (-b)
    #[arg(short = 'b', required = true)]
    overlay: String,

    /// Time to start the overlay (e.g., 5 or 00:00:05)
    #[arg(short = 'p', required = true)]
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

/// check if the nvenc code is available
fn has_nvenc() -> bool {
    let output = Command::new("ffmpeg").args(["-encoders"]).output().expect("ffmpeg check failed");
    String::from_utf8_lossy(&output.stdout).contains("hevc_nvenc")
}

fn main() {
    let args = Args::parse();

    if !Path::new(&args.input).exists() || !Path::new(&args.overlay).exists() {
        eprintln!("Error: One or more input files not found.");
        std::process::exit(1);
    }

    let start_secs = parse_to_seconds(&args.position);
    let info = get_media_info(&args.input);
    let fg_info = get_media_info(&args.overlay);
    
    // Format the position for the filename (e.g., 00:00:10)
    let full_ts = format_seconds_ms(start_secs);

    // Apply LIB-10 OS check
    let timestamp = format_time_for_filename(&full_ts);
    
    let final_output = args.outfile.unwrap_or_else(|| {
        format!("{}-overlay-{}-[{}].mp4", info.stem, fg_info.stem, timestamp)
    });

    // Filter: setpts delays the foreground, overlay=eof_action=pass keeps bg after fg ends
    let filter = format!("[1:v]setpts=PTS+{}/TB[fg]; [0:v][fg]overlay=eof_action=pass", start_secs);

    let mut cmd = Command::new("ffmpeg");
    cmd.args([
        "-hide_banner",
        "-loglevel", "error",
        "-stats",
        "-i", &args.input,
        "-i", &args.overlay,
        "-filter_complex", &filter,
    ]);

    // Video Encoder Settings (NVENC with x264 fallback)
    if has_nvenc() {
        println!("+ Using High-Fidelity Hardware Encoding (NVENC)");
        cmd.args([
            "-c:v", "hevc_nvenc",
            "-tune", "hq",
            "-preset", "p7",
            "-rc", "vbr",
            "-multipass", "fullres",
            "-rc-lookahead", "32",
            "-spatial-aq", "1",
            "-cq", "20",
            "-b:v", "0",
        ]);
    } else {
        println!("+ NVENC not found. Falling back to libx264 (CRF 18)");
        cmd.args(["-c:v", "libx264", "-crf", "18"]);
    }

    // Final Output Arguments
    cmd.args([
        "-pix_fmt", "yuv420p",
        "-movflags", "+faststart",
        "-y",
        &final_output,
    ]);

    let status = cmd.status().expect("Failed to execute FFmpeg");

    if !status.success() {
        eprintln!("Error: FFmpeg failed to overlay clips.");
        std::process::exit(1);
    }
}

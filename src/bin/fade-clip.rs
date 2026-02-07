//==============================================================================
// fade-in
// Description: Apply a fade-in effect to both video and audio with timestamped output
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
    about = "Fade in a video and audio clip",
    after_help = "Example:\n  fade-clip -i input.mp4 -d 00:00:02\n\nDependencies:\n  ffmpeg: https://www.ffmpeg.org/",
    override_usage = "fade-clip [OPTIONS] -i <INFILE>"
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// Input video file
    #[arg(short = 'i', required = true)]
    infile: String,

    /// Fade duration (e.g., 2 or 00:00:02)
    #[arg(short = 'd', default_value = "00:00:00.500")]
    duration: String,

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

    if !Path::new(&args.infile).exists() {
        eprintln!("Error: Input file '{}' not found.", args.infile);
        std::process::exit(1);
    }

    let dur_secs = parse_to_seconds(&args.duration);
    let info = get_media_info(&args.infile);
    
    // Format the duration for the filename (e.g., 00:00:02)
    let full_ts = format_seconds_ms(dur_secs);

    // This single line replaces all the old raw/split logic
    let timestamp = format_time_for_filename(&full_ts);

    let final_output = args.outfile.unwrap_or_else(|| {
        format!("{}-faded-in-[{}].mp4", info.stem, timestamp)
    });

    // Filters
    let v_filter = format!("fade=t=in:st=0:d={}", dur_secs);
    let a_filter = format!("afade=t=in:st=0:d={}", dur_secs);

    // EXECUTE FFMPEG
    let mut cmd = Command::new("ffmpeg");
    cmd.args([
        "-hide_banner",
        "-loglevel", "error",
        "-stats",
        "-i", &args.infile,
    ]);

    // Apply Video and Audio filters
    cmd.args(["-vf", &v_filter]);
    cmd.args(["-af", &a_filter]);

    // Video Encoder Settings
    if has_nvenc() {
        println!("+ Using High-Fidelity Hardware Encoding (NVENC)");
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

    // Audio and Final Output Arguments
    cmd.args([
        "-c:a", "aac",
        "-pix_fmt", "yuv420p",
        "-movflags", "+faststart",
        "-y",
        &final_output,
    ]);

    // Capture the status here so the success check below still works
    let status = cmd.status().expect("Failed to execute FFmpeg");

    if !status.success() {
        eprintln!("Error: FFmpeg execution failed.");
        std::process::exit(1);
    }

}

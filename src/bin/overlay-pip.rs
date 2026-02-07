//==============================================================================
// overlay-pip
// Description: Advanced PiP with position, margin, border, and fade options
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
    about = "Create a Picture-in-Picture (PiP) overlay",
    after_help = "Example:\n  overlay-pip -a background.mp4 -b pip.mp4 -p 00:00:05 -x br -m 30 -k 4 -c white\n\nDependencies:\n  ffmpeg: https://www.ffmpeg.org/",
    override_usage = "overlay-pip -a <INPUT> -b <PIP_VIDEO> -p <POSITION> [OPTIONS]"
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// Bottom video (-a)
    #[arg(short = 'a', required = true)]
    input: String,

    /// Overlay video (-b)
    #[arg(short = 'b', required = true)]
    overlay: String,

    /// Time to start the overlay
    #[arg(short = 'p', required = true)]
    position: String,

    /// Margin [default: 20]
    #[arg(short = 'm')]
    margin: Option<String>,

    /// PiP position (tl, tr, bl, br) [default: tr]
    #[arg(short = 'x')]
    pip_pos: Option<String>,

    /// Width (defaults to 1/4 of video size)
    #[arg(short = 'w')]
    width: Option<String>,

    /// Fade duration [default: 0.2]
    #[arg(short = 'f')]
    fade: Option<String>,

    /// Border size (4 or 0) [default: 4]
    #[arg(short = 'k', value_parser = ["0", "4"])]
    border: Option<String>,

    /// Border color [default: #2f2f2f]
    #[arg(short = 'c')]
    color: Option<String>,

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
        eprintln!("Error: Input files not found.");
        std::process::exit(1);
    }

    // 1. SET DEFAULTS
    let margin_str = args.margin.as_deref().unwrap_or("20");
    let margin: i32 = margin_str.parse().unwrap_or(20);
    let pip_pos = args.pip_pos.as_deref().unwrap_or("tr");
    let fade = args.fade.as_deref().unwrap_or("0.2");
    let border_str = args.border.as_deref().unwrap_or("4");
    let border: i32 = border_str.parse().unwrap_or(4);
    let color = args.color.as_deref().unwrap_or("#2f2f2f");
    
    // Offset is half the border to center the pad (e.g., 4 / 2 = 2)
    let offset = border / 2;

    let start_secs = parse_to_seconds(&args.position);
    let info = get_media_info(&args.input);
    let fg_info = get_media_info(&args.overlay);

    let full_ts = format_seconds_ms(start_secs);

    // Apply LIB-10 OS check
    let timestamp = format_time_for_filename(&full_ts);
    
    // 2. FILENAME LOGIC (Updated to use p- instead of pos-)
    let mut name_parts = format!("{}-pip-{}-p-[{}]", info.stem, fg_info.stem, timestamp);
    if let Some(ref m) = args.margin { name_parts.push_str(&format!("-m-{}", m)); }
    if let Some(ref x) = args.pip_pos { name_parts.push_str(&format!("-x-{}", x)); }
    if let Some(ref w) = args.width { name_parts.push_str(&format!("-w-{}", w)); }
    if let Some(ref f) = args.fade { name_parts.push_str(&format!("-f-[{}]", f)); }
    if let Some(ref k) = args.border { name_parts.push_str(&format!("-k-{}", k)); }
    if let Some(ref c) = args.color { name_parts.push_str(&format!("-c-{}", c)); }
    
    let final_output = args.outfile.unwrap_or_else(|| format!("./{}.mp4", name_parts));

    // 3. SCALE & POSITION
    let scale_val = args.width.as_deref().unwrap_or("iw/4");

    // Coordinate adjustments ensure the visible border starts at the margin
    let x_coord = match pip_pos {
        "tl" | "bl" => format!("{}-{}", margin, offset),
        _ => format!("main_w-overlay_w-{}+{}", margin, offset), // tr, br
    };

    let y_coord = match pip_pos {
        "tl" | "tr" => format!("{}-{}", margin, offset),
        _ => format!("main_h-overlay_h-{}+{}", margin, offset), // bl, br
    };

    // 4. FILTER COMPLEX
    let filter = format!(
        "[1:v]scale={}:-1,pad=w={}+iw:h={}+ih:x={}:y={}:color={}[pip_b]; \
         [pip_b]fade=t=in:st=0:d={},fade=t=out:st=999:d={}[pip_f]; \
         [pip_f]setpts=PTS+{}/TB[fg]; \
         [0:v][fg]overlay={}:{}:eof_action=pass",
        scale_val, border, border, offset, offset, color, fade, fade, start_secs, x_coord, y_coord
    );

    // 5. Build FFmpeg Command
    let mut cmd = Command::new("ffmpeg");
    cmd.args([
        "-hide_banner", "-loglevel", "error", "-stats",
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
            "-cq", "20",
            "-b:v", "0",
            "-rc-lookahead", "32",
            "-spatial-aq", "1"
        ]);
    } else {
        println!("+ NVENC not found. Falling back to libx264 (CRF 18)");
        cmd.args(["-c:v", "libx264", "-crf", "18"]);
    }

    cmd.args([
        "-pix_fmt", "yuv420p",
        "-movflags", "+faststart",
        "-y",
        &final_output,
    ]);

    let status = cmd.status().expect("Failed to execute FFmpeg");

    if !status.success() {
        std::process::exit(1);
    }
}

//==============================================================================
// trim-clip-to
// Description: Trim video/audio using start and end timestamps (Output Seeking)
// References: [LIB-01] through [LIB-06], [LIB-10]
//==============================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path; 
// Removed unused imports to fix compiler warnings
use ffmpeg_rust_scripts::{get_media_info, has_encoder, format_time_for_filename};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "trim video or audio clips using start and end timestamps",
    after_help = "Example:\n  trim-clip-to -s 00:00:45 -i input.mkv -t 00:01:30\n\n  This creates a 45s clip starting at 45s and ending at 1m 30s.\n\nDependencies:\n  ffmpeg: https://www.ffmpeg.org/",
    override_usage = "trim-clip-to [OPTIONS] -s <START> -i <INFILE> -t <END>"
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// start time (HH:MM:SS.mmm)
    #[arg(short = 's', help = "start time")]
    start: String,

    /// input file
    #[arg(short = 'i', help = "input file")]
    infile: String,

    /// end time (HH:MM:SS.mmm)
    #[arg(short = 't', help = "end time")]
    end: String,

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

fn main() {
    let args = Args::parse();

    if !Path::new(&args.infile).exists() {
        eprintln!("! Error: Input file '{}' does not exist.", args.infile);
        std::process::exit(1);
    }

    let info = get_media_info(&args.infile);

    // 1. Generate the cross-platform output path
    let out_path = match args.outfile {
        Some(ref path) => path.clone(),
        None => {
            // Get clean HH:MM:SS for the filename (strip milliseconds)
            let start_clean = args.start.split('.').next().unwrap_or("00:00:00");
            let end_clean = args.end.split('.').next().unwrap_or("00:00:00");

            // LIB-10: OS check for Windows safety (replaces : with -)
            let start_fs = format_time_for_filename(start_clean);
            let end_fs = format_time_for_filename(end_clean);

            format!("{}-[{}–{}].{}", info.stem, start_fs, end_fs, info.extension)
        }
    };

    let ffmpeg_output_path = format!("./{}", out_path);

    // 2. Select runner based on extension (Case-insensitive)
    match info.extension.to_lowercase().as_str() {
        "mp4" | "m4v" | "mov" | "mkv" => {
            let aac_encoder = if has_encoder("libfdk_aac") { "libfdk_aac" } else { "aac" };
            run_ffmpeg_video(&args, &ffmpeg_output_path, aac_encoder, &info.extension);
        },
        "webm" => run_ffmpeg_webm(&args, &ffmpeg_output_path),
        "m4a" | "aac" => {
            let aac_encoder = if has_encoder("libfdk_aac") { "libfdk_aac" } else { "aac" };
            run_ffmpeg_audio(&args, &ffmpeg_output_path, aac_encoder, "adts"); 
        },
        "mp3" => run_ffmpeg_audio(&args, &ffmpeg_output_path, "libmp3lame", "mp3"),
        "wav" => run_ffmpeg_audio(&args, &ffmpeg_output_path, "pcm_s16le", "wav"),
        "ogg" => run_ffmpeg_audio(&args, &ffmpeg_output_path, "libopus", "ogg"),
        _ => run_ffmpeg_fallback(&args, &ffmpeg_output_path),
    }
}

/// Video Runner (Output Seeking: -i before -ss/-to)
fn run_ffmpeg_video(args: &Args, out_path: &str, aac: &str, ext: &str) {
    let mut cmd = Command::new("ffmpeg");
    cmd.args([
        "-hide_banner", "-stats", "-v", "error",
        "-i", &args.infile, 
        "-ss", &args.start, 
        "-to", &args.end,
        "-c:a", aac, "-c:v", "libx264", "-profile:v", "high",
        "-pix_fmt", "yuv420p", "-movflags", "+faststart",
    ]);
    
    // Explicitly set format except for MKV
    if ext.to_lowercase() != "mkv" {
        cmd.args(["-f", ext]);
    }
    
    cmd.arg(out_path);
    cmd.status().expect("Failed to execute FFmpeg");
}

/// WebM Runner (Output Seeking)
fn run_ffmpeg_webm(args: &Args, out_path: &str) {
    Command::new("ffmpeg")
        .args([
            "-hide_banner", "-stats", "-v", "error",
            "-i", &args.infile, 
            "-ss", &args.start, 
            "-to", &args.end,
            "-c:a", "libopus", "-c:v", "vp9", "-f", "webm", out_path
        ])
        .status().expect("Failed to execute FFmpeg");
}

/// Audio Runner (Output Seeking)
fn run_ffmpeg_audio(args: &Args, out_path: &str, codec: &str, format: &str) {
    Command::new("ffmpeg")
        .args([
            "-hide_banner", "-stats", "-v", "error",
            "-i", &args.infile, 
            "-ss", &args.start, 
            "-to", &args.end,
            "-c:a", codec, "-f", format, out_path
        ])
        .status().expect("Failed to execute FFmpeg");
}

/// Fallback Stream Copy (Output Seeking)
fn run_ffmpeg_fallback(args: &Args, out_path: &str) {
    Command::new("ffmpeg")
        .args([
            "-hide_banner", "-stats", "-v", "error",
            "-i", &args.infile, 
            "-ss", &args.start, 
            "-to", &args.end,
            "-c", "copy", out_path
        ])
        .status().expect("Failed to execute FFmpeg");
}

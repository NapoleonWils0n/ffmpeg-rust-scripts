//==============================================================================
// trim-clip
// Description: Trim video or audio clips with millisecond accuracy
// References: [LIB-01] through [LIB-06], [LIB-10]
//==============================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path; 
use ffmpeg_rust_scripts::{get_media_info, parse_to_seconds, format_seconds_ms, has_encoder, format_time_for_filename};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "trim video or audio clips with millisecond accuracy\nhttps://trac.ffmpeg.org/wiki/Seeking",
    after_help = "Example:\n  trim-clip -s 00:00:30 -i input -t 00:00:30 -o output\n\n  This will create a 30 second clip starting at 30 seconds and ending at 60 seconds.\n\nDependencies:\n  ffmpeg: https://www.ffmpeg.org/\n\nNotes:\n  If -o is not provided, defaults to: input-name-[start-end].ext\n\n",
    override_usage = "trim-clip [OPTIONS] -s <START> -i <INFILE> -t <DURATION>"
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// start time (HH:MM:SS.mmm)
    #[arg(short = 's', help = "start time")]
    start: String,

    /// input file
    #[arg(short = 'i', help = "input file")]
    infile: String,

    /// number of seconds after start time (HH:MM:SS.mmm)
    #[arg(short = 't', help = "number of seconds after start time")]
    duration: String,

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

    // Check if input file exists
    if !Path::new(&args.infile).exists() {
        eprintln!("! Error: Input file '{}' does not exist.", args.infile);
        std::process::exit(1);
    }

    // Get file name and extension
    let info = get_media_info(&args.infile);

    // 1. Generate the cross-platform output path
    let out_path = match args.outfile {
        Some(ref path) => path.clone(),
        None => {
            let start_sec = parse_to_seconds(&args.start);
            let duration_sec = parse_to_seconds(&args.duration);
            let end_sec = start_sec + duration_sec;

            // Get clean HH:MM:SS for the filename (strip milliseconds)
            let start_clean = format_seconds_ms(start_sec).split('.').next().unwrap_or("00:00:00").to_string();
            let end_clean = format_seconds_ms(end_sec).split('.').next().unwrap_or("00:00:00").to_string();

            // LIB-10: OS check to replace colons with dashes if on Windows
            let start_fs = format_time_for_filename(&start_clean);
            let end_fs = format_time_for_filename(&end_clean);

            format!("{}-[{}-{}].{}", info.stem, start_fs, end_fs, info.extension)
        }
    };

    // Use ./ prefix to ensure FFmpeg doesn't treat colons in filenames as protocols
    let ffmpeg_output_path = format!("./{}", out_path);

    // 2. Select the correct FFmpeg runner based on extension
    match info.extension.to_lowercase().as_str() {
        // VIDEO FORMATS
        "mp4" | "m4v" | "mov" | "mkv" => {
            let aac_encoder = if has_encoder("libfdk_aac") { "libfdk_aac" } else { "aac" };
            run_ffmpeg_video(&args, &ffmpeg_output_path, aac_encoder, &info.extension);
        },
        "webm" => {
            run_ffmpeg_webm(&args, &ffmpeg_output_path);
        },
        // AUDIO FORMATS
        "m4a" | "aac" => {
            let aac_encoder = if has_encoder("libfdk_aac") { "libfdk_aac" } else { "aac" };
            run_ffmpeg_audio(&args, &ffmpeg_output_path, aac_encoder, "adts"); 
        },
        "mp3" => {
            run_ffmpeg_audio(&args, &ffmpeg_output_path, "libmp3lame", "mp3");
        },
        "wav" => {
            run_ffmpeg_audio(&args, &ffmpeg_output_path, "pcm_s16le", "wav");
        },
        "ogg" => {
            run_ffmpeg_audio(&args, &ffmpeg_output_path, "libopus", "ogg");
        },
        // FALLBACK
        _ => {
            run_ffmpeg_fallback(&args, &ffmpeg_output_path);
        }
    }
}

/// FFmpeg command for General Video (MP4, MOV, MKV)
fn run_ffmpeg_video(args: &Args, out_path: &str, aac: &str, ext: &str) {
    let mut cmd = Command::new("ffmpeg");
    cmd.args([
        "-hide_banner", "-stats", "-v", "error",
        "-ss", &args.start, "-i", &args.infile, "-t", &args.duration,
        "-c:a", aac, "-c:v", "libx264", "-profile:v", "high",
        "-pix_fmt", "yuv420p", "-movflags", "+faststart",
    ]);
    
    // Explicitly set format for MP4/MOV, let FFmpeg auto-detect for MKV
    if ext != "mkv" {
        cmd.args(["-f", ext]);
    }
    
    cmd.arg(out_path);
    cmd.status().expect("Failed to execute FFmpeg");
}

/// FFmpeg command for WebM video
fn run_ffmpeg_webm(args: &Args, out_path: &str) {
    Command::new("ffmpeg")
        .args([
            "-hide_banner", "-stats", "-v", "error",
            "-ss", &args.start, "-i", &args.infile, "-t", &args.duration,
            "-c:a", "libopus", "-c:v", "vp9", "-f", "webm", out_path
        ])
        .status().expect("Failed to execute FFmpeg");
}

/// FFmpeg command for specific Audio formats
fn run_ffmpeg_audio(args: &Args, out_path: &str, codec: &str, format: &str) {
    Command::new("ffmpeg")
        .args([
            "-hide_banner", "-stats", "-v", "error",
            "-ss", &args.start, "-i", &args.infile, "-t", &args.duration,
            "-c:a", codec, "-f", format, out_path
        ])
        .status().expect("Failed to execute FFmpeg");
}

/// Fallback command (Stream Copy) - Helps with MKV and unknown types
fn run_ffmpeg_fallback(args: &Args, out_path: &str) {
    Command::new("ffmpeg")
        .args([
            "-hide_banner", "-stats", "-v", "error",
            "-ss", &args.start, "-i", &args.infile, "-t", &args.duration,
            "-c", "copy", out_path
        ])
        .status().expect("Failed to execute FFmpeg");
}

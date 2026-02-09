//==============================================================================
// trim-clip-to
// Description: Trim video/audio using start and end timestamps (Output Seeking)
// References: [LIB-01] through [LIB-06], [LIB-10], [LIB-11]
//==============================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path; 
// Removed unused imports to fix compiler warnings
use ffmpeg_rust_scripts::{get_media_info, has_encoder, format_time_for_filename, hardware_encoding};

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
            // In trim-clip-to, args.start and args.end are already strings.
            // We just need to ensure they are processed for the OS.
            let start_fs = format_time_for_filename(&args.start);
            let end_fs = format_time_for_filename(&args.end);

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
fn run_ffmpeg_video(args: &Args, out_path: &str, aac_encoder: &str, ext: &str) {
    // 1. Determine encoder and parameters upfront
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
            vec!["-crf", "18", "-preset", "medium"],
        )
    };

    // 2. Create the unified command
    let mut cmd = Command::new("ffmpeg");

    // Input and Seeking (Exact order preserved)
    cmd.args([
        "-hide_banner",
        "-stats",
        "-v", "error", 
        "-i", &args.infile,
        "-ss", &args.start,
        "-to", &args.end,
    ]);

    // Apply selected Video Encoder and Params
    cmd.arg("-c:v").arg(v_codec);
    cmd.args(v_params);

    // Audio and Output settings
    cmd.args([
        "-c:a", aac_encoder,
        "-pix_fmt", "yuv420p",
        "-movflags", "+faststart",
        "-y", // Force overwrite
    ]);

    if ext != "mkv" {
        cmd.args(["-f", ext]);
    }

    cmd.arg(out_path);

    // 3. Execution
    let status = cmd.status().expect("Failed to execute FFmpeg");
    
    if !status.success() {
        eprintln!("! FFmpeg exited with an error.");
    }
}

/// WebM Runner (Output Seeking)
fn run_ffmpeg_webm(args: &Args, out_path: &str) {
    let mut cmd = Command::new("ffmpeg");

    // Common Input/Seeking
    cmd.args(vec![
        "-hide_banner",
        "-stats",
        "-v", "error",
        "-i", &args.infile,
        "-ss", &args.start,
        "-to", &args.end,
    ]);

    // WebM Specific Encoding
    cmd.args(vec![
        "-c:v", "libvpx-vp9",
        "-c:a", "libopus",
        "-f", "webm",
        "-y",
    ]);

    cmd.arg(out_path);

    let status = cmd.status().expect("Failed to execute FFmpeg");
    if !status.success() {
        eprintln!("! FFmpeg WebM export exited with an error.");
    }
}

/// Audio Runner (Output Seeking)
fn run_ffmpeg_audio(args: &Args, out_path: &str, codec: &str, format: &str) {
    let mut cmd = Command::new("ffmpeg");

    cmd.args(vec![
        "-hide_banner",
        "-stats",
        "-v", "error",
        "-i", &args.infile,
        "-ss", &args.start,
        "-to", &args.end,
    ]);

    cmd.args(vec![
        "-c:a", codec,
        "-f", format,
        "-y",
    ]);

    cmd.arg(out_path);

    let status = cmd.status().expect("Failed to execute FFmpeg");
    if !status.success() {
        eprintln!("! FFmpeg Audio export exited with an error.");
    }
}

/// Fallback Stream Copy (Output Seeking)
fn run_ffmpeg_fallback(args: &Args, out_path: &str) {
    let mut cmd = Command::new("ffmpeg");

    cmd.args(vec![
        "-hide_banner",
        "-stats",
        "-v", "error",
        "-i", &args.infile,
        "-ss", &args.start,
        "-to", &args.end,
    ]);

    cmd.args(vec![
        "-c", "copy",
        "-y",
    ]);

    cmd.arg(out_path);

    let status = cmd.status().expect("Failed to execute FFmpeg");
    if !status.success() {
        eprintln!("! FFmpeg Fallback export exited with an error.");
    }
}

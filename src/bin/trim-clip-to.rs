//==============================================================================
// trim-clip-to
// Description: Trim video or audio clips by specifying start and end timestamps
// References: [LIB-01], [LIB-02], [LIB-03], [LIB-06]
//==============================================================================

use clap::Parser;
use std::process::Command;
// [LIB-01] Path import used for file existence check
use std::path::Path; 
use ffmpeg_scripts_rust::{get_media_info, has_encoder}; 

#[derive(Parser, Debug)]
#[command(
    author, 
    version, 
    about = "trim video or audio clips by specifying start and end timestamps\nhttps://trac.ffmpeg.org/wiki/Seeking",
    after_help = "Example:\n  trim-clip-to -s 00:00:30 -i input -t 00:01:00 -o output\n\n  This will create a 30 second clip starting at 30 seconds and ending at 60 seconds.\n\nDependencies:\n  ffmpeg: https://www.ffmpeg.org/\n\nNotes:\n  If -o is not provided, defaults to: input-name-[start–end].(mp4|webm|aac|mp3|wav|ogg)",
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

    /// optional output file
    #[arg(short = 'o', help = "optional output file")]
    outfile: Option<String>,

    /// Print help
    #[arg(short = 'h', action = clap::ArgAction::Help)]
    help: Option<bool>,

    /// Print version
    #[arg(short = 'v', action = clap::ArgAction::Version)]
    version: Option<bool>,
}

fn main() {
    let args = Args::parse();

    // Check if input file exists [LIB-01]
    if !Path::new(&args.infile).exists() {
        eprintln!("! Error: Input file '{}' does not exist.", args.infile);
        std::process::exit(1);
    }

    // Get file name and extension [LIB-03]
    let info = get_media_info(&args.infile);

    // Check available AAC encoders [LIB-06]
    let aac_codec = if has_encoder("libfdk_aac") { "libfdk_aac" } else { "aac" };

    // Decision logic for file extensions
    let out_ext = match info.extension.as_str() {
        "mp4" | "mov" | "mkv" | "m4v" => "mp4",
        "webm" => "webm",
        "aac" | "m4a" => "m4a",
        "mp3" => "mp3",
        "wav" => "wav",
        "ogg" => "ogg",
        _ => &info.extension,
    };

    // Format final filename string [Uses LIB-02 / MediaInfo fields]
    let out = args.outfile.clone().unwrap_or_else(|| {
        format!("{}-trimmed-to-[{}–{}].{}", info.stem, args.start, args.end, out_ext)
    });

    // Use ./ prefix to ensure FFmpeg doesn't treat colons in the filename as a protocol
    let ffmpeg_output_path = format!("./{}", out);

    // Match extension to trigger the right FFmpeg command
    match out_ext {
        "mp4" => run_ffmpeg_video(&args, &ffmpeg_output_path, aac_codec),
        "webm" => run_ffmpeg_webm(&args, &ffmpeg_output_path),
        "m4a" => run_ffmpeg_audio(&args, &ffmpeg_output_path, aac_codec, "mp4"),
        "mp3" => run_ffmpeg_audio(&args, &ffmpeg_output_path, "libmp3lame", "mp3"),
        "wav" => run_ffmpeg_audio(&args, &ffmpeg_output_path, "pcm_s16le", "wav"),
        "ogg" => run_ffmpeg_audio(&args, &ffmpeg_output_path, "libopus", "ogg"),
        _ => eprintln!("! {} is not a recognized media file", args.infile),
    }
}

/// FFmpeg command for MP4 video
/// Encoders: Video = libx264, Audio = libfdk_aac or aac
fn run_ffmpeg_video(args: &Args, out_path: &str, aac: &str) {
    Command::new("ffmpeg")
        .args([
            "-hide_banner", "-stats", "-v", "panic",
            "-ss", &args.start, "-to", &args.end, "-i", &args.infile,
            "-c:a", aac, "-c:v", "libx264", "-profile:v", "high",
            "-pix_fmt", "yuv420p", "-movflags", "+faststart", "-f", "mp4", out_path
        ])
        .status().expect("Failed to execute FFmpeg");
}

/// FFmpeg command for WebM video
/// Encoders: Video = vp9, Audio = libopus
fn run_ffmpeg_webm(args: &Args, out_path: &str) {
    Command::new("ffmpeg")
        .args([
            "-hide_banner", "-stats", "-v", "panic",
            "-ss", &args.start, "-to", &args.end, "-i", &args.infile,
            "-c:a", "libopus", "-c:v", "vp9", "-f", "webm", out_path
        ])
        .status().expect("Failed to execute FFmpeg");
}

/// FFmpeg command for Audio-only files
/// Encoders: M4A = aac, MP3 = libmp3lame, WAV = pcm_s16le, OGG = libopus
fn run_ffmpeg_audio(args: &Args, out_path: &str, codec: &str, format: &str) {
    Command::new("ffmpeg")
        .args([
            "-hide_banner", "-stats", "-v", "panic",
            "-ss", &args.start, "-to", &args.end, "-i", &args.infile,
            "-c:a", codec, "-f", format, out_path
        ])
        .status().expect("Failed to execute FFmpeg");
}

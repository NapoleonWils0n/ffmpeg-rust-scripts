//==============================================================================
// trim-clip
// Description: Trim video or audio clips with millisecond accuracy
// References: [LIB-01] through [LIB-06]
//==============================================================================

use clap::Parser;
use std::process::Command;
// [LIB-01] Path import used for file existence check
use std::path::Path; 
use ffmpeg_scripts_rust::{get_media_info, parse_to_seconds, format_seconds, has_encoder}; 

#[derive(Parser, Debug)]
#[command(
    author, 
    version, 
    // Use 'about' for the header and 'after_help' to match your shell script's layout
    about = "trim video or audio clips with millisecond accuracy\nhttps://trac.ffmpeg.org/wiki/Seeking",
    after_help = "Example:\n  trim-clip -s 00:00:30 -i input -t 00:00:30 -o output\n\n  This will create a 30 second clip starting at 30 seconds and ending at 60 seconds.\n\nNotes:\n  If -o is not provided, defaults to: input-name-[start-end].(mp4|webm|aac|mp3|wav|ogg)",
)]
// This attribute tells clap to use -v for version and -h for help manually
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// start time (HH:MM:SS.mmm)
    #[arg(short = 's', help = "start time")]
    start: String,

    /// input.(mp4|mov|mkv|m4v|webm|aac|m4a|wav|mp3|ogg)
    #[arg(short = 'i', help = "input file")]
    infile: String,

    /// number of seconds after start time (HH:MM:SS.mmm)
    #[arg(short = 't', help = "number of seconds after start time")]
    duration: String,

    /// optional argument: output.(mp4|webm|aac|mp3|wav|ogg)
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

    // Parse timestamps and calculate end time [LIB-04, LIB-05]
    let start_sec = parse_to_seconds(&args.start);
    let dur_sec = parse_to_seconds(&args.duration);
    let calculated_end = format_seconds(start_sec + dur_sec);

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
        format!("{}-[{}-{}].{}", info.stem, args.start, calculated_end, out_ext)
    });

    // Match extension to trigger the right FFmpeg command
    match out_ext {
        "mp4" => run_ffmpeg_video(&args, &out, aac_codec),
        "webm" => run_ffmpeg_webm(&args, &out),
        "m4a" => run_ffmpeg_audio(&args, &out, aac_codec, "mp4"),
        "mp3" => run_ffmpeg_audio(&args, &out, "libmp3lame", "mp3"),
        "wav" => run_ffmpeg_audio(&args, &out, "pcm_s16le", "wav"),
        "ogg" => run_ffmpeg_audio(&args, &out, "libopus", "ogg"),
        _ => eprintln!("! {} is not a recognized media file", args.infile),
    }
}

/// FFmpeg command for MP4 video
/// Encoders: Video = libx264, Audio = libfdk_aac or aac
fn run_ffmpeg_video(args: &Args, out: &str, aac: &str) {
    Command::new("ffmpeg")
        .args([
            "-hide_banner", "-stats", "-v", "panic",
            "-ss", &args.start, "-i", &args.infile, "-t", &args.duration,
            "-c:a", aac, "-c:v", "libx264", "-profile:v", "high",
            "-pix_fmt", "yuv420p", "-movflags", "+faststart", "-f", "mp4", out
        ])
        .status().expect("Failed to execute FFmpeg");
}

/// FFmpeg command for WebM video
/// Encoders: Video = vp9, Audio = libopus
fn run_ffmpeg_webm(args: &Args, out: &str) {
    Command::new("ffmpeg")
        .args([
            "-hide_banner", "-stats", "-v", "panic",
            "-ss", &args.start, "-i", &args.infile, "-t", &args.duration,
            "-c:a", "libopus", "-c:v", "vp9", "-f", "webm", out
        ])
        .status().expect("Failed to execute FFmpeg");
}

/// FFmpeg command for Audio-only files
/// Encoders: M4A = aac, MP3 = libmp3lame, WAV = pcm_s16le, OGG = libopus
fn run_ffmpeg_audio(args: &Args, out: &str, codec: &str, format: &str) {
    Command::new("ffmpeg")
        .args([
            "-hide_banner", "-stats", "-v", "panic",
            "-ss", &args.start, "-i", &args.infile, "-t", &args.duration,
            "-c:a", codec, "-f", format, out
        ])
        .status().expect("Failed to execute FFmpeg");
}

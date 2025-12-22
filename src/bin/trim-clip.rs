use clap::Parser;
use std::process::Command;
// Use the package name from your Cargo.toml
use ffmpeg_scripts_rust::{get_media_info, parse_to_seconds, format_seconds, has_encoder}; 

#[derive(Parser, Debug)]
#[command(
    author, 
    version, 
    // Use 'about' for the header and 'after_help' to match your shell script's layout
    about = "trim video or audio clips with millisecond accuracy\nhttps://trac.ffmpeg.org/wiki/Seeking",
    after_help = "Example:\n  trim-clip -s 00:00:00.000 -i input -t 00:00:00.000 -o output\n\nNotes:\n  If -o is not provided, defaults to: input-name-[start-end].(mp4|webm|aac|mp3|wav|ogg)",
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

    let info = get_media_info(&args.infile);
    let aac_codec = if has_encoder("libfdk_aac") { "libfdk_aac" } else { "aac" };

    // Calculate end timestamp
    let start_sec = parse_to_seconds(&args.start);
    let dur_sec = parse_to_seconds(&args.duration);
    let calculated_end = format_seconds(start_sec + dur_sec);

    // 1. Logic to match your shell script: default all video to .mp4
    let out_ext = match info.extension.as_str() {
        "mp4" | "mov" | "mkv" | "m4v" => "mp4",
        "webm" => "webm",
        "aac" | "m4a" => "m4a",
        "mp3" => "mp3",
        "wav" => "wav",
        "ogg" => "ogg",
        _ => &info.extension,
    };

    // 2. Format filename exactly like the shell script: [start-end].ext
    let out = args.outfile.clone().unwrap_or_else(|| {
        format!("{}-[{}-{}].{}", info.stem, args.start, calculated_end, out_ext)
    });

    // 3. Match extension to trigger the right FFmpeg command
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

fn run_ffmpeg_webm(args: &Args, out: &str) {
    Command::new("ffmpeg")
        .args([
            "-hide_banner", "-stats", "-v", "panic",
            "-ss", &args.start, "-i", &args.infile, "-t", &args.duration,
            "-c:a", "libopus", "-c:v", "vp9", "-f", "webm", out
        ])
        .status().expect("Failed to execute FFmpeg");
}

fn run_ffmpeg_audio(args: &Args, out: &str, codec: &str, format: &str) {
    Command::new("ffmpeg")
        .args([
            "-hide_banner", "-stats", "-v", "panic",
            "-ss", &args.start, "-i", &args.infile, "-t", &args.duration,
            "-c:a", codec, "-f", format, out
        ])
        .status().expect("Failed to execute FFmpeg");
}

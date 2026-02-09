//==============================================================================
// trim-remote-clip
// Description: Trim a remote video clip using yt-dlp and ffmpeg with -to end time
// References: [LIB-01], [LIB-10], [LIB-11]
//==============================================================================

use clap::Parser;
use std::process::Command;
// Import the filename formatter from your library
use ffmpeg_rust_scripts::{format_time_for_filename, hardware_encoding};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Trim remote video clips with millisecond accuracy",
    after_help = "Example:\n  trim-remote-clip -s 00:01:00 -t 00:01:30 -i 'URL' -o clip.mp4\n\nDependencies:\n  ffmpeg: https://www.ffmpeg.org/\n  yt-dlp: https://github.com/yt-dlp/yt-dlp\n  deno: https://deno.com/",
)]
// disable_version_flag allows lowercase -v
// disable_help_flag prevents the naming conflict with the manual 'help' field
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// Start time (HH:MM:SS.mmm)
    #[arg(short = 's', required = true)]
    start: String,

    /// End time (HH:MM:SS.mmm)
    #[arg(short = 't', required = true)]
    end: String,

    /// Input URL (YouTube, Vimeo, etc.)
    #[arg(short = 'i', required = true)]
    input: String,

    /// Output filename (optional, defaults to Title-[start-end].mp4)
    #[arg(short = 'o')]
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

    // 1. Fetch metadata using yt-dlp
    let metadata = Command::new("yt-dlp")
        .args(["--get-title", "--get-filename", "-o", "%(ext)s", "--default-search", "ytsearch", &args.input])
        .output()
        .expect("Failed to execute yt-dlp for metadata");

    let meta_str = String::from_utf8_lossy(&metadata.stdout);
    let mut lines = meta_str.lines();
    let raw_title = lines.next().unwrap_or("remote_video").trim();
    let extension = lines.next().unwrap_or("mp4").trim(); // Detect if remote is webm or mp4

    // 2. Generate Output Path
    let out_path = args.outfile.clone().unwrap_or_else(|| {
        let start_fs = format_time_for_filename(&args.start);
        let end_fs = format_time_for_filename(&args.end);
        let safe_title = raw_title.replace(['/', ':'], "_");
        format!("{}-[{}–{}].{}", safe_title, start_fs, end_fs, extension)
    });

    // 3. Get stream URLs
    let url_output = Command::new("yt-dlp")
        .args(["-g", "--default-search", "ytsearch", &args.input])
        .output()
        .expect("Failed to execute yt-dlp for URLs");

    let url_string = String::from_utf8_lossy(&url_output.stdout);
    let stream_urls: Vec<&str> = url_string.trim().lines().collect();

    if stream_urls.is_empty() {
        eprintln!("Error: Could not retrieve stream URLs.");
        std::process::exit(1);
    }

    // 4. Run the appropriate runner
    if extension == "webm" {
        run_remote_webm(&args, &stream_urls, &out_path);
    } else {
        run_remote_mp4(&args, &stream_urls, &out_path);
    }
}

/// Remote MP4 Runner (Standard/NVENC)
fn run_remote_mp4(args: &Args, urls: &[&str], out_path: &str) {
    let (v_codec, v_params) = if hardware_encoding() {
        println!("+ using hardware acceleration.");
        ("hevc_nvenc", vec!["-tune", "hq", "-preset", "p7", "-rc", "vbr", "-cq", "20", "-b:v", "0"])
    } else {
        println!("+ using software encoding.");
        ("libx264", vec!["-crf", "18", "-preset", "medium"])
    };

    let mut cmd = Command::new("ffmpeg");
    cmd.args(["-hide_banner", "-stats", "-v", "error"]);

    for url in urls {
        cmd.args(["-ss", &args.start, "-to", &args.end, "-i", url]);
    }

    if urls.len() > 1 {
        cmd.args(["-map", "0:v:0", "-map", "1:a:0"]);
    }

    cmd.arg("-c:v").arg(v_codec).args(v_params);
    cmd.args(["-c:a", "aac", "-pix_fmt", "yuv420p", "-movflags", "+faststart", out_path]);

    let status = cmd.status().expect("FFmpeg failed");
    if !status.success() { eprintln!("! FFmpeg remote MP4 export failed."); }
}

/// Remote WebM Runner
fn run_remote_webm(args: &Args, urls: &[&str], out_path: &str) {
    println!("+ using WebM software encoding (VP9/Opus).");
    let mut cmd = Command::new("ffmpeg");
    cmd.args(["-hide_banner", "-stats", "-v", "error"]);

    for url in urls {
        cmd.args(["-ss", &args.start, "-to", &args.end, "-i", url]);
    }

    if urls.len() > 1 {
        cmd.args(["-map", "0:v:0", "-map", "1:a:0"]);
    }

    cmd.args(["-c:v", "libvpx-vp9", "-c:a", "libopus", out_path]);

    let status = cmd.status().expect("FFmpeg failed");
    if !status.success() { eprintln!("! FFmpeg remote WebM export failed."); }
}

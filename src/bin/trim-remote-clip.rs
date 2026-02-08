//==============================================================================
// trim-remote-clip
// Description: Trim a remote video clip using yt-dlp and ffmpeg with -to end time
// References: [LIB-01], [LIB-10]
//==============================================================================

use clap::Parser;
use std::process::Command;
// Import the filename formatter from your library
use ffmpeg_rust_scripts::format_time_for_filename;

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

/// check if the nvenc code is available
fn has_nvenc() -> bool {
    let output = Command::new("ffmpeg").args(["-encoders"]).output().expect("ffmpeg check failed");
    String::from_utf8_lossy(&output.stdout).contains("hevc_nvenc")
}

fn main() {
    let args = Args::parse();

    // 1. Fetch the remote title using yt-dlp
    let title_output = Command::new("yt-dlp")
        .args(["--get-title", "--default-search", "ytsearch", &args.input])
        .output()
        .expect("Failed to execute yt-dlp to get title");

    let raw_title = String::from_utf8_lossy(&title_output.stdout).trim().to_string();
    
    // 2. Generate the cross-platform output path
    let out_path = args.outfile.unwrap_or_else(|| {
        // 3. Handle the Timestamps using the library's built-in OS check
        // This automatically keeps colons on NixOS and uses underscores on Windows
        let start_fs = format_time_for_filename(&args.start);
        let end_fs = format_time_for_filename(&args.end);

        // 4. Handle the Video Title
        // We still need to replace '/' because it's a directory separator on Linux
        // and ':' because it's a drive separator on Windows.
        let safe_title = raw_title.replace(['/', ':'], "_");

        format!("{}-[{}–{}].mp4", safe_title, start_fs, end_fs)
    });

    // 5. Get stream URLs
    let url_output = Command::new("yt-dlp")
        .args(["-g", "--default-search", "ytsearch", &args.input])
        .output()
        .expect("Failed to execute yt-dlp to get stream URLs");

    // FIX: Convert the output to an owned String so the Vec<&str> has a valid reference to borrow from
    let url_string = String::from_utf8_lossy(&url_output.stdout);
    let stream_urls: Vec<&str> = url_string
        .trim()
        .lines()
        .collect();

    if stream_urls.is_empty() {
        eprintln!("Error: Could not retrieve stream URLs.");
        std::process::exit(1);
    }

    // 6. Construct FFmpeg command (Output Seeking)
    let mut ffmpeg = Command::new("ffmpeg");
    ffmpeg.args(["-hide_banner", "-stats", "-v", "error"]);

    // Apply -ss and -to to input(s) for synchronized trimming
    if stream_urls.len() == 1 {
        ffmpeg.args(["-ss", &args.start, "-to", &args.end, "-i", stream_urls[0]]);
    } else {
        // Video Input
        ffmpeg.args(["-ss", &args.start, "-to", &args.end, "-i", stream_urls[0]]);
        // Audio Input 
        ffmpeg.args(["-ss", &args.start, "-to", &args.end, "-i", stream_urls[1]]);
        // Map streams correctly
        ffmpeg.args(["-map", "0:v:0", "-map", "1:a:0"]);
    }

    // 7. Video Encoder Settings
    if has_nvenc() {
        println!("+ Using High-Fidelity Hardware Encoding (NVENC)");
        ffmpeg.args([
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
        ffmpeg.args(["-c:v", "libx264", "-crf", "18", "-preset", "medium"]);
    }

    // 8. Audio and Final Output
    ffmpeg.args(["-c:a", "aac", "-pix_fmt", "yuv420p", "-movflags", "+faststart", &out_path]);

    let status = ffmpeg.status().expect("Failed to execute FFmpeg");

    if !status.success() {
        eprintln!("\nFFmpeg failed to process the remote stream.");
        std::process::exit(1);
    }

}

//==============================================================================
// trim-remote-clip
// Description: Trim a remote video clip using yt-dlp and ffmpeg with -to end time
// References: [LIB-01] std::process::Command for external tool execution
//==============================================================================

use clap::Parser;
use std::process::Command;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Trim remote video clips with millisecond accuracy",
    after_help = "Example:\n  \
                  trim-remote-clip -s 00:01:00 -e 00:01:30 -i 'https://www.youtube.com/watch?v=...' -o clip.mp4\n\n  \
                  This will create a 30 second clip starting at one minute and ending at one minute 30 seconds.\n\n\
                  Dependencies:\n  \
                  ffmpeg, ffplay: https://www.ffmpeg.org/\n\n  \
                  yt-dlp: https://github.com/yt-dlp/yt-dlp",
)]
struct Args {
    /// Start time (HH:MM:SS.mmm)
    #[arg(short = 's', required = true)]
    start: String,

    /// End time (HH:MM:SS.mmm)
    #[arg(short = 'e', required = true)]
    end: String,

    /// Input URL (YouTube, Vimeo, etc.)
    #[arg(short = 'i', required = true)]
    input: String,

    /// Output filename (optional, defaults to Title-[start-end].mp4)
    #[arg(short = 'o')]
    outfile: Option<String>,
}

fn main() {
    let args = Args::parse();

    // 1. Call yt-dlp to get the title and high-quality stream URLs
    // [FIX]: Using -f "bv+ba/b" to ensure we get 1080p/4K streams when available
    let output = Command::new("yt-dlp")
        .args([
            "-i",
            "-f", "bv+ba/b",
            "-g",
            "--no-playlist",
            "--print", "%(title)s",
            &args.input,
        ])
        .output()
        .expect("Failed to execute yt-dlp");

    if !output.status.success() {
        eprintln!("Error: yt-dlp failed to fetch stream information.");
        std::process::exit(1);
    }

    let results: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.to_string())
        .collect();

    if results.len() < 2 {
        eprintln!("Error: Could not retrieve title or stream URLs.");
        std::process::exit(1);
    }

    let video_title = &results[0];
    let stream_urls: Vec<&String> = results.iter().skip(1).collect();
    
    // 2. Determine output filename with compact timestamp format
    let final_output = args.outfile.unwrap_or_else(|| {
        format!("{}-[{} - {}].mp4", video_title, args.start, args.end)
            .replace(" - ", "-") // Result: Title-[00:00:08-00:00:14].mp4
            .replace("/", "_")   // Sanitize title
    });

    // 3. Construct FFmpeg command
    let mut ffmpeg = Command::new("ffmpeg");
    ffmpeg.arg("-hide_banner")
          .arg("-stats")
          .arg("-v").arg("fatal");

    // [FIX]: Synchronized trimming for remote streams
    // We apply -ss and -to to EACH input separately. 
    // This ensures both audio and video streams reach EOF at the exact same time.
    if stream_urls.len() == 1 {
        ffmpeg.arg("-ss").arg(&args.start)
              .arg("-to").arg(&args.end)
              .arg("-i").arg(stream_urls[0]);
    } else {
        // Video Input
        ffmpeg.arg("-ss").arg(&args.start)
              .arg("-to").arg(&args.end)
              .arg("-i").arg(stream_urls[0]);
        
        // Audio Input 
        ffmpeg.arg("-ss").arg(&args.start)
              .arg("-to").arg(&args.end)
              .arg("-i").arg(stream_urls[1]);

        ffmpeg.arg("-map").arg("0:v:0")
              .arg("-map").arg("1:a:0");
    }

    // 4. Encoding parameters for high quality mp4
    ffmpeg.args([
        "-c:v", "libx264",
        "-profile:v", "high",
        "-pix_fmt", "yuv420p",
        "-c:a", "aac",
        "-movflags", "+faststart",
        "-f", "mp4",
        "-y", 
        &final_output,
    ]);

    let status = ffmpeg.status().expect("Failed to execute ffmpeg");

    if !status.success() {
        eprintln!("Error: ffmpeg process exited with an error.");
        std::process::exit(1);
    }
}

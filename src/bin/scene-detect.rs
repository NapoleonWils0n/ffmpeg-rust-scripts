//==============================================================================
// scene-detect
// Description: Detect scene changes in a video and output timestamps
// References: [LIB-01], [LIB-03], [LIB-08], [LIB-09]
//==============================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path;
use std::io::Write;
use ffmpeg_scripts_rust::{get_media_info, get_video_duration, format_seconds_ms, parse_to_seconds};

#[derive(Parser, Debug)]
#[command(
    author, 
    version, 
    about = "Detect scene changes in a video",
    override_usage = "scene-detect -i <INPUT> [OPTIONS]"
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// Input video file
    #[arg(short = 'i', required = true)]
    input: String,

    /// Start time (HH:MM:SS.mmm)
    #[arg(short = 's')]
    start: Option<String>,

    /// End time (HH:MM:SS.mmm)
    #[arg(short = 'e')]
    end: Option<String>,

    /// Detection threshold (0.1 to 0.9) [default: 0.3]
    #[arg(short = 't', default_value = "0.3")]
    threshold: String,

    /// Output format: "sec" for seconds, else HH:MM:SS.mmm
    #[arg(short = 'f')]
    format: Option<String>,

    /// Output filename (optional)
    #[arg(short = 'o')]
    outfile: Option<String>,

    /// Print version
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    version: Option<bool>,

    /// Print help
    #[arg(short = 'h', long = "help", action = clap::ArgAction::Help)]
    help: Option<bool>,
}

fn main() {
    let args = Args::parse();

    if !Path::new(&args.input).exists() {
        eprintln!("Error: Input file '{}' not found.", args.input);
        std::process::exit(1);
    }

    // 1. GATHER METADATA & DURATION
    let info = get_media_info(&args.input);
    let total_duration = get_video_duration(&args.input);
    
    // Logic: Use video duration with colons for the filename
    let duration_stamp = format_seconds_ms(total_duration)
        .split('.')
        .next()
        .unwrap_or("00:00:00")
        .to_string();

    // 2. TIMING CALCULATIONS
    let start_sec = args.start.as_ref().map(|s| parse_to_seconds(s)).unwrap_or(0.0);
    let end_sec = args.end.as_ref().map(|e| parse_to_seconds(e)).unwrap_or(total_duration);

    // 3. FFMPEG DETECTION
    let filter = if args.start.is_some() && args.end.is_some() {
        format!("[0:v]select='between(t,{},{})'[t1]; [t1]select='gt(scene,{})',metadata=print:file=-", 
                start_sec, end_sec, args.threshold)
    } else {
        format!("select='gt(scene,{})',metadata=print:file=-", args.threshold)
    };

    // Use -loglevel error to suppress warnings but keep progress/stats
    let output = Command::new("ffmpeg")
        .args([
            "-hide_banner", 
            "-loglevel", "error",
            "-stats",
            "-i", &args.input, 
            "-filter_complex", &filter, 
            "-f", "null", "-"
        ])
        .stderr(std::process::Stdio::inherit()) // This shows the stats/progress to terminal
        .output()
        .expect("Failed to execute FFmpeg");

    // 4. PARSE OUTPUT
    let detection_data = String::from_utf8_lossy(&output.stdout);
    let mut timestamps = vec![start_sec];
    for line in detection_data.lines() {
        if line.contains("pts_time:") {
            if let Some(time_str) = line.split("pts_time:").last() {
                if let Ok(t) = time_str.trim().parse::<f64>() {
                    timestamps.push(t);
                }
            }
        }
    }
    timestamps.push(end_sec);

    // 5. FILE WRITING
    let final_output = args.outfile.unwrap_or_else(|| {
        format!("./{}-detection-[{}].txt", info.stem, duration_stamp)
    });

    let mut file = std::fs::File::create(&final_output).expect("Could not create file");
    for &t in &timestamps {
        let entry = if args.format.as_deref() == Some("sec") {
            format!("{:.3}\n", t)
        } else {
            format!("{}\n", format_seconds_ms(t))
        };
        file.write_all(entry.as_bytes()).expect("Failed to write to file");
    }

    println!("\nDetection saved to: {}", final_output);
}

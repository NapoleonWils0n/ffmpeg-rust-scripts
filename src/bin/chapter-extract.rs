//==============================================================================
// chapter-extract
// Description: Extract chapters from media and save to CSV (Time, Title)
// References: [LIB-01] Path validation, [LIB-09] format_seconds_ms
//==============================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path;
use std::fs::File;
use std::io::Write;
use ffmpeg_rust_scripts::{format_seconds_ms, get_media_info};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Extract chapters from a video or audio file and save as a CSV",
    after_help = "Example:\n  chapter-extract -i input.mkv -o chapters.csv\n\n  \
                  This creates a CSV with: Time, Title\n\n\
                  Dependencies:\n  \
                  ffmpeg, ffprobe: https://www.ffmpeg.org/",
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// Input video or audio file
    #[arg(short = 'i', required = true)]
    infile: String,

    /// Output CSV file (optional, defaults to input_name.csv)
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

    if !Path::new(&args.infile).exists() {
        eprintln!("! error: Input file '{}' not found.", args.infile);
        std::process::exit(1);
    }

    let final_output = args.outfile.unwrap_or_else(|| {
        let info = get_media_info(&args.infile);
        format!("{}.csv", info.stem)
    });

    let output = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-show_chapters",
            "-print_format", "csv",
            &args.infile,
        ])
        .output()
        .expect("Failed to execute ffprobe");

    if !output.status.success() {
        eprintln!("! error: ffprobe failed to read chapters.");
        std::process::exit(1);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut csv_content = String::new();
    let mut last_end_time = String::from("00:00:00");

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 8 {
            let start_raw: f64 = parts[4].parse().unwrap_or(0.0);
            let end_raw: f64 = parts[6].parse().unwrap_or(0.0);
            let title = parts[7..].join(",").replace("\"", "");

            // Convert to HH:MM:SS format (trimming milliseconds to match your chapter.csv)
            let start = format_seconds_ms(start_raw).split('.').next().unwrap().to_string();
            last_end_time = format_seconds_ms(end_raw).split('.').next().unwrap().to_string();

            csv_content.push_str(&format!("{},{}\n", start, title));
        }
    }

    // Add the final "End" marker using the end time of the last chapter found
    if !csv_content.is_empty() {
        csv_content.push_str(&format!("{},End\n", last_end_time));
    }

    let mut file = File::create(&final_output).expect("Failed to create output file");
    file.write_all(csv_content.as_bytes()).expect("Failed to write to output file");
}

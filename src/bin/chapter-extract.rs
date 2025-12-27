//==============================================================================
// chapter-extract
// Description: Extract chapters from media and save to CSV (Start, End, Title)
// References: [LIB-01] Path validation, [LIB-09] format_seconds_ms
//==============================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path;
use std::fs::File;
use std::io::Write;
use ffmpeg_scripts_rust::{format_seconds_ms, get_media_info};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Extract chapters from a video or audio file and save as a CSV",
    after_help = "Example:\n  chapter-extract -i input.mkv -o chapters.csv\n\n  \
                  This creates a CSV with: Start Time, End Time, Title\n\n\
                  Dependencies:\n  \
                  ffmpeg, ffprobe: https://www.ffmpeg.org/",
)]
struct Args {
    /// Input video or audio file
    #[arg(short = 'i', required = true)]
    infile: String,

    /// Output CSV file (optional, defaults to input_name.csv)
    #[arg(short = 'o')]
    outfile: Option<String>,
}

fn main() {
    let args = Args::parse();

    // 1. Validate input file exists [LIB-01]
    let path = Path::new(&args.infile);
    if !path.exists() {
        eprintln!("Error: Input file '{}' not found.", args.infile);
        std::process::exit(1);
    }

    // 2. Determine output filename
    let final_output = args.outfile.unwrap_or_else(|| {
        let info = get_media_info(&args.infile);
        format!("{}.csv", info.stem)
    });

    // 3. Run ffprobe to get chapters in CSV format
    // ffprobe columns: chapter,id,timebase,start,start_time,end,end_time,tag:title
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
        eprintln!("Error: ffprobe failed to read chapters.");
        std::process::exit(1);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut csv_content = String::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split(',').collect();
        
        // ffprobe CSV format for chapters usually contains at least 8 fields.
        // Field indices: 4 is start_time, 6 is end_time, 7+ is title tags.
        if parts.len() >= 8 {
            let start_raw: f64 = parts[4].parse().unwrap_or(0.0);
            let end_raw: f64 = parts[6].parse().unwrap_or(0.0);
            
            // Rejoin remaining parts in case the title itself contains commas
            let title = parts[7..].join(",").replace("\"", "");

            // Format to sexagesimal with millisecond precision [LIB-09]
            let start = format_seconds_ms(start_raw);
            let end = format_seconds_ms(end_raw);

            csv_content.push_str(&format!("{},{},{}\n", start, end, title));
        }
    }

    // 4. Write the formatted content to the CSV file
    let mut file = File::create(&final_output).expect("Failed to create CSV file");
    file.write_all(csv_content.as_bytes()).expect("Failed to write content");
}

//==============================================================================
// chapter-csv
// Description: Convert a chapter CSV (Time, Title) to FFmpeg metadata format
// References: [LIB-01] Path validation, [LIB-04] parse_to_seconds
//==============================================================================

use clap::Parser;
use std::fs::{read_to_string, File};
use std::io::Write;
use std::path::Path;
use ffmpeg_rust_scripts::{parse_to_seconds, get_media_info};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Convert a chapter CSV (Time, Title) to FFmpeg metadata format",
    after_help = "Example:\n  chapter-csv -i chapters.csv -o chapters-metadata.txt\n\n  \
                  Dependencies:\n  \
                  ffmpeg: https://www.ffmpeg.org/",
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// Input CSV file
    #[arg(short = 'i', required = true)]
    infile: String,

    /// Output metadata file (optional, defaults to input_name-metadata.txt)
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

    // 1. Validate input exists
    if !Path::new(&args.infile).exists() {
        eprintln!("Error: Input file '{}' not found.", args.infile);
        std::process::exit(1);
    }

    // 2. Determine output filename
    let out_name = args.outfile.clone().unwrap_or_else(|| {
        let info = get_media_info(&args.infile);
        format!("{}-metadata.txt", info.stem)
    });

    // 3. Read CSV and generate FFmpeg metadata
    let content = read_to_string(&args.infile).expect("Failed to read CSV");
    
    let mut entries: Vec<(i64, String)> = Vec::new();
    for line in content.lines() {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 2 {
            let ts_ms = (parse_to_seconds(parts[0]) * 1000.0).round() as i64;
            let title = parts[1..].join(",");
            entries.push((ts_ms, title.trim().to_string()));
        }
    }

    if entries.len() < 2 {
        eprintln!("Error: CSV must have at least one chapter line and one 'End' duration line.");
        std::process::exit(1);
    }

    // Header matches your chapter-metadata.txt exactly
    let mut meta_content = String::from(";FFMETADATA1\n");

    for i in 0..entries.len() - 1 {
        let (start_ms, title) = &entries[i];
        let (next_ts_ms, _) = &entries[i + 1];

        // i == entries.len() - 2 is the last actual chapter before the "End" marker
        let end_ms = if i == entries.len() - 2 {
            *next_ts_ms
        } else {
            next_ts_ms - 1
        };

        meta_content.push_str("[CHAPTER]\n");
        meta_content.push_str("TIMEBASE=1/1000\n");
        meta_content.push_str(&format!("START={}\n", start_ms));
        meta_content.push_str(&format!("END={}\n", end_ms));
        meta_content.push_str(&format!("title={}\n", title));
    }

    // 4. Write to file
    let mut file = File::create(&out_name).expect("Failed to create output file");
    file.write_all(meta_content.as_bytes()).expect("Failed to write to output file");
}

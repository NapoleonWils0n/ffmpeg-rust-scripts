//==============================================================================
// chapter-csv
// Description: Convert a chapter CSV to FFmpeg metadata format
// References: [LIB-01] Path validation, [LIB-04] parse_to_seconds
//==============================================================================

use clap::Parser;
use std::fs::{read_to_string, File};
use std::io::Write;
use std::path::Path;
use ffmpeg_scripts_rust::{parse_to_seconds, get_media_info};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Convert a chapter CSV (Start, End, Title) to FFmpeg metadata format",
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

    // 1. Validate input file exists [LIB-01]
    if !Path::new(&args.infile).exists() {
        eprintln!("Error: Input file '{}' not found.", args.infile);
        std::process::exit(1);
    }

    // 2. Determine output filename - changed to filename-metadata.txt
    let final_output = args.outfile.unwrap_or_else(|| {
        let info = get_media_info(&args.infile);
        format!("{}-metadata.txt", info.stem)
    });

    // 3. Read CSV and generate FFmpeg metadata
    let content = read_to_string(&args.infile).expect("Failed to read CSV");
    
    // Header required by FFmpeg for metadata files
    let mut meta_content = String::from("; FFMPEG METADATA\n");
    meta_content.push_str("major_brand=isom\nminor_version=512\ncompatible_brands=isomiso2avc1mp41\n\n");

    for line in content.lines() {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 3 {
            // Convert sexagesimal (00:00:00.000) to total seconds, then to units for TIMEBASE 1/1000
            // [LIB-04] parse_to_seconds
            let start_ms = (parse_to_seconds(parts[0]) * 1000.0).round() as i64;
            let end_ms = (parse_to_seconds(parts[1]) * 1000.0).round() as i64;
            let title = parts[2..].join(","); // Handle titles that might contain commas

            meta_content.push_str("[CHAPTER]\n");
            meta_content.push_str("TIMEBASE=1/1000\n");
            meta_content.push_str(&format!("START={}\n", start_ms));
            meta_content.push_str(&format!("END={}\n", end_ms));
            meta_content.push_str(&format!("title={}\n\n", title));
        }
    }

    // 4. Write to file
    let mut file = File::create(&final_output).expect("Failed to create metadata file");
    file.write_all(meta_content.as_bytes()).expect("Failed to write metadata");
}

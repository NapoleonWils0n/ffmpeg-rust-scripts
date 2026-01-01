//==============================================================================
// clip-time
// Description: Convert a list of timestamps into a start,duration cutlist
// References: [LIB-01], [LIB-03]
//==============================================================================

use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use ffmpeg_scripts_rust::{parse_to_seconds, format_seconds_ms};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "convert a list of timestamps into an ffmpeg cutlist",
    after_help = "Example:\n  clip-time -i timestamps.txt -o cutlist.txt\n\nInput format:\n  00:00:00\n  00:00:10\n  (Pairs represent start and end of a clip)",
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// input file containing timestamps
    #[arg(short = 'i', required = true, value_name = "INPUT")]
    infile: String,

    /// output file for cutlist
    #[arg(short = 'o', value_name = "OUTFILE")]
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

    let input_path = Path::new(&args.infile);
    if !input_path.exists() {
        eprintln!("Error: Input file '{}' not found.", args.infile);
        std::process::exit(1);
    }

    // Determine output filename
    let out_name = args.outfile.unwrap_or_else(|| {
        let stem = input_path.file_stem().unwrap_or_default().to_string_lossy();
        format!("{}-cutlist.txt", stem)
    });

    let file = File::open(input_path).expect("Failed to open input file");
    let reader = BufReader::new(file);

    // Collect all non-empty lines
    let timestamps: Vec<String> = reader
        .lines()
        .map_while(Result::ok)
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    if timestamps.len() < 2 {
        eprintln!("Error: Need at least two timestamps to create a clip.");
        std::process::exit(1);
    }

    let mut output_file = File::create(&out_name).expect("Failed to create output file");

    // Process pairs: (0,1), (2,3), (4,5)...
    for chunk in timestamps.chunks_exact(2) {
        let start_str = &chunk[0];
        let end_str = &chunk[1];

        let start_sec = parse_to_seconds(start_str);
        let end_sec = parse_to_seconds(end_str);

        if end_sec <= start_sec {
            eprintln!("Warning: End time {} is not after start time {}. Skipping.", end_str, start_str);
            continue;
        }

        let duration_sec = end_sec - start_sec;
        let duration_str = format_seconds_ms(duration_sec);

        // Format: start,duration (removing trailing zeros if necessary)
        writeln!(output_file, "{},{}", start_str, duration_str).expect("Failed to write to output");
    }
}

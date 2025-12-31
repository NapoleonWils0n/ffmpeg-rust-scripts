//==============================================================================
// scene-time
// Description: Create ffmpeg cutlist from scene detection timestamps
// References: [LIB-04], [LIB-09]
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
    about = "Create ffmpeg cutlist from scene detection timestamps",
    after_help = "Example:\n  scene-time -i timestamps.txt -o cutlist.txt\n\nDependencies:\n  ffmpeg: https://www.ffmpeg.org/",
    override_usage = "scene-time -i <INPUT> [OPTIONS]"
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// Input file containing timestamps
    #[arg(short = 'i', required = true)]
    input: String,

    /// Output filename (optional)
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

    let input_path = Path::new(&args.input);
    if !input_path.exists() {
        eprintln!("Error: Input file '{}' not found.", args.input);
        std::process::exit(1);
    }

    // 1. PREPARE OUTPUT FILENAME
    let stem = input_path.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
    let final_output = args.outfile.unwrap_or_else(|| {
        format!("{}-cutlist.txt", stem)
    });

    // 2. READ TIMESTAMPS
    let file = File::open(&args.input).expect("Failed to open input file");
    let reader = BufReader::new(file);
    let mut timestamps: Vec<f64> = Vec::new();

    for line in reader.lines() {
        if let Ok(l) = line {
            let trimmed = l.trim();
            if !trimmed.is_empty() {
                // parse_to_seconds [LIB-04] handles both seconds and HH:MM:SS
                timestamps.push(parse_to_seconds(trimmed));
            }
        }
    }

    if timestamps.len() < 2 {
        eprintln!("Error: Not enough timestamps found in file to create a cutlist.");
        std::process::exit(1);
    }

    // 3. CALCULATE DURATIONS AND WRITE CUTLIST
    let mut out_file = File::create(&final_output).expect("Could not create output file");
    
    // Check first line format to decide output style (sexagesimal vs seconds)
    let first_line = std::fs::read_to_string(&args.input)
        .unwrap_or_default()
        .lines()
        .next()
        .unwrap_or("")
        .to_string();
    
    let is_sexagesimal = first_line.contains(':');

    for i in 0..timestamps.len() - 1 {
        let start = timestamps[i];
        let end = timestamps[i + 1];
        let duration = end - start;

        let entry = if is_sexagesimal {
            format!("{},{}\n", format_seconds_ms(start), format_seconds_ms(duration))
        } else {
            format!("{:.3},{:.3}\n", start, duration)
        };
        
        out_file.write_all(entry.as_bytes()).expect("Failed to write to cutlist");
    }

    println!("Cutlist saved to: {}", final_output);
}

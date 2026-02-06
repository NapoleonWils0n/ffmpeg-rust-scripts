//==============================================================================
// normalize
// Description: normalize audio
// References: [LIB-01] [LIB-03] 
//==============================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path;
use ffmpeg_rust_scripts::{get_media_info};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "2-Pass audio normalization (loudnorm)",
    after_help = "Example:\n  normalize -i input.mp4 -t -3.0 -l -16\n\nDependencies: ffmpeg",
    override_usage = "normalize [OPTIONS] -i <INFILE>"
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]

struct Args {
    #[arg(short = 'i', long = "infile", required = true)]
    infile: String,

    /// Target True Peak (TP) level (Default: -3.0)
    #[arg(short = 't', long = "tp", default_value = "-3.0")]
    tp: String,

    /// Target Integrated Loudness (LUFS) level (Default: -16)
    #[arg(short = 'l', long = "lufs", default_value = "-16")]
    lufs: String,

    #[arg(short = 'o', long = "outfile")]
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
        eprintln!("! Error: Input file '{}' does not exist.", args.infile);
        std::process::exit(1);
    }

    let info = get_media_info(&args.infile); // 
    let out_path = args.outfile.unwrap_or_else(|| {
        format!("{}-normalize.{}", info.stem, info.extension)
    });

    // PASS 1: Analysis
    println!("+ Pass 1: Analyzing file for {} LUFS target...", args.lufs);
    
    let analysis_output = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-i", &args.infile,
            "-af", &format!("loudnorm=I={}:TP={}:LRA=11:print_format=json", args.lufs, args.tp),
            "-f", "null",
            "-",
        ])
        .output()
        .expect("Failed to execute ffmpeg analysis");

    let measurements = String::from_utf8_lossy(&analysis_output.stderr); // 

    // Extracting values using simple string parsing (avoiding heavy regex)
    let get_val = |key: &str| {
        measurements.lines()
            .find(|line| line.contains(key))
            .and_then(|line| line.split('"').nth(3))
            .unwrap_or("")
            .to_string()
    };

    let measured_i = get_val("input_i");
    let measured_tp = get_val("input_tp");
    let measured_lra = get_val("input_lra");
    let measured_thresh = get_val("input_thresh");
    let offset = get_val("target_offset");

    if measured_i.is_empty() {
        eprintln!("! Error: Could not analyze audio. Verify ffmpeg installation."); // [cite: 4, 5]
        std::process::exit(1);
    }

    // PASS 2: Application
    println!("+ Pass 2: Applying normalization (Source: {} LUFS)", measured_i);

    let status = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-i", &args.infile,
            "-af", &format!(
                "loudnorm=I={}:TP={}:LRA=11:measured_i={}:measured_tp={}:measured_lra={}:measured_thresh={}:offset={}:linear=true",
                args.lufs, args.tp, measured_i, measured_tp, measured_lra, measured_thresh, offset
            ),
            "-c:v", "copy", // 
            "-ar", "48000", // Reset sample rate 
            &out_path,
        ])
        .status()
        .expect("Failed to execute ffmpeg normalization");

    if status.success() {
        println!("+ Successfully normalized: {}", out_path);
    }
}

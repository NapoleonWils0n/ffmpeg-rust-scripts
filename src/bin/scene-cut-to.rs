//==============================================================================
// scene-cut-to
// Description: Split video into clips using start time and duration (End-point seeking)
// References: [LIB-01], [LIB-03], [LIB-06], [LIB-10], [LIB-11]
//==============================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path;
use std::fs::File;
use std::io::{BufRead, BufReader};
// Integrated hardware_encoding for cross-platform support
use ffmpeg_rust_scripts::{get_media_info, parse_to_seconds, format_seconds_ms, format_time_for_filename, hardware_encoding};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Split video into clips using a start,duration cutlist by calculating end-point",
    after_help = "Example:\n  scene-cut-to -i input.mp4 -c cutlist.txt\n\nDependencies:\n  ffmpeg: https://www.ffmpeg.org/",
    override_usage = "scene-cut-to -i <INPUT> -c <CUTLIST> [OPTIONS]"
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// Input video file
    #[arg(short = 'i', required = true, value_name = "INPUT")]
    input: String,

    /// Cutlist file comma-separated start,duration
    #[arg(short = 'c', required = true, value_name = "CUTLIST")]
    cutlist: String,

    /// Print help
    #[arg(short = 'h', long = "help", action = clap::ArgAction::Help)]
    help: Option<bool>,

    /// Print version
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    version: Option<bool>,
}

fn main() {
    let args = Args::parse();

    if !Path::new(&args.input).exists() {
        eprintln!("Error: Input file '{}' not found.", args.input);
        std::process::exit(1);
    }

    // 1. Get Media Info
    let info = get_media_info(&args.input);

    // 2. Open Cutlist
    let file = File::open(&args.cutlist).expect("Could not open cutlist file");
    let reader = BufReader::new(file);

    // 3. Encoder Setup (Cross-Platform Logic)
    let (v_codec, v_params) = if hardware_encoding() {
        println!("+ hardware acceleration detected: using NVENC.");
        (
            "hevc_nvenc",
            vec![
                "-tune", "hq",
                "-preset", "p7",
                "-rc", "vbr",
                "-multipass", "fullres",
                "-rc-lookahead", "32",
                "-spatial-aq", "1",
                "-cq", "20",
                "-b:v", "0",
            ],
        )
    } else {
        println!("+ no hardware acceleration detected: falling back to libx264.");
        (
            "libx264",
            vec!["-crf", "18", "-preset", "medium"],
        )
    };

    // 4. Process Each Scene
    for (index, line) in reader.lines().enumerate() {
        let line = line.expect("Could not read line from cutlist");
        if line.trim().is_empty() { continue; }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 2 { continue; }

        let start_raw = parts[0].trim();
        let duration_raw = parts[1].trim();

        // Calculate end time for seeking
        let start_sec = parse_to_seconds(start_raw);
        let dur_sec = parse_to_seconds(duration_raw);
        let end_sec = start_sec + dur_sec;

        let end_filename_raw = format_seconds_ms(end_sec);

        // Format timestamps for filename (Handles OS-specific character safety)
        let start_ts = format_time_for_filename(&format_seconds_ms(start_sec));
        let end_ts = format_time_for_filename(&end_filename_raw);

        let output_name = format!(
            "{}-scene-{:03}-[{}-{}].mp4",
            info.stem, index + 1, start_ts, end_ts
        );

        println!("+ processing scene {:03}: {} -> {}", index + 1, start_raw, end_filename_raw);

        // 5. Build and Execute FFmpeg Command
        let mut ffmpeg_args = vec![
            "-hide_banner",
            "-v", "error",
            "-stats",
            "-i", &args.input,
            "-ss", start_raw,
            "-to", &end_filename_raw,
            "-c:v", v_codec,
        ];

        ffmpeg_args.extend(v_params.iter().cloned());

        ffmpeg_args.extend(vec![
            "-pix_fmt", "yuv420p",
            "-c:a", "aac",
            "-movflags", "+faststart",
            "-y",
            &output_name,
        ]);

        let status = Command::new("ffmpeg")
            .args(&ffmpeg_args)
            .status()
            .expect("Failed to execute FFmpeg");

        if !status.success() {
            eprintln!("Error: FFmpeg failed on scene {}.", index + 1);
        }
    }

    println!("+ processing complete.");
}

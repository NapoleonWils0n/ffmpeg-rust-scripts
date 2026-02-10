//==============================================================================
// scene-detect-auto
// Description: Automated scene detection, timing, and cutting in one command
// References: [LIB-01], [LIB-03], [LIB-04], [LIB-09], [LIB-10] [LIB-11]
//==============================================================================

use clap::Parser;
use std::process::Command;
use std::fs::{File, write};
use std::io::{BufRead, BufReader};
// 1) Use standard library imports including get_video_duration
use ffmpeg_rust_scripts::{get_media_info, get_video_duration, parse_to_seconds, format_seconds_ms, format_time_for_filename, hardware_encoding};

#[derive(Parser, Debug)]
#[command(
    author, 
    version,
    about = "Automated scene detection and video splitting",
    after_help = "Example:\n  scene-detect-auto -i input.mp4\n\nDependencies:\n  ffmpeg, ffprobe: https://www.ffmpeg.org/",
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// input video file
    #[arg(short = 'i', required = true)]
    input: String,

    /// detection threshold (0.0 to 1.0)
    #[arg(short = 't', default_value = "0.3")]
    threshold: String,

    /// Print help
    #[arg(short = 'h', long = "help", action = clap::ArgAction::Help)]
    help: Option<bool>,

    /// Print version
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    version: Option<bool>,
}

fn main() {
    let args = Args::parse();
    let info = get_media_info(&args.input);
    // FIXED: Use function to get duration instead of accessing non-existent field
    let total_duration = get_video_duration(&args.input);

    // --- STEP 1: DETECT SCENES ---
    println!("+ detecting scenes (threshold: {})...", args.threshold);
    let filter = format!("select='gt(scene,{})',showinfo", args.threshold);
    
    let output = Command::new("ffmpeg")
        .args([
            "-hide_banner", "-loglevel", "info",
            "-i", &args.input,
            "-filter_complex", &filter,
            "-f", "null", "-",
        ])
        .output()
        .expect("failed to execute ffmpeg for detection");

    let stderr_out = String::from_utf8_lossy(&output.stderr);
    let mut timestamps = vec!["00:00:00.000".to_string()];

    for line in stderr_out.lines() {
        if line.contains("pts_time:") {
            if let Some(pts_part) = line.split("pts_time:").last() {
                if let Some(ts_str) = pts_part.split_whitespace().next() {
                    if let Ok(ts_float) = ts_str.parse::<f64>() {
                        timestamps.push(format_seconds_ms(ts_float));
                    }
                }
            }
        }
    }
    timestamps.push(format_seconds_ms(total_duration));

    // --- STEP 2: GENERATE CUTLIST ---
    let mut cutlist_content = String::new();
    for i in 0..timestamps.len() - 1 {
        let start = parse_to_seconds(&timestamps[i]);
        let end = parse_to_seconds(&timestamps[i+1]);
        let duration = end - start;
        if duration > 0.1 {
            cutlist_content.push_str(&format!("{},{}\n", timestamps[i], format_seconds_ms(duration)));
        }
    }
    write("cutlist.txt", &cutlist_content).expect("failed to write cutlist.txt");

    // --- STEP 3: ENCODER SETUP ---
    // Removed has_nvenc() and replaced with library check
    let (v_codec, v_params) = if hardware_encoding() {
        println!("+ using hardware acceleration (nvenc).");
        (
            "hevc_nvenc",
            vec![
                "-tune", "hq", "-preset", "p7", "-rc", "vbr",
                "-multipass", "fullres", "-rc-lookahead", "32",
                "-spatial-aq", "1", "-cq", "20", "-b:v", "0",
            ],
        )
    } else {
        println!("+ using software encoding (libx264).");
        ("libx264", vec!["-crf", "18", "-preset", "medium"])
    };

    // --- STEP 4: CUT SCENES ---
    let file = File::open("cutlist.txt").expect("failed to open cutlist.txt");
    let reader = BufReader::new(file);

    // Using multi-clip execution logic from scene-cut.rs
    for (index, line) in reader.lines().enumerate() {
        let line = line.expect("failed to read line");
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 2 { continue; }

        let start_raw = parts[0].trim();
        let dur_raw = parts[1].trim();

        let start_sec = parse_to_seconds(start_raw);
        let dur_sec = parse_to_seconds(dur_raw);
        let end_sec = start_sec + dur_sec;

        let start_filename_raw = format_seconds_ms(start_sec);
        let end_filename_raw = format_seconds_ms(end_sec);

        let start_fs = format_time_for_filename(&start_filename_raw);
        let end_fs = format_time_for_filename(&end_filename_raw);

        let output_name = format!(
            "{}-scene-{:03}-[{}–{}].mp4",
            info.stem, index + 1, start_fs, end_fs
        );

        println!("Cutting Scene {}: {} -> {}", index + 1, start_filename_raw, end_filename_raw);

        let mut ffmpeg_args = vec![
            "-hide_banner", "-v", "error", "-stats",
            "-ss", start_raw,
            "-t", dur_raw,
            "-i", &args.input,
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
            .expect("failed to execute ffmpeg");

        if !status.success() {
            eprintln!("Error: FFmpeg failed on scene {}", index + 1);
        }
    }

    println!("+ processing complete.");
}

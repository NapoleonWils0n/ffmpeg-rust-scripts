//==============================================================================
// scene-cut
// Description: Read a cutlist and split a video into individual scene clips
// References: [LIB-01], [LIB-03], [LIB-04], [LIB-09], [LIB-10] [LIB-11]
//==============================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path;
use std::fs::File;
use std::io::{BufRead, BufReader};
use ffmpeg_rust_scripts::{get_media_info, parse_to_seconds, format_seconds_ms, format_time_for_filename, hardware_encoding};

#[derive(Parser, Debug)]
#[command(
    author, 
    version, 
    about = "Split a video into individual scenes based on a cutlist",
    after_help = "Example:\n  scene-cut -i input.mp4 -c cutlist.txt\n\nDependencies:\n  ffmpeg: https://www.ffmpeg.org/",
    override_usage = "scene-cut -i <INPUT> -c <CUTLIST> [OPTIONS]"
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// Input video file
    #[arg(short = 'i', required = true)]
    input: String,

    /// Cutlist file (comma-separated start,duration)
    #[arg(short = 'c', required = true)]
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
        eprintln!("! error: input file '{}' not found.", args.input);
        std::process::exit(1);
    }
    if !Path::new(&args.cutlist).exists() {
        eprintln!("! error: cutlist file '{}' not found.", args.cutlist);
        std::process::exit(1);
    }

    let info = get_media_info(&args.input);
    
    // 1. Encoder Selection (Check once outside the loop)
    let (v_codec, v_params) = if hardware_encoding() {
        println!("+ using hardware acceleration (nvenc).");
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
        println!("+ using software encoding (libx264).");
        (
            "libx264",
            vec![
                "-crf", "18",
                "-preset", "medium",
            ],
        )
    };

    let file = File::open(&args.cutlist).expect("! error: could not open cutlist.");
    let reader = BufReader::new(file);

    for (index, line) in reader.lines().enumerate() {
        let line = line.unwrap();
        if line.trim().is_empty() { continue; }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 2 { continue; }

        let start_raw = parts[0].trim();
        let duration_raw = parts[1].trim();

        let start_sec = parse_to_seconds(start_raw);
        let dur_sec = parse_to_seconds(duration_raw);
        let end_sec = start_sec + dur_sec;

        // 2. Millisecond Precision in Filename
        let start_filename_raw = format_seconds_ms(start_sec);
        let end_filename_raw = format_seconds_ms(end_sec);
        
        let start_ts = format_time_for_filename(&start_filename_raw);
        let end_ts = format_time_for_filename(&end_filename_raw);

        let output_name = format!("{}-scene-{:03}-[{}-{}].mp4", 
            info.stem, index + 1, start_ts, end_ts);

        println!("+ processing scene {}: {} -> {}", index + 1, start_raw, format_seconds_ms(end_sec));

        // 3. EXECUTE FFMPEG (Unified Vec inside loop)
        let mut cmd = Command::new("ffmpeg");
        
        let mut ffmpeg_args = vec![
            "-hide_banner",
            "-v", "error",
            "-stats",
            "-ss", start_raw,
            "-t", duration_raw,
            "-i", &args.input,
            "-c:v", v_codec,
        ];

        ffmpeg_args.extend(v_params.iter().cloned());

        ffmpeg_args.extend(vec![
            "-pix_fmt", "yuv420p",
            "-c:a", "aac",
            "-movflags", "+faststart",
            &output_name,
        ]);

        let status = cmd.args(ffmpeg_args)
            .status()
            .expect("! error: failed to execute ffmpeg");

        if !status.success() {
            eprintln!("! error: ffmpeg failed on scene {}.", index + 1);
            std::process::exit(1);
        }
    }
}

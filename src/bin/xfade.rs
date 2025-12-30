//==============================================================================
// xfade
// Description: Add a transition effect between two clips using filter_complex
// References: [LIB-01], [LIB-03], [LIB-04], [LIB-08], [LIB-09]
//==============================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path;
use ffmpeg_scripts_rust::{get_media_info, get_video_duration, format_seconds_ms, parse_to_seconds};

#[derive(Parser, Debug)]
#[command(
    author, 
    version, 
    about = "FFmpeg xfade transitions",
    after_help = "TRANSITIONS:\n\
                  circleclose, circlecrop, circleopen, diagbl, diagbr, diagtl, diagtr, \n\
                  dissolve, distancefade, fade, fadeblack, fadegrays, fadewhite, hblur, \n\
                  hlslice, horzclose, horzopen, hrslice, pixelize, radial, rectcrop, \n\
                  slidedown, slideleft, slideright, slideup, smoothdown, smoothleft, \n\
                  smoothright, smoothup, squeezeh, squeezev, vdslice, vertclose, \n\
                  vertopen, vuslice, wipebl, wipebr, wipedown, wipeleft, wiperight, \n\
                  wipetl, wipetr, wipeup"
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// First clip (-a)
    #[arg(short = 'a', required = true)]
    input1: String,

    /// Second clip (-b)
    #[arg(short = 'b', required = true)]
    input2: String,

    /// Transition duration (e.g., 2 or 00:00:02)
    #[arg(short = 'd', required = true)]
    duration: String,

    /// Transition type [default: fade]
    #[arg(short = 't', default_value = "fade")]
    transition: String,

    /// Offset (start time of transition). Calculated automatically if not provided.
    #[arg(short = 'f')]
    offset: Option<String>,

    /// Output file (optional)
    #[arg(short = 'o')]
    outfile: Option<String>,

    /// Print version
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    version: Option<bool>,

    /// Print help
    #[arg(short = 'h', long = "help", action = clap::ArgAction::Help)]
    help: Option<bool>,
}

fn main() {
    let args = Args::parse();

    // Validate inputs
    for f in &[&args.input1, &args.input2] {
        if !Path::new(f).exists() {
            eprintln!("Error: File '{}' not found.", f);
            std::process::exit(1);
        }
    }

    let fade_dur = parse_to_seconds(&args.duration);
    
    // Logic for offset: Use provided -f or calculate (Clip1 Duration - Fade Duration)
    let offset_secs = match args.offset {
        Some(ref f) => parse_to_seconds(f),
        None => {
            let dur1 = get_video_duration(&args.input1);
            dur1 - fade_dur
        }
    };

    if offset_secs < 0.0 {
        eprintln!("Error: Offset calculation resulted in a negative value. Clip 1 is too short for this transition.");
        std::process::exit(1);
    }

    let info = get_media_info(&args.input1);
    let full_ts = format_seconds_ms(fade_dur);
    let timestamp = full_ts.split('.').next().unwrap_or("00:00:00");
    
    let final_output = args.outfile.unwrap_or_else(|| {
        format!("{}-xfade-{}-[{}].mp4", info.stem, args.transition, timestamp)
    });

    // Correct Filter Complex Logic:
    // Both xfade (video) and acrossfade (audio) need to explicitly map their inputs
    let filter_complex = format!(
        "[0:v][1:v]xfade=transition={}:duration={}:offset={}[v]; \
         [0:a][1:a]acrossfade=d={}[a]", 
        args.transition, fade_dur, offset_secs, fade_dur
    );

    let status = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel", "error",
            "-stats",
            "-i", &args.input1,
            "-i", &args.input2,
            "-filter_complex", &filter_complex,
            "-map", "[v]",
            "-map", "[a]",
            "-c:v", "libx264",
            "-crf", "18",
            "-pix_fmt", "yuv420p",
            "-y",
            &final_output,
        ])
        .status()
        .expect("Failed to execute FFmpeg");

    if !status.success() {
        std::process::exit(1);
    }
}

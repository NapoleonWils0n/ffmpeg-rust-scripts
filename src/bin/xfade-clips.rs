//==================================================================================
// xfade-clips
// Description: Add a transition effect between multiple clips using filter_complex
// References: [LIB-01], [LIB-03], [LIB-04], [LIB-08], [LIB-09], [LIB-10] [LIB-11]
//==================================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path;
// Removed get_media_info to fix the warning
use ffmpeg_rust_scripts::{get_video_duration, parse_to_seconds, hardware_encoding};

#[derive(Parser, Debug)]
#[command(
    author, 
    version, 
    about = "FFmpeg xfade transitions",
    after_help = "TRANSITIONS:\n\
                  circleclose, circlecrop, circleopen, diagbl, diagbr, diagtl, diagtr, \n\
                  dissolve, distance, fade, fadeblack, fadegrays, fadewhite, hblur, \n\
                  hlslice, horzclose, horzopen, hrslice, pixelize, radial, rectcrop, \n\
                  slidedown, slideleft, slideright, slideup, smoothdown, smoothleft, \n\
                  smoothright, smoothup, squeezeh, squeezev, vdslice, vertclose, \n\
                  vertopen, vuslice, wipebl, wipebr, wipedown, wipeleft, wiperight, \n\
                  wipetl, wipetr, wipeup\n\n\
                  Dependencies:\n  ffmpeg: https://www.ffmpeg.org/"
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// Input clips in order
    #[arg(short = 'i', long, required = true, num_args = 2..)]
    inputs: Vec<String>,

    /// Transition duration(s). Provide one for all, or one per gap.
    #[arg(short = 'd', long, required = true, num_args = 1..)]
    durations: Vec<String>,

    /// Transition type(s). Provide one for all, or one per gap.
    #[arg(short = 't', long, default_value = "fade", num_args = 1..)]
    transitions: Vec<String>,

    /// Output file
    #[arg(short = 'o', long, required = true)]
    output: String,

    /// Print help
    #[arg(short = 'h', long = "help", action = clap::ArgAction::Help)]
    help: Option<bool>,

    /// Print version
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    version: Option<bool>,
}

fn main() {
    let args = Args::parse();

    for f in &args.inputs {
        if !Path::new(f).exists() {
            eprintln!("! error: File '{}' not found.", f);
            std::process::exit(1);
        }
    }


    let mut filter_complex = String::new();
    let mut total_output_duration = 0.0;
    
    for i in 1..args.inputs.len() {
        let prev_clip_dur = get_video_duration(&args.inputs[i-1]);

        // NEW: Get specific duration for this gap, or fall back to the last one provided
        let current_dur_str = args.durations.get(i-1)
            .unwrap_or_else(|| args.durations.last().unwrap());
        let fade_dur = parse_to_seconds(current_dur_str);

        // Logic: The offset for the NEXT transition is 
        // the current end-of-file timestamp.
        if i == 1 {
            // Transition between Clip 0 and Clip 1 starts at Clip 0's end minus fade
            total_output_duration = prev_clip_dur - fade_dur;
        } else {
            // For subsequent clips, we add the NEW clip's length 
            // and subtract the overlap used by the transition
            total_output_duration += prev_clip_dur - fade_dur;
        }

        let trans_type = if i <= args.transitions.len() {
            &args.transitions[i-1]
        } else {
            &args.transitions[args.transitions.len() - 1]
        };

        let v_in_left = if i == 1 { "[0:v]".to_string() } else { format!("[v{}]", i - 1) };
        let a_in_left = if i == 1 { "[0:a]".to_string() } else { format!("[a{}]", i - 1) };
        let v_out = format!("[v{}]", i);
        let a_out = format!("[a{}]", i);

        // Subtract a very small 'epsilon' (0.01) from the offset.
        // This ensures FFmpeg doesn't try to start a transition 
        // on the literal last null-frame of a clip, which triggers the crash.
        let safe_offset = total_output_duration - 0.01;

        filter_complex.push_str(&format!(
            "{}[{}:v]xfade=transition='{}':duration={:.3}:offset={:.3}{};",
            v_in_left, i, trans_type, fade_dur, safe_offset, v_out
        ));

        filter_complex.push_str(&format!(
            "{}[{}:a]acrossfade=d={:.3}{};",
            a_in_left, i, fade_dur, a_out
        ));
    }

    // FFmpeg Command Building
    let (v_codec, mut v_params) = if hardware_encoding() {
        ("hevc_nvenc", vec!["-preset", "p7", "-cq", "20"])
    } else {
        ("libx264", vec!["-preset", "medium", "-crf", "18"])
    };

    let mut ffmpeg_args = vec!["-hide_banner", "-v", "error", "-stats"];
    for f in &args.inputs {
        ffmpeg_args.extend(vec!["-i", f]);
    }

    let last_idx = args.inputs.len() - 1;
    let v_map = format!("[v{}]", last_idx);
    let a_map = format!("[a{}]", last_idx);

    ffmpeg_args.extend(vec![
        "-filter_complex", &filter_complex,
        "-map", &v_map,
        "-map", &a_map,
        "-c:v", v_codec,
    ]);
    
    ffmpeg_args.append(&mut v_params);
    ffmpeg_args.extend(vec!["-c:a", "aac", "-pix_fmt", "yuv420p", "-movflags", "+faststart", &args.output]);

// DEBUG: Print the command
    println!("--- DEBUG: RUNNING COMMAND ---");
    println!("ffmpeg {}", ffmpeg_args.join(" "));
    println!("------------------------------");

    let status = Command::new("ffmpeg").args(ffmpeg_args).status().expect("failed to execute ffmpeg");
    if !status.success() { std::process::exit(1); }
}

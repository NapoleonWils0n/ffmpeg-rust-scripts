//==============================================================================
// img2video
// Description: Convert a static image (png, jpg, jpeg) to a video file
// References: [LIB-01], [LIB-03] [LIB-04], [LIB-09] [LIB-10] [LIB-11]
//==============================================================================

use clap::Parser;
use std::process::Command; // Cleaned up unused Stdio
use std::path::Path;
use ffmpeg_rust_scripts::{get_media_info, format_seconds_ms, parse_to_seconds, format_time_for_filename, hardware_encoding};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Convert a static image to a video file with a specified duration",
    after_help = "Example:\n  img2video -i input.png -d 00:00:10 -o output.mp4\n\n\
    Dependencies:\n  \
    ffmpeg: https://www.ffmpeg.org/",
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// Input image file (png, jpg, jpeg)
    #[arg(short = 'i', required = true)]
    infile: String,

    /// Duration (e.g., 10 or 00:00:10.500)
    #[arg(short = 'd', required = true)]
    duration: String,

    /// Output file (optional)
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
        eprintln!("! error: input file '{}' not found.", args.infile);
        std::process::exit(1);
    }

    // 2. Determine duration and naming
    let info = get_media_info(&args.infile);
    let duration_secs = parse_to_seconds(&args.duration);
    
    // Convert duration to a string once so it lives long enough for the Command
    let dur_str = duration_secs.to_string(); 
    
    let full_ts = format_seconds_ms(duration_secs);
    let timestamp = format_time_for_filename(&full_ts);
    
    let out_path = args.outfile.unwrap_or_else(|| {
        format!("{}-[{}].mp4", info.stem, timestamp)
    });

    // 3. Encoder Selection Logic
    let (v_codec, v_params) = if hardware_encoding() {
        println!("+ using hardware acceleration.");
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
        println!("+ using software encoding.");
        (
            "libx264",
            vec![
                "-crf", "18",
                "-preset", "medium",
            ],
        )
    };

    // 4. EXECUTE FFMPEG (Unified Vec)
    let mut cmd = Command::new("ffmpeg");
    
    let mut ffmpeg_args = vec![
        "-hide_banner",
        "-v", "error",
        "-stats",
        "-loop", "1",
        "-i", &args.infile,
        "-c:v", v_codec,
    ];

    ffmpeg_args.extend(v_params);

    ffmpeg_args.extend(vec![
        "-t", &dur_str,           // Now referencing a variable that lives until end of main
        "-r", "30",
        "-pix_fmt", "yuv420p",
        "-movflags", "+faststart",
        &out_path,
    ]);

    let status = cmd.args(ffmpeg_args)
        .status()
        .expect("failed to execute ffmpeg");

    if !status.success() {
        eprintln!("! error: ffmpeg failed to process img2video.");
        std::process::exit(1);
    }
}

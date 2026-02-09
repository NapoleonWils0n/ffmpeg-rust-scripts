//==============================================================================
// fade-in
// Description: Apply a fade-in effect to both video and audio with timestamped output
// References: [LIB-01], [LIB-03], [LIB-04], [LIB-09], [LIB-10], [LIB-11]
//==============================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path;
use ffmpeg_rust_scripts::{get_media_info, format_seconds_ms, parse_to_seconds, format_time_for_filename, hardware_encoding};

#[derive(Parser, Debug)]
#[command(
    author, 
    version, 
    about = "Fade in a video and audio clip",
    after_help = "Example:\n  fade-clip -i input.mp4 -d 00:00:02\n\nDependencies:\n  ffmpeg: https://www.ffmpeg.org/",
    override_usage = "fade-clip [OPTIONS] -i <INFILE>"
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// Input video file
    #[arg(short = 'i', required = true)]
    infile: String,

    /// Fade duration (e.g., 2 or 00:00:02)
    #[arg(short = 'd', default_value = "00:00:00.500")]
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

    // 1. Validate input
    if !Path::new(&args.infile).exists() {
        eprintln!("Error: Input file '{}' not found.", args.infile);
        std::process::exit(1);
    }

    // 2. Determine duration and naming
    let info = get_media_info(&args.infile);
    let dur_secs = parse_to_seconds(&args.duration);
    let timestamp = format_time_for_filename(&format_seconds_ms(dur_secs));

    let out_path = args.outfile.unwrap_or_else(|| {
        format!("{}-fade-in-[{}].mp4", info.stem, timestamp)
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

    // 4. Filters
    let v_filter = format!("fade=t=in:st=0:d={}", dur_secs);
    let a_filter = format!("afade=t=in:st=0:d={}", dur_secs);

    // 5. EXECUTE FFMPEG (Unified Vec)
    let mut cmd = Command::new("ffmpeg");
    
    let mut ffmpeg_args = vec![
        "-hide_banner",
        "-v", "error",
        "-stats",
        "-i", &args.infile,
        "-vf", &v_filter,
        "-af", &a_filter,
        "-c:v", v_codec,
    ];

    // Append encoder-specific parameters
    ffmpeg_args.extend(v_params);

    // Finalize arguments
    ffmpeg_args.extend(vec![
        "-c:a", "aac",
        "-pix_fmt", "yuv420p",
        "-movflags", "+faststart",
        &out_path,
    ]);

    let status = cmd.args(ffmpeg_args)
        .status()
        .expect("failed to execute ffmpeg");

    if !status.success() {
        eprintln!("! error: ffmpeg failed to process fade-clip.");
        std::process::exit(1);
    }
}

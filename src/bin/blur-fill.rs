//==============================================================================
// blur-fill
// Description: Fill pillarboxes/letterboxes with a blurred version of the video
// References: [LIB-01], [LIB-03], [LIB-11]
//==============================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path;
use ffmpeg_rust_scripts::{get_media_info, hardware_encoding};

#[derive(Parser, Debug)]
#[command(
    author, 
    version,
    about = "Fill pillarboxes with a blurred version of the input video",
    after_help = "Example:\n  blur-fill -i input.mp4 -b 10 -o output.mp4\n\n\
    Dependencies:\n  \
    ffmpeg: https://www.ffmpeg.org/",
    override_usage = "blur-fill [OPTIONS] -i <INFILE>"
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// Input file
    #[arg(short = 'i', required = true)]
    infile: String,

    /// Blur strength (default: 10)
    #[arg(short = 'b', default_value = "10")]
    blur: u32,

    /// Optional output file
    #[arg(short = 'o')]
    outfile: Option<String>,

    /// Print help
    #[arg(short = 'h', long = "help", action = clap::ArgAction::Help)]
    help: Option<bool>,

    /// Print version
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    version: Option<bool>,
}

fn get_audio_codec(path: &str) -> String {
    let output = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-select_streams", "a:0",
            "-show_entries", "stream=codec_name",
            "-of", "default=noprint_wrappers=1:nokey=1",
            path
        ])
        .output()
        .expect("ffprobe audio check failed");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn main() {
    let args = Args::parse();

    // 1. Validate input exists
    if !Path::new(&args.infile).exists() {
        eprintln!("! error: input file '{}' not found.", args.infile);
        std::process::exit(1);
    }

    // 2. Determine output and filter string
    let info = get_media_info(&args.infile);
    let out_path = args.outfile.unwrap_or_else(|| format!("{}-blurfill.mp4", info.stem));

    let filter_complex = format!(
        "[0:v]split=2[main][bg]; \
         [bg]scale=1920:1080:force_original_aspect_ratio=increase,crop=1920:1080,boxblur={}:10[blurred]; \
         [main]scale=-1:1080[foreground]; \
         [blurred][foreground]overlay=(W-w)/2:(H-h)/2",
        args.blur
    );

    // 3. Encoder and Audio Logic
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

    let audio_codec = get_audio_codec(&args.infile);
    let a_params = if audio_codec == "aac" {
        println!("+ audio is aac: using stream copy.");
        vec!["copy"]
    } else {
        println!("+ audio is {}: transcoding to aac.", audio_codec);
        vec!["aac"]
    };

    // 4. EXECUTE FFMPEG (Unified Vec)
    let mut cmd = Command::new("ffmpeg");

    let ffmpeg_args = vec![
        "-hide_banner",
        "-v", "error",
        "-stats",
        "-i", &args.infile,
        "-filter_complex", &filter_complex,
        "-c:v", v_codec,
    ];

    cmd.args(ffmpeg_args);
    cmd.args(v_params); // Append the vertical pairs
    
    cmd.args([
        "-c:a", a_params[0],
        "-pix_fmt", "yuv420p",
        "-movflags", "+faststart",
        &out_path,
    ]);

    let status = cmd.status().expect("failed to execute ffmpeg");

    if !status.success() {
        eprintln!("! error: ffmpeg failed to process blur-fill.");
        std::process::exit(1);
    }
}

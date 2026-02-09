//==============================================================================
// overlay-clip
// Description: Overlay a video onto a background clip at a specific time
// References: [LIB-01], [LIB-03], [LIB-04], [LIB-09], [LIB-10] [LIB-11]
//==============================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path;
use ffmpeg_rust_scripts::{get_media_info, parse_to_seconds, format_seconds_ms, format_time_for_filename, hardware_encoding};

#[derive(Parser, Debug)]
#[command(
    author, 
    version, 
    about = "Overlay one video clip on top of another video clip",
    after_help = "Example:\n  overlay-clip -a bottom-video.mp4 -b overlay.mp4 -p 00:00:05\n\nDependencies:\n  ffmpeg: https://www.ffmpeg.org/",
    override_usage = "overlay-clip -a <INPUT> -b <OVERLAY> -p <POSITION> [OPTIONS]"
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// Bottom video (-a)
    #[arg(short = 'a', required = true)]
    input: String,

    /// Overlay video (-b)
    #[arg(short = 'b', required = true)]
    overlay: String,

    /// Time to start the overlay (e.g., 5 or 00:00:05)
    #[arg(short = 'p', required = true)]
    position: String,

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

    // 1. Validate inputs
    if !Path::new(&args.input).exists() || !Path::new(&args.overlay).exists() {
        eprintln!("! error: one or both input files not found.");
        std::process::exit(1);
    }

    // 2. Logic and Naming
    let info = get_media_info(&args.input);
    let fg_info = get_media_info(&args.overlay);
    let start_secs = parse_to_seconds(&args.position);
    let timestamp = format_time_for_filename(&format_seconds_ms(start_secs));

    let out_path = args.outfile.unwrap_or_else(|| {
        format!("{}-overlay-{}-[{}].mp4", info.stem, fg_info.stem, timestamp)
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

    // 4. Filter Construction
    // setpts delays the foreground, overlay=eof_action=pass keeps bg after fg ends
    let filter = format!("[1:v]setpts=PTS+{}/TB[fg]; [0:v][fg]overlay=eof_action=pass", start_secs);

    // 5. EXECUTE FFMPEG (Unified Vec)
    let mut cmd = Command::new("ffmpeg");
    
    let mut ffmpeg_args = vec![
        "-hide_banner",
        "-v", "error",
        "-stats",
        "-i", &args.input,
        "-i", &args.overlay,
        "-filter_complex", &filter,
        "-c:v", v_codec,
    ];

    // Append encoder params
    ffmpeg_args.extend(v_params);

    // Finalize with audio and output
    ffmpeg_args.extend(vec![
        "-c:a", "aac",           // Transcode to ensure sync and compatibility
        "-pix_fmt", "yuv420p",
        "-movflags", "+faststart",
        &out_path,
    ]);

    let status = cmd.args(ffmpeg_args)
        .status()
        .expect("failed to execute ffmpeg");

    if !status.success() {
        eprintln!("! error: ffmpeg failed to process overlay-clip.");
        std::process::exit(1);
    }
}

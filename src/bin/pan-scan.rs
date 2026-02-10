//==============================================================================
// pan-scan
// Description: Create a pan animation using scale and crop math from shell script
// References: [LIB-01], [LIB-03], [LIB-04], [LIB-09] [LIB-10] [LIB-11]
//==============================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path;
use ffmpeg_rust_scripts::{get_media_info, format_seconds_ms, parse_to_seconds, format_time_for_filename, hardware_encoding};

#[derive(Parser, Debug)]
#[command(
    author, 
    version, 
    about = "Pan scan over an image using scale/crop math",
    after_help = "Example:\n  pan-scan -i photo.jpg -d 00:00:10 -p l\n\nDependencies:\n  ffmpeg: https://www.ffmpeg.org/",
    override_usage = "pan-scan [OPTIONS] -i <INFILE> -d <DURATION> -p <POSITION>"
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// Input image file
    #[arg(short = 'i', required = true)]
    infile: String,

    /// Duration (e.g., 10 or 00:00:10)
    #[arg(short = 'd', required = true)]
    duration: String,

    /// Position: l (left), r (right), u (up), d (down)
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

fn get_image_dimensions(path: &str) -> (u32, u32) {
    let output = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-select_streams", "v:0",
            "-show_entries", "stream=width,height",
            "-of", "csv=s=x:p=0",
            path,
        ])
        .output()
        .expect("Failed to execute ffprobe");
    
    let res = String::from_utf8_lossy(&output.stdout);
    let dims: Vec<u32> = res.trim().split('x').filter_map(|s| s.parse().ok()).collect();
    if dims.len() == 2 { (dims[0], dims[1]) } else { (1920, 1080) }
}

fn main() {
    let args = Args::parse();

    if !Path::new(&args.infile).exists() {
        eprintln!("! error: input image '{}' not found.", args.infile);
        std::process::exit(1);
    }

    let (iw, ih) = get_image_dimensions(&args.infile);
    let dur = parse_to_seconds(&args.duration);
    let dur_str = dur.to_string(); // Variable to hold lifetime for the command
    let info = get_media_info(&args.infile);
    
    // 1. Enable Milliseconds in filename
    let full_ts = format_seconds_ms(dur);
    let timestamp = format_time_for_filename(&full_ts);
    
    let pos_full = match args.position.as_str() {
        "l" => "left", "r" => "right", "u" => "up", "d" => "down",
        _ => &args.position,
    };

    let out_path = args.outfile.unwrap_or_else(|| {
        format!("{}-pan-{}-[{}].mp4", info.stem, pos_full, timestamp)
    });

    // 2. Filter logic (original math preserved)
    let filter = match args.position.as_str() {
        "l" => format!("scale=w=-2:h=3*{},crop=w=3*{}/1.05:h=3*{}/1.05:x=t*(in_w-out_w)/{}:y=(in_h-out_h)/2,scale=w={}:h={},setsar=1", ih, iw, ih, dur, iw, ih),
        "r" => format!("scale=w=-2:h=3*{},crop=w=3*{}/1.05:h=3*{}/1.05:x=(in_w-out_w)-t*(in_w-out_w)/{}:y=(in_h-out_h)/2,scale=w={}:h={},setsar=1", ih, iw, ih, dur, iw, ih),
        "u" => format!("scale=w=-2:h=3*{},crop=w=3*{}/1.2:h=3*{}/1.2:x=(in_w-out_w)/2:y=t*(in_h-out_h)/{},scale=w={}:h={},setsar=1", ih, iw, ih, dur, iw, ih),
        "d" => format!("scale=w=-2:h=3*{},crop=w=3*{}/1.2:h=3*{}/1.2:x=(in_w-out_w)/2:y=(in_h-out_h)-t*(in_h-out_h)/{},scale=w={}:h={},setsar=1", ih, iw, ih, dur, iw, ih),
        _ => {
            eprintln!("! error: use l, r, u, or d for position.");
            std::process::exit(1);
        }
    };

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
        "-vf", &filter,
        "-t", &dur_str,
        "-c:v", v_codec,
    ];

    ffmpeg_args.extend(v_params);

    ffmpeg_args.extend(vec![
        "-pix_fmt", "yuv420p",
        "-movflags", "+faststart",
        &out_path, // Removed -y for safety
    ]);

    let status = cmd.args(ffmpeg_args)
        .status()
        .expect("failed to execute ffmpeg");

    if !status.success() {
        eprintln!("! error: ffmpeg failed to create pan-scan animation.");
        std::process::exit(1);
    }
}

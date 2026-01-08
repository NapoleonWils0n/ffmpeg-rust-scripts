//==============================================================================
// waveform
// Description: Create a waveform image from a video or audio file
// References: [LIB-01], [LIB-03]
//==============================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path;
use ffmpeg_rust_scripts::get_media_info; 

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "create a waveform image from a video or audio file",
    after_help = "Example:\n  waveform -i input.mp4 -c orange -j jpg\n\nColors: https://ffmpeg.org/ffmpeg-utils.html#Color\n\nDependencies:\n  ffmpeg: https://www.ffmpeg.org/\n\n",
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// input file
    #[arg(short = 'i', required = true, value_name = "INPUT")]
    infile: String,

    /// waveform color
    #[arg(short = 'c', default_value = "white", value_name = "COLOR")]
    color: String,

    /// output width
    #[arg(short = 'w', default_value = "1280", value_name = "WIDTH")]
    width: i32,

    /// output height
    #[arg(short = 'e', default_value = "420", value_name = "HEIGHT")]
    height: i32,

    /// image format jpg or png
    #[arg(short = 'j', default_value = "jpg", value_name = "FORMAT")]
    format: String,

    /// output file optional
    #[arg(short = 'o', value_name = "OUTFILE")]
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

    if !Path::new(&args.infile).exists() {
        eprintln!("Error: Input file '{}' not found.", args.infile);
        std::process::exit(1);
    }

    let info = get_media_info(&args.infile);
    let ext = args.format.to_lowercase();
    
    let out = args.outfile.clone().unwrap_or_else(|| {
        format!("{}-waveform.{}", info.stem, ext)
    });

    let out_path = format!("./{}", out);

    let filter = format!(
        "showwavespic=s={}x{}:colors={}", 
        args.width, args.height, args.color
    );

    let status = Command::new("ffmpeg")
        .args([
            "-hide_banner", 
            "-v", "error", 
            "-stats",
            "-i", &args.infile,
            "-filter_complex", &filter,
            "-frames:v", "1",
            "-y",
            &out_path,
        ])
        .status()
        .expect("Failed to execute FFmpeg");

    if !status.success() {
        std::process::exit(1);
    }
}

//==============================================================================
// ebu-meter
// Description: Display real-time EBU R128 audio loudness levels using ffplay
// References: [LIB-01]
//==============================================================================

use clap::Parser;
use std::process::Command;
// [LIB-01] Path import used for file existence check
use std::path::Path;

#[derive(Parser, Debug)]
#[command(
    author, 
    version, 
    about = "display EBU R128 audio loudness meter",
    after_help = "Example:\n  ebu-meter -i input.mp4 -t -16\n\nDependencies:\n  ffplay: https://www.ffmpeg.org/",
    override_usage = "ebu-meter [OPTIONS] -i <INFILE>"
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// input file
    #[arg(short = 'i', help = "input file")]
    infile: String,

    /// target audio level (LUFS) - e.g., -16
    #[arg(short = 't', default_value = "-16", help = "audio target level")]
    target: String,

    /// Print help
    #[arg(short = 'h', long = "help", action = clap::ArgAction::Help)]
    help: Option<bool>,

    /// Print version
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    version: Option<bool>,
}

fn main() {
    let args = Args::parse();

    let path = Path::new(&args.infile);

    // [LIB-01] Ensure the source file exists before processing
    if !path.exists() {
        eprintln!("Error: Input file '{}' not found.", args.infile);
        std::process::exit(1);
    }

    // Handle pathing: absolute paths are used as-is, relative paths get ./ for safety
    let path_str = if path.is_absolute() {
        args.infile.clone()
    } else {
        format!("./{}", args.infile)
    };

    // amovie needs escaped paths for its internal filter parser
    let escaped_path = path_str.replace('\\', "\\\\").replace(':', "\\:");

    // Construct the lavfi graph string using amovie to load the file
    // ebur128 generates both video [out0] and audio [out1]
    let filter = format!(
        "amovie='{}',ebur128=video=1:meter=18:dualmono=true:target={}[out0][out1]", 
        escaped_path, args.target
    );

    let status = Command::new("ffplay")
        // Inherit environment variables from shell (allows manual SDL_VIDEODRIVER usage)
        .envs(std::env::vars())
        .args([
            "-hide_banner",
            "-v", "error",
            "-window_title", &format!("EBU Meter: {}", args.infile),
            "-f", "lavfi",
            "-i", &filter,
        ])
        .status()
        .expect("Failed to execute ffplay");

    if !status.success() {
        eprintln!("ffplay process exited with an error.");
    }
}

//==============================================================================
// scopes
// Description: Display video with professional scopes stacked below
//==============================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "display video scopes",
    after_help = "Example:\n  scopes -i input.mkv\n\nDependencies:\n  ffplay: https://www.ffmpeg.org/",
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// histogram
    #[arg(short = 'i', action = clap::ArgAction::SetTrue)]
    histogram: bool,
    /// rgb overlay
    #[arg(short = 'o', action = clap::ArgAction::SetTrue)]
    overlay: bool,
    /// rgb parade
    #[arg(short = 'p', action = clap::ArgAction::SetTrue)]
    parade: bool,
    /// rgb overlay and parade
    #[arg(short = 's', action = clap::ArgAction::SetTrue)]
    both: bool,
    /// waveform
    #[arg(short = 'w', action = clap::ArgAction::SetTrue)]
    waveform: bool,
    /// vectorscope
    #[arg(short = 'v', action = clap::ArgAction::SetTrue)]
    vectorscope: bool,
    /// input file
    #[arg(required = true)]
    infile: String,
    /// Print help
    #[arg(short = 'h', action = clap::ArgAction::Help)]
    help: Option<bool>,
}

fn main() {
    let args = Args::parse();

    let path = Path::new(&args.infile);
    if !path.exists() {
        eprintln!("Error: Input file '{}' not found.", args.infile);
        std::process::exit(1);
    }

    let input_path = if path.is_absolute() {
        args.infile.clone()
    } else {
        format!("./{}", args.infile)
    };

    let filter = if args.histogram {
        // Restored your original working histogram logic
        "split=2[v1][v2];[v2]histogram=display_mode=parade[hist];[hist]scale=640:256,setsar=1[scope];[v1][scope]vstack"
    } else if args.overlay {
        // Robust RGB Overlay:
        // 1. Forced horizontal (d=0) and numeric graticule (g=1).
        // 2. Manual plane extraction and lutrgb tinting to fix green/sideways/bw issues.
        "split=2[v1][v2];\
         [v2]format=rgb24,extractplanes=r+g+b[r][g][b];\
         [r]waveform=d=0:g=1[rw];\
         [g]waveform=d=0:g=0[gw];\
         [b]waveform=d=0:g=0[bw];\
         [rw]lutrgb=g=0:b=0[rc];\
         [gw]lutrgb=r=0:b=0[gc];\
         [bw]lutrgb=r=0:g=0[bc];\
         [rc][gc]blend=all_mode=addition[rg];\
         [rg][bc]blend=all_mode=addition[rgb_scope];\
         [rgb_scope]scale=640:256,setsar=1[scope];\
         [v1][scope]vstack"
    } else if args.parade {
  // FIXED RGB PARADE:
        // 1. m=1: Numeric value for parade mode.
        // 2. d=0: Force horizontal orientation.
        // 3. g=1: Numeric graticule preset.
        // 4. c=7: Bitmask to enable Red(1) + Green(2) + Blue(4) = 7.
        "split=2[v1][v2];[v2]format=rgb24,waveform=m=1:d=0:g=1:c=7[p];[p]scale=640:256,setsar=1[scope];[v1][scope]vstack"
    } else if args.both {
        "split=2[v1][v2];[v2]waveform=m=parade,waveform=m=overlay,format=rgba,colorchannelmixer=aa=0.5[scope];[v1][scope]overlay"
    } else if args.waveform {
        "split=2[v1][v2];[v2]format=rgb24,waveform=d=0:g=1[w];[w]scale=640:256,setsar=1[scope];[v1][scope]vstack"
    } else if args.vectorscope {
        // Restored your original working vectorscope logic
        "split=2[v1][v2];[v2]vectorscope=m=color:i=1.0[vsc];[vsc]scale=640:256,setsar=1[scope];[v1][scope]vstack"
    } else {
        "split=2[v1][v2];[v2]histogram=display_mode=parade[hist];[hist]scale=640:256,setsar=1[scope];[v1][scope]vstack"
    };

    let status = Command::new("ffplay")
        .envs(std::env::vars())
        .args([
            "-hide_banner",
            "-v", "error",
            "-window_title", &format!("Scopes: {}", args.infile),
            "-i", &input_path,
            "-vf", filter,
        ])
        .status()
        .expect("Failed to execute ffplay");

    if !status.success() {
        eprintln!("ffplay process exited with an error.");
    }
}

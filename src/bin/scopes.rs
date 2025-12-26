//==============================================================================
// scopes
// Description: Display video with professional scopes stacked below
// References: [LIB-01] std::path::Path for file system validation
//==============================================================================

use clap::Parser;
use std::process::Command;
// [LIB-01] Import Path for reliable cross-platform file existence checks
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

    // Use [LIB-01] to verify the input file exists before launching ffplay
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
        // [FIX]: Restored original histogram logic
        "split=2[v1][v2];[v2]histogram=display_mode=parade[hist];[hist]scale=640:256,setsar=1[scope];[v1][scope]vstack"
    } else if args.overlay {
        // [FIX]: Robust RGB Overlay using manual plane extraction.
        // This avoids orientation and color parsing errors by building the 
        // overlay trace from individual R, G, and B planes
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
        // [FIX]: RGB Parade using numeric constants (m=1, c=7) for compatibility.
        // d=0 ensures horizontal orientation
        "split=2[v1][v2];[v2]format=rgb24,waveform=m=1:d=0:g=1:c=7[p];[p]scale=640:256,setsar=1[scope];[v1][scope]vstack"
    } else if args.both {
        // [FIX]: Vertically stacked view of both Parade and Overlay.
        // Reuses the stable c=7 parade logic and the manual extraction overlay logic
        // for maximum reliability
        "split=2[v1][v2];[v2]split=2[v_p][v_o];\
         [v_p]format=rgb24,waveform=m=1:d=0:g=1:c=7,scale=640:256,setsar=1[p];\
         [v_o]format=rgb24,extractplanes=r+g+b[r][g][b];\
         [r]waveform=d=0:g=1[rw];[g]waveform=d=0:g=0[gw];[b]waveform=d=0:g=0[bw];\
         [rw]lutrgb=g=0:b=0[rc];[gw]lutrgb=r=0:b=0[gc];[bw]lutrgb=r=0:g=0[bc];\
         [rc][gc]blend=all_mode=addition[rg];[rg][bc]blend=all_mode=addition[ov];\
         [ov]scale=640:256,setsar=1[o];\
         [v1][p]vstack[top];[top][o]vstack"
    } else if args.waveform {
        // [FIX]: Luma Waveform using format=gray to ensure a white brightness trace.
        // d=0 and g=1 ensure proper orientation and graticules
        "split=2[v1][v2];[v2]format=gray,waveform=d=0:g=1[w];[w]scale=640:256,setsar=1[scope];[v1][scope]vstack"
    } else if args.vectorscope {
        // Restored your original working vectorscope logic
        "split=2[v1][v2];[v2]vectorscope=m=color:i=1.0[vsc];[vsc]scale=640:256,setsar=1[scope];[v1][scope]vstack"
    } else {
        // Default to Histogram if no specific scope is selected
        "split=2[v1][v2];[v2]histogram=display_mode=parade[hist];[hist]scale=640:256,setsar=1[scope];[v1][scope]vstack"
    };

    let status = Command::new("ffplay")
        .envs(std::env::vars())
        .args([
            "-hide_banner",
            "-v", "fatal", // [FIX]: Use fatal log level to suppress Opus header errors
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

//==============================================================================
// scopes
// Description: Display video with professional scopes stacked below
// References: [LIB-01] std::path::Path for file system validation
//==============================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(
    author, 
    version, 
    about = "Display video with professional scopes stacked below",
    after_help = "Example:\n  scopes -w input.mp4\n\nDependencies:\n  ffplay: https://www.ffmpeg.org/\n\n",
    override_usage = "scopes [OPTIONS] <INPUT>"
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// Display Histogram
    #[arg(short = 'i', action = clap::ArgAction::SetTrue)]
    histogram: bool,

    /// Display RGB Overlay
    #[arg(short = 'o', action = clap::ArgAction::SetTrue)]
    overlay: bool,

    /// Display RGB Parade
    #[arg(short = 'p', action = clap::ArgAction::SetTrue)]
    parade: bool,

    /// Display RGB Overlay and Parade
    #[arg(short = 's', action = clap::ArgAction::SetTrue)]
    both: bool,

    /// Display Waveform
    #[arg(short = 'w', action = clap::ArgAction::SetTrue)]
    waveform: bool,

    /// Display Vectorscope
    #[arg(short = 'v', action = clap::ArgAction::SetTrue)]
    vectorscope: bool,

    /// Input file
    #[arg(required = true, value_name = "INPUT")]
    infile: String,

    /// Print help
    #[arg(short = 'h', long = "help", action = clap::ArgAction::Help)]
    help: Option<bool>,

    /// Print version
    #[arg(short = 'V', long = "version", action = clap::ArgAction::Version)]
    version: Option<bool>,
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

    // [FIX]: scale2ref is used to force the scope [v2] to match the width of the video [v1].
    // This solves the 'width 768 does not match 1920' error by ensuring the bottom 
    // input to vstack is always scaled to the top input's width.
    let filter = if args.histogram {
        "split=2[v1][v2];[v2]histogram=display_mode=parade[h];[h][v1]scale2ref=iw:256[scope][main];[main][scope]vstack"
    } else if args.overlay {
        "split=2[v1][v2];[v2]format=rgb24,extractplanes=r+g+b[r][g][b];\
         [r]waveform=d=0:g=1[rw];[g]waveform=d=0:g=0[gw];[b]waveform=d=0:g=0[bw];\
         [rw]lutrgb=g=0:b=0[rc];[gw]lutrgb=r=0:b=0[gc];[bw]lutrgb=r=0:g=0[bc];\
         [rc][gc]blend=all_mode=addition[rg];[rg][bc]blend=all_mode=addition[ov];\
         [ov][v1]scale2ref=iw:256[scope][main];[main][scope]vstack"
    } else if args.parade {
        "split=2[v1][v2];[v2]format=rgb24,waveform=m=1:d=0:g=1:c=7[p];\
         [p][v1]scale2ref=iw:256[scope][main];[main][scope]vstack"
    } else if args.both {
        "split=2[v1][v2];[v2]split=2[v_p][v_o];\
         [v_p]format=rgb24,waveform=m=1:d=0:g=1:c=7[p_raw];\
         [v_o]format=rgb24,extractplanes=r+g+b[r][g][b];\
         [r]waveform=d=0:g=1[rw];[g]waveform=d=0:g=0[gw];[b]waveform=d=0:g=0[bw];\
         [rw]lutrgb=g=0:b=0[rc];[gw]lutrgb=r=0:b=0[gc];[bw]lutrgb=r=0:g=0[bc];\
         [rc][gc]blend=all_mode=addition[rg];[rg][bc]blend=all_mode=addition[o_raw];\
         [p_raw][v1]scale2ref=iw:256[p][v_ref1];\
         [o_raw][v_ref1]scale2ref=iw:256[o][main];\
         [main][p]vstack[top];[top][o]vstack"
    } else if args.waveform {
        "split=2[v1][v2];[v2]format=gray,waveform=d=0:g=1[w];\
         [w][v1]scale2ref=iw:256[scope][main];[main][scope]vstack"
    } else if args.vectorscope {
        "split=2[v1][v2];[v2]vectorscope=m=color:i=1.0[vsc];\
         [vsc][v1]scale2ref=iw:256[scope][main];[main][scope]vstack"
    } else {
        // Default to Histogram
        "split=2[v1][v2];[v2]histogram=display_mode=parade[h];\
         [h][v1]scale2ref=iw:256[scope][main];[main][scope]vstack"
    };

    let status = Command::new("ffplay")
        .envs(std::env::vars()) // Inherit Wayland/SDL variables
        .args([
            "-hide_banner",
            "-loglevel", "error",
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

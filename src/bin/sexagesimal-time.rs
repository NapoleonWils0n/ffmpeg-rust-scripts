//==============================================================================
// sexagesimal-time
// Description: Calculate the duration between two sexagesimal timestamps
// References: [LIB-04], [LIB-05], [LIB-07]
//==============================================================================

use clap::Parser;
// [LIB-07] and [LIB-05] are directly called in main()
use ffmpeg_rust_scripts::{calculate_duration, format_seconds};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "calculate duration from start and end timecodes",
    after_help = "Example:\n  sexagesimal-time -s 00:01:00 -e 00:01:45.500\n\n\
    Output:\n  00:00:45.500\n\nDependencies:\n  None (Pure Rust math)",
    override_usage = "sexagesimal-time -s <START> -e <END>"
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// start time (HH:MM:SS.mmm)
    #[arg(short = 's', help = "start time")]
    start: String,

    /// end time (HH:MM:SS.mmm)
    #[arg(short = 'e', help = "end time")]
    end: String,

    /// Print help
    #[arg(short = 'h', long = "help", action = clap::ArgAction::Help)]
    help: Option<bool>,

    /// Print version
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    version: Option<bool>,
}

fn main() {
    let args = Args::parse();

    // Calculate duration using library logic [LIB-07]
    let duration_secs = calculate_duration(&args.start, &args.end);
    
    // Format the result to HH:MM:SS.mmm [LIB-05]
    let duration_sexagesimal = format_seconds(duration_secs);

    // Print only the sexagesimal result to match original script output
    println!("{}", duration_sexagesimal);
}

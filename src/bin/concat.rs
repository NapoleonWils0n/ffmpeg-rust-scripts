//==================================================================================
// concat
// Description: Concatenate files of the same type using the concat demuxer
// References: [LIB-01], [LIB-03]
//==================================================================================

use clap::Parser;
use std::process::Command;
use std::fs::File;
use std::io::Write;
use std::fs;
use std::path::Path;
use ffmpeg_rust_scripts::{get_media_info};

#[derive(Parser, Debug)]
#[command(
    author, 
    version, 
    about = "Concatenate videos using the concat demuxer",
    after_help = "Example:\n  concat -i input-1.mp4 input-2.mp4 input-3.mp4 -o output.mp4\n\n \
                  Note: The input files must be exactly the same type (codec, resolution, and frame rate).\n\n \
                  Dependencies:\n \
                  ffmpeg, ffplay: https://www.ffmpeg.org"
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// Input clips to concatenate
    #[arg(short = 'i', required = true, num_args = 2..)]
    inputs: Vec<String>,

    /// Output file (defaults to first_input-concat.ext)
    #[arg(short = 'o')]
    output: Option<String>,

    /// Preview concatenation with ffplay without saving
    #[arg(short = 'p')]
    preview: bool,

    /// Print help information
    #[arg(short = 'h', long = "help", action = clap::ArgAction::Help)]
    help: Option<bool>,

    /// Print version information
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    version: Option<bool>,
}

fn main() {
    let args = Args::parse();

    // 1. Verify all input files exist
    for f in &args.inputs {
        if !Path::new(f).exists() {
            eprintln!("! error: File '{}' not found.", f);
            std::process::exit(1);
        }
    }

    // 2. Determine output name
    let output_file = match &args.output {
        Some(o) => o.clone(),
        None => {
            let info = get_media_info(&args.inputs[0]);
            format!("{}-concat.{}", info.stem, info.extension)
        }
    };

    // 3. Create the temporary list file for FFmpeg demuxer
    let list_filename = "concat_list.txt";
    let mut file = File::create(list_filename).expect("! error: Could not create temp list file");
    
    for input in &args.inputs {
        // We use absolute paths or ensured relative paths for the demuxer
        writeln!(file, "file '{}'", input).expect("! error: Could not write to temp list file");
    }

    // 4. Decide between Preview (ffplay) or Record (ffmpeg)
    if args.preview {
        println!("+ previewing concatenation with ffplay...");
        let status = Command::new("ffplay")
            .args(["-hide_banner",
                   "-v", "error",
                   "-stats",
                   "-f", "concat",
                   "-safe", "0",
                   "-i", list_filename])
            .status()
            .expect("Failed to execute ffplay");

        if !status.success() {
            eprintln!("! error: ffplay exited with an error.");
        }
    } else {
        println!("+ recording to {}...", output_file);
        let status = Command::new("ffmpeg")
            .args([
                "-hide_banner", "-v",
                "error", "-stats",
                "-f", "concat",
                "-safe", "0",
                "-i", list_filename,
                "-c", "copy", // Stream copy (no re-encoding)
                &output_file
            ])
            .status()
            .expect("Failed to execute ffmpeg");

        if !status.success() {
            eprintln!("! error: ffmpeg concatenation failed.");
        }
    }

    // 5. Cleanup: Delete the temporary file
    if let Err(e) = fs::remove_file(list_filename) {
        eprintln!("! Warning: Could not delete temporary file {}: {}", list_filename, e);
    }
}

//==================================================================================
// lut-apply
// Description: Apply a color-graded Hald CLUT to a video file
// References: [LIB-01], [LIB-03], [LIB-08], [LIB-11]
//==================================================================================

use clap::Parser;
use std::process::Command;
use std::path::Path;
use ffmpeg_rust_scripts::{get_media_info, hardware_encoding};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Apply a color-graded Hald CLUT to a video file",
    after_help = "Example:\n  lut-apply -i input.mp4 -l lut.png -o output.mp4\n\nDependencies:\n  ffmpeg, ffplay: https://www.ffmpeg.org"
)]
#[clap(disable_version_flag = true, disable_help_flag = true)]
struct Args {
    /// Input video file
    #[arg(short = 'i', required = true, help = "input video file")]
    infile: String,

    /// Corrected LUT image
    #[arg(short = 'l', required = true, help = "corrected haldclut image")]
    lutfile: String,

    /// Optional output video file
    #[arg(short = 'o', help = "optional output file")]
    outfile: Option<String>,

    /// Preview with ffplay
    #[arg(short = 'p', help = "preview with ffplay")]
    preview: bool,

    /// Print help
    #[arg(short = 'h', long = "help", action = clap::ArgAction::Help)]
    help: Option<bool>,

    /// Print version
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    version: Option<bool>,
}

/// Helper to get image width using ffprobe
fn get_image_width(path: &str) -> u32 {
    let output = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-select_streams", "v:0",
            "-show_entries", "stream=width",
            "-of", "csv=p=0",
            path,
        ])
        .output()
        .expect("Failed to execute ffprobe");

    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<u32>()
        .unwrap_or(0)
}

/// Helper to get audio codec using ffprobe (as seen in blur-fill)
fn get_audio_codec(path: &str) -> String {
    let output = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-select_streams", "a:0",
            "-show_entries", "stream=codec_name",
            "-of", "csv=p=0",
            path,
        ])
        .output()
        .expect("Failed to execute ffprobe");

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn main() {
    let args = Args::parse();

    // 1. Verify files exist
    if !Path::new(&args.infile).exists() {
        eprintln!("! error: File '{}' not found.", args.infile);
        std::process::exit(1);
    }
    if !Path::new(&args.lutfile).exists() {
        eprintln!("! error: LUT file '{}' not found.", args.lutfile);
        std::process::exit(1);
    }

    let info = get_media_info(&args.infile);

    // 2. Determine output name
    let output_file = match &args.outfile {
        Some(o) => o.clone(),
        None => format!("{}-lut-applied.mp4", info.stem),
    };

    // 3. Check LUT width for conditional filtering
    let lut_width = get_image_width(&args.lutfile);

    // 4. ffplay Preview Logic
    if args.preview {
        println!("+ previewing color grade with ffplay...");
        
        let filter = if lut_width > 512 {
            // Composite image: crop left half
            format!("movie='{}',crop=iw/2:ih:0:0,[in]haldclut", args.lutfile)
        } else {
            // Standalone LUT: use directly
            format!("movie='{}', [in] haldclut", args.lutfile)
        };

        let ffplay_args = vec![
            "-hide_banner", "-v", "error", "-stats",
            "-i", &args.infile,
            "-vf", &filter,
        ];

        let status = Command::new("ffplay")
            .args(&ffplay_args)
            .status()
            .expect("Failed to execute ffplay");

        if !status.success() {
            eprintln!("! error: ffplay exited with an error.");
        }
        return;
    }

    // 5. Hardware/Software Encoding Logic
    let (v_codec, v_params) = if hardware_encoding() {
        println!("+ using hardware acceleration.");
        (
            "hevc_nvenc",
            vec![
                "-tune", "hq", "-preset", "p7", "-rc", "vbr",
                "-multipass", "fullres", "-rc-lookahead", "32",
                "-spatial-aq", "1", "-cq", "20", "-b:v", "0",
            ],
        )
    } else {
        println!("+ using software encoding.");
        ("libx264", vec!["-crf", "18", "-preset", "slow"])
    };

    // 6. Audio Logic (check if AAC)
    let audio_codec = get_audio_codec(&args.infile);
    let a_params = if audio_codec == "aac" {
        println!("+ audio is aac: using stream copy.");
        "copy"
    } else {
        println!("+ audio is {}: transcoding to aac.", audio_codec);
        "aac"
    };

    // 7. Final FFmpeg Command
    let filter_complex = if lut_width > 512 {
        format!("movie='{}',crop=iw/2:ih:0:0[lut];[0:v][lut]haldclut", args.lutfile)
    } else {
        format!("movie='{}'[lut];[0:v][lut]haldclut", args.lutfile)
    };

    let mut ffmpeg_args = vec![
        "-hide_banner", "-v", "error", "-stats",
        "-i", &args.infile,
        "-filter_complex", &filter_complex,
        "-c:v", v_codec,
    ];
    
    ffmpeg_args.extend(v_params);

    ffmpeg_args.extend_from_slice(&[
        "-c:a", a_params,
        "-pix_fmt", "yuv420p",
        "-movflags", "+faststart",
        &output_file,
    ]);

    let status = Command::new("ffmpeg")
        .args(&ffmpeg_args)
        .status()
        .expect("Failed to execute ffmpeg");

    if status.success() {
        println!("+ color grading applied: {}", output_file);
    } else {
        eprintln!("! error: ffmpeg failed.");
        std::process::exit(1);
    }
}

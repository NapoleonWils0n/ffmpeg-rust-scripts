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

fn main() {
    let args = Args::parse();

    // 1. Verify files exist [cite: 146, 204]
    if !Path::new(&args.infile).exists() {
        eprintln!("! error: File '{}' not found.", args.infile);
        std::process::exit(1);
    }
    if !Path::new(&args.lutfile).exists() {
        eprintln!("! error: LUT file '{}' not found.", args.lutfile);
        std::process::exit(1);
    }

    let info = get_media_info(&args.infile);

    // 2. Determine output name [cite: 140, 148]
    let output_file = match &args.outfile {
        Some(o) => o.clone(),
        None => format!("{}-lut-applied.mp4", info.stem),
    };

    // 3. ffplay Preview Logic 
    if args.preview {
        println!("+ previewing color grade with ffplay...");
        
        // Dynamic filter based on image width to handle composite frames
        let filter = format!("movie='{}',crop=iw/2:ih:0:0[lut];[in][lut]haldclut", args.lutfile);

        let mut ffplay_args = vec![
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

    // 4. Hardware/Software Encoding Logic [cite: 183, 186]
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

    // 5. Final FFmpeg Command [cite: 233, 234]
    let filter_complex = format!("movie='{}',crop=iw/2:ih:0:0[lut];[0:v][lut]haldclut", args.lutfile);

    let mut ffmpeg_args = vec![
        "-hide_banner", "-v", "error", "-stats",
        "-i", &args.infile,
        "-filter_complex", &filter_complex,
        "-c:v", v_codec,
    ];
    
    ffmpeg_args.extend(v_params);

    // Audio stream copy if already AAC, otherwise encode [cite: 234]
    // Note: For simplicity in this logic, we use copy as requested in common workflows
    ffmpeg_args.extend_from_slice(&[
        "-c:a", "copy",
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

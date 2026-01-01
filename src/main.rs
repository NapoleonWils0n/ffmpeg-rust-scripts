fn main() {
    println!("NapoleonWils0n FFmpeg Rust Tools");
    println!("===============================");
    println!("A collection of Rust-based utilities for automated video and audio processing.");
    println!("\nUsage: cargo run --bin <name> -- [args]");
    
    println!("\nAvailable Binaries:");

    println!("\n--- Scene Detection & Cutting ---");
    println!("  - scene-detect-auto   : Automated detection and splitting in one command");
    println!("  - scene-detect        : Detect scene changes and output timestamps");
    println!("  - scene-time          : Convert detection timestamps to a cutlist");
    println!("  - scene-cut           : Split video based on a (start, duration) cutlist");
    println!("  - scene-cut-to        : Split video based on a (start, end) cutlist");
    println!("  - scene-images        : Extract representative images from scenes");

    println!("\n--- Trimming & Clipping ---");
    println!("  - trim-clip           : Trim a clip using start and duration");
    println!("  - trim-clip-to        : Trim a clip using start and end points");
    println!("  - trim-short          : Quick trim for short segments");
    println!("  - trim-remote-clip    : Trim clips from remote URLs");
    println!("  - clip-time           : Calculate durations for a list of timestamps");
    println!("  - combine-clips       : Concatenate multiple video files");

    println!("\n--- Chapters & Metadata ---");
    println!("  - chapter-extract     : Pull chapter metadata from a file");
    println!("  - chapter-csv         : Convert chapter info to CSV format");
    println!("  - chapter-add         : Embed chapters into a video file");
    println!("  - subtitle-add        : Burn or mux subtitles into video");

    println!("\n--- Filters & Overlays ---");
    println!("  - overlay-pip         : Create Picture-in-Picture effects");
    println!("  - overlay-clip        : Overlay one clip onto another");
    println!("  - pan-scan            : Create pan and scan movements");
    println!("  - zoompan             : Apply zoom and pan effects");
    println!("  - xfade               : Apply crossfade transitions between clips");
    println!("  - fade-clip           : Add fade-in/out transitions");

    println!("\n--- Visualization & Analysis ---");
    println!("  - waveform            : Generate a waveform image from audio");
    println!("  - scopes              : Generate video scopes (vectorscope, waveform)");
    println!("  - ebu-meter           : Measure R128 loudness levels");
    println!("  - contact-sheet       : Create a thumbnail contact sheet");

    println!("\n--- Conversion & Extras ---");
    println!("  - audio-silence       : Detect or strip silence from audio");
    println!("  - extract-frame       : Extract a single high-quality frame");
    println!("  - img2video           : Create a video from a sequence of images");
    println!("  - vid2gif             : Convert video segments to high-quality GIFs");
    println!("  - webp                : Convert images or video to WebP");
    println!("  - sexagesimal-time    : Utility for time format conversions");
    
    println!("\nRun a script with '-h' for specific options, e.g.:");
    println!("  cargo run --bin scene-detect-auto -- -h\n");
}

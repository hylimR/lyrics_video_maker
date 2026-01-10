use std::env;

fn main() {
    // Standard Tauri build
    tauri_build::build();

    // FFmpeg sidecar - download FFmpeg binaries during build
    // This will automatically download the appropriate FFmpeg for the target platform
    #[cfg(feature = "download-ffmpeg")]
    {
        use ffmpeg_sidecar::download::auto_download;
        if let Err(e) = auto_download() {
            eprintln!("Warning: Failed to download FFmpeg: {}", e);
            eprintln!("FFmpeg will need to be installed manually or available in PATH");
        }
    }

    // Set the OUT_DIR for finding binaries
    println!("cargo:rerun-if-changed=build.rs");
    
    // On Windows, include the FFmpeg directory in the path
    if env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        println!("cargo:rustc-link-search=native=bin");
    }
}

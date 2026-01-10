//! Video Encoder
//!
//! Handles frame-by-frame video encoding using FFmpeg via ffmpeg-sidecar.

use serde::{Deserialize, Serialize};
use std::io::Write;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

/// Video codec options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VideoCodec {
    /// H.264/AVC - Most compatible
    H264,
    /// H.265/HEVC - Better compression, less compatible
    H265,
    /// VP9 - WebM format
    VP9,
    /// AV1 - Best compression, slow encoding
    AV1,
}

impl Default for VideoCodec {
    fn default() -> Self {
        Self::H264
    }
}

/// Video encoder configuration
#[derive(Debug, Clone)]
pub struct EncoderConfig {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub codec: VideoCodec,
    /// Quality (CRF for H.264/H.265, 0-51, lower is better)
    pub quality: u32,
    /// Bitrate in kbps (if using CBR mode)
    pub bitrate: Option<u32>,
    /// Keyframe interval
    pub keyframe_interval: u32,
    /// Pixel format (usually "yuv420p" for compatibility)
    pub pixel_format: String,
    /// Use hardware acceleration if available
    pub use_hw_accel: bool,
}

impl Default for EncoderConfig {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            fps: 30,
            codec: VideoCodec::H264,
            quality: 23, // Reasonable quality
            bitrate: None,
            keyframe_interval: 30, // One keyframe per second
            pixel_format: "yuv420p".to_string(),
            use_hw_accel: false,
        }
    }
}

/// Video encoder (FFmpeg wrapper)
pub struct VideoEncoder {
    config: EncoderConfig,
    output_path: String,
    frame_count: AtomicU32,
    total_frames: u32,
    process: Option<Child>,
    stdin: Option<ChildStdin>,
    cancelled: Arc<AtomicBool>,
}

/// Encoder errors
#[derive(Debug, thiserror::Error)]
pub enum EncoderError {
    #[error("FFmpeg not found. Please install FFmpeg or ensure it's in PATH.")]
    FfmpegNotFound,
    #[error("Failed to start encoder: {0}")]
    StartError(String),
    #[error("Failed to encode frame: {0}")]
    EncodeError(String),
    #[error("Failed to finalize video: {0}")]
    FinalizeError(String),
    #[error("Encoding cancelled")]
    Cancelled,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl VideoEncoder {
    /// Create a new video encoder
    ///
    /// This will start an FFmpeg process that accepts raw frames via stdin.
    pub fn new(output_path: &str, config: EncoderConfig, total_frames: u32) -> Result<Self, EncoderError> {
        // Build FFmpeg arguments
        let args = build_ffmpeg_args(&config, output_path);
        
        log::info!(
            "Starting FFmpeg encoder: {}x{} @ {} fps, codec: {:?}",
            config.width, config.height, config.fps, config.codec
        );
        log::debug!("FFmpeg args: {:?}", args);

        // Try to find FFmpeg
        let ffmpeg_path = find_ffmpeg().ok_or(EncoderError::FfmpegNotFound)?;

        // Start FFmpeg process
        let mut process = Command::new(&ffmpeg_path)
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| EncoderError::StartError(format!("Failed to start FFmpeg: {}", e)))?;

        let stdin = process.stdin.take();

        Ok(Self {
            config,
            output_path: output_path.to_string(),
            frame_count: AtomicU32::new(0),
            total_frames,
            process: Some(process),
            stdin,
            cancelled: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Get a cancellation handle
    pub fn get_cancel_handle(&self) -> Arc<AtomicBool> {
        self.cancelled.clone()
    }

    /// Check if encoding was cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }

    /// Cancel the encoding process
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    /// Submit a frame for encoding
    ///
    /// The frame should be RGBA pixel data (width * height * 4 bytes)
    pub fn submit_frame(&mut self, rgba_data: &[u8]) -> Result<(), EncoderError> {
        if self.is_cancelled() {
            return Err(EncoderError::Cancelled);
        }

        // Validate frame size
        let expected_size = (self.config.width * self.config.height * 4) as usize;
        if rgba_data.len() != expected_size {
            return Err(EncoderError::EncodeError(format!(
                "Invalid frame size: expected {}, got {}",
                expected_size, rgba_data.len()
            )));
        }

        // Write to FFmpeg stdin
        if let Some(ref mut stdin) = self.stdin {
            stdin.write_all(rgba_data)
                .map_err(|e| EncoderError::EncodeError(format!("Failed to write frame: {}", e)))?;
        } else {
            return Err(EncoderError::EncodeError("FFmpeg stdin not available".to_string()));
        }

        self.frame_count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Finalize encoding and close the output file
    pub fn finalize(mut self) -> Result<String, EncoderError> {
        // Close stdin to signal EOF to FFmpeg
        drop(self.stdin.take());

        // Wait for FFmpeg process to complete
        if let Some(process) = self.process.take() {
            let output = process.wait_with_output()
                .map_err(|e| EncoderError::FinalizeError(format!("Failed to wait for FFmpeg: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                log::error!("FFmpeg failed: {}", stderr);
                return Err(EncoderError::FinalizeError(format!("FFmpeg exited with error: {}", stderr)));
            }
        }

        let frame_count = self.frame_count.load(Ordering::Relaxed);
        let output_path = self.output_path.clone();
        log::info!(
            "VideoEncoder finalized: {} frames written to {}",
            frame_count, output_path
        );

        Ok(output_path)
    }

    /// Get current frame count
    pub fn frame_count(&self) -> u32 {
        self.frame_count.load(Ordering::Relaxed)
    }

    /// Get total frames to encode
    pub fn total_frames(&self) -> u32 {
        self.total_frames
    }

    /// Get progress as a percentage (0.0 - 1.0)
    pub fn progress(&self) -> f32 {
        if self.total_frames == 0 {
            return 0.0;
        }
        self.frame_count.load(Ordering::Relaxed) as f32 / self.total_frames as f32
    }

    /// Check if FFmpeg is available on the system
    pub fn check_ffmpeg() -> bool {
        find_ffmpeg().is_some()
    }

    /// Get FFmpeg version string
    pub fn get_ffmpeg_version() -> Option<String> {
        let ffmpeg = find_ffmpeg()?;
        Command::new(&ffmpeg)
            .arg("-version")
            .output()
            .ok()
            .and_then(|output| {
                String::from_utf8(output.stdout)
                    .ok()
                    .and_then(|s| s.lines().next().map(|l| l.to_string()))
            })
    }
}

impl Drop for VideoEncoder {
    fn drop(&mut self) {
        // Ensure process is cleaned up
        if let Some(mut process) = self.process.take() {
            let _ = process.kill();
            let _ = process.wait();
        }
    }
}

/// Find FFmpeg executable
/// Priority: 1. Bundled with app, 2. ffmpeg-sidecar location, 3. System PATH, 4. Common paths
fn find_ffmpeg() -> Option<String> {
    // 1. Check for bundled FFmpeg (in the app's resource directory)
    if let Some(exe_path) = std::env::current_exe().ok() {
        let exe_dir = exe_path.parent().unwrap_or(std::path::Path::new("."));
        
        #[cfg(target_os = "windows")]
        let bundled = exe_dir.join("ffmpeg.exe");
        #[cfg(not(target_os = "windows"))]
        let bundled = exe_dir.join("ffmpeg");
        
        if bundled.exists() {
            return Some(bundled.to_string_lossy().to_string());
        }

        // Check in resources subdirectory (Tauri bundled resources)
        #[cfg(target_os = "windows")]
        let bundled_res = exe_dir.join("resources").join("ffmpeg.exe");
        #[cfg(not(target_os = "windows"))]
        let bundled_res = exe_dir.join("resources").join("ffmpeg");
        
        if bundled_res.exists() {
            return Some(bundled_res.to_string_lossy().to_string());
        }
    }

    // 2. Check ffmpeg-sidecar default location
    #[cfg(target_os = "windows")]
    {
        if let Some(home) = dirs::home_dir() {
            let sidecar_path = home.join(".ffmpeg-sidecar").join("ffmpeg.exe");
            if sidecar_path.exists() {
                return Some(sidecar_path.to_string_lossy().to_string());
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Some(home) = dirs::home_dir() {
            let sidecar_path = home.join(".ffmpeg-sidecar").join("ffmpeg");
            if sidecar_path.exists() {
                return Some(sidecar_path.to_string_lossy().to_string());
            }
        }
    }

    // 3. Check if ffmpeg is in PATH
    if Command::new("ffmpeg")
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        return Some("ffmpeg".to_string());
    }

    // 4. Try common installation paths on Windows
    #[cfg(target_os = "windows")]
    {
        let common_paths = [
            "C:\\ffmpeg\\bin\\ffmpeg.exe",
            "C:\\Program Files\\ffmpeg\\bin\\ffmpeg.exe",
            "C:\\tools\\ffmpeg\\bin\\ffmpeg.exe",
            "D:\\ffmpeg\\bin\\ffmpeg.exe",
        ];
        for path in common_paths {
            if std::path::Path::new(path).exists() {
                return Some(path.to_string());
            }
        }
    }

    // Try Homebrew path on macOS
    #[cfg(target_os = "macos")]
    {
        let homebrew_paths = [
            "/opt/homebrew/bin/ffmpeg",
            "/usr/local/bin/ffmpeg",
        ];
        for path in homebrew_paths {
            if std::path::Path::new(path).exists() {
                return Some(path.to_string());
            }
        }
    }

    // Try common paths on Linux
    #[cfg(target_os = "linux")]
    {
        let linux_paths = [
            "/usr/bin/ffmpeg",
            "/usr/local/bin/ffmpeg",
        ];
        for path in linux_paths {
            if std::path::Path::new(path).exists() {
                return Some(path.to_string());
            }
        }
    }

    None
}

/// Build FFmpeg command arguments for encoding
fn build_ffmpeg_args(config: &EncoderConfig, output_path: &str) -> Vec<String> {
    let mut args = vec![
        "-y".to_string(), // Overwrite output
        "-f".to_string(), "rawvideo".to_string(),
        "-pix_fmt".to_string(), "rgba".to_string(),
        "-s".to_string(), format!("{}x{}", config.width, config.height),
        "-r".to_string(), config.fps.to_string(),
        "-i".to_string(), "-".to_string(), // Read from stdin
    ];

    // Hardware acceleration (NVENC for NVIDIA, VideoToolbox for macOS)
    if config.use_hw_accel {
        #[cfg(target_os = "windows")]
        {
            args.extend_from_slice(&[
                "-hwaccel".to_string(), "cuda".to_string(),
            ]);
        }
        #[cfg(target_os = "macos")]
        {
            args.extend_from_slice(&[
                "-hwaccel".to_string(), "videotoolbox".to_string(),
            ]);
        }
    }

    // Codec-specific options
    match config.codec {
        VideoCodec::H264 => {
            let encoder = if config.use_hw_accel {
                #[cfg(target_os = "windows")]
                { "h264_nvenc" }
                #[cfg(target_os = "macos")]
                { "h264_videotoolbox" }
                #[cfg(not(any(target_os = "windows", target_os = "macos")))]
                { "libx264" }
            } else {
                "libx264"
            };

            args.extend_from_slice(&[
                "-c:v".to_string(), encoder.to_string(),
            ]);

            // CRF or QP for quality
            if encoder == "libx264" {
                args.extend_from_slice(&[
                    "-crf".to_string(), config.quality.to_string(),
                    "-preset".to_string(), "medium".to_string(),
                ]);
            } else {
                // Hardware encoders use different quality parameters
                args.extend_from_slice(&[
                    "-q:v".to_string(), config.quality.to_string(),
                ]);
            }
        }
        VideoCodec::H265 => {
            let encoder = if config.use_hw_accel {
                #[cfg(target_os = "windows")]
                { "hevc_nvenc" }
                #[cfg(target_os = "macos")]
                { "hevc_videotoolbox" }
                #[cfg(not(any(target_os = "windows", target_os = "macos")))]
                { "libx265" }
            } else {
                "libx265"
            };

            args.extend_from_slice(&[
                "-c:v".to_string(), encoder.to_string(),
            ]);

            if encoder == "libx265" {
                args.extend_from_slice(&[
                    "-crf".to_string(), config.quality.to_string(),
                    "-preset".to_string(), "medium".to_string(),
                ]);
            } else {
                args.extend_from_slice(&[
                    "-q:v".to_string(), config.quality.to_string(),
                ]);
            }
        }
        VideoCodec::VP9 => {
            args.extend_from_slice(&[
                "-c:v".to_string(), "libvpx-vp9".to_string(),
                "-crf".to_string(), config.quality.to_string(),
                "-b:v".to_string(), "0".to_string(),
                "-row-mt".to_string(), "1".to_string(), // Enable row-based multithreading
            ]);
        }
        VideoCodec::AV1 => {
            args.extend_from_slice(&[
                "-c:v".to_string(), "libaom-av1".to_string(),
                "-crf".to_string(), config.quality.to_string(),
                "-cpu-used".to_string(), "4".to_string(), // Faster encoding
                "-row-mt".to_string(), "1".to_string(),
            ]);
        }
    }

    // Bitrate override if specified
    if let Some(bitrate) = config.bitrate {
        args.extend_from_slice(&[
            "-b:v".to_string(), format!("{}k", bitrate),
        ]);
    }

    // Output pixel format and keyframe interval
    args.extend_from_slice(&[
        "-pix_fmt".to_string(), config.pixel_format.clone(),
        "-g".to_string(), config.keyframe_interval.to_string(),
    ]);

    // Thread count (use all available cores)
    args.extend_from_slice(&[
        "-threads".to_string(), "0".to_string(),
    ]);

    // Output file
    args.push(output_path.to_string());

    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoder_config_default() {
        let config = EncoderConfig::default();
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
        assert_eq!(config.fps, 30);
    }

    #[test]
    fn test_build_ffmpeg_args() {
        let config = EncoderConfig::default();
        let args = build_ffmpeg_args(&config, "output.mp4");
        
        assert!(args.contains(&"-c:v".to_string()));
        assert!(args.contains(&"libx264".to_string()));
        assert!(args.contains(&"output.mp4".to_string()));
    }

    #[test]
    fn test_find_ffmpeg() {
        // This test might fail if FFmpeg is not installed
        let result = find_ffmpeg();
        println!("FFmpeg found: {:?}", result);
    }
}

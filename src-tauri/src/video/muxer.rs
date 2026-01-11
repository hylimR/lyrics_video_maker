//! Audio/Video Muxer
//!
//! Handles combining rendered video with audio tracks using FFmpeg.

use std::path::Path;
use std::process::{Command, Stdio};

/// Audio codec options
#[derive(Debug, Clone)]
pub enum AudioCodec {
    /// AAC - MP4 compatible
    AAC,
    /// Opus - WebM compatible
    Opus,
    /// MP3 - Wide compatibility
    MP3,
    /// Copy - Don't re-encode audio
    Copy,
}

impl Default for AudioCodec {
    fn default() -> Self {
        Self::AAC
    }
}

/// Muxer configuration
#[derive(Debug, Clone)]
pub struct MuxerConfig {
    pub audio_codec: AudioCodec,
    /// Audio bitrate in kbps
    pub audio_bitrate: u32,
    /// Audio offset in seconds (can be negative)
    pub audio_offset: f64,
    /// Fade in duration for audio (seconds)
    pub fade_in: f64,
    /// Fade out duration for audio (seconds)  
    pub fade_out: f64,
    /// Total duration for fade out calculation
    pub total_duration: Option<f64>,
}

impl Default for MuxerConfig {
    fn default() -> Self {
        Self {
            audio_codec: AudioCodec::AAC,
            audio_bitrate: 192,
            audio_offset: 0.0,
            fade_in: 0.0,
            fade_out: 0.0,
            total_duration: None,
        }
    }
}

/// Muxer errors
#[derive(Debug, thiserror::Error)]
pub enum MuxerError {
    #[error("FFmpeg not found")]
    FfmpegNotFound,
    #[error("Audio file not found: {0}")]
    AudioNotFound(String),
    #[error("Video file not found: {0}")]
    VideoNotFound(String),
    #[error("Muxing failed: {0}")]
    MuxError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Mux video and audio into a single output file
///
/// This uses FFmpeg to combine a video file with an audio file.
pub fn mux_audio_video(
    video_path: &str,
    audio_path: &str,
    output_path: &str,
    ffmpeg_path: Option<&str>,
    config: &MuxerConfig,
) -> Result<(), MuxerError> {
    // Verify input files exist
    if !Path::new(video_path).exists() {
        return Err(MuxerError::VideoNotFound(video_path.to_string()));
    }
    if !Path::new(audio_path).exists() {
        return Err(MuxerError::AudioNotFound(audio_path.to_string()));
    }

    // Build FFmpeg command
    let args = build_mux_args(video_path, audio_path, output_path, config);

    log::info!("Muxing: {} + {} -> {}", video_path, audio_path, output_path);
    log::debug!("FFmpeg args: {:?}", args);

    // Find FFmpeg
    let ffmpeg = if let Some(path) = ffmpeg_path {
        path.to_string()
    } else {
        crate::video::encoder::find_ffmpeg().ok_or(MuxerError::FfmpegNotFound)?
    };

    // Execute FFmpeg command
    let output = Command::new(&ffmpeg)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|_| MuxerError::FfmpegNotFound)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(MuxerError::MuxError(stderr.to_string()));
    }

    log::info!("Muxing complete: {}", output_path);
    Ok(())
}

/// Build FFmpeg arguments for muxing
fn build_mux_args(
    video_path: &str,
    audio_path: &str,
    output_path: &str,
    config: &MuxerConfig,
) -> Vec<String> {
    let mut args = vec![
        "-y".to_string(), // Overwrite output
        "-i".to_string(), video_path.to_string(),
    ];

    // Audio input with offset if needed
    if config.audio_offset > 0.0 {
        // Delay audio
        args.extend_from_slice(&[
            "-itsoffset".to_string(),
            format!("{:.3}", config.audio_offset),
        ]);
    }

    args.extend_from_slice(&[
        "-i".to_string(), audio_path.to_string(),
    ]);

    // Map streams
    args.extend_from_slice(&[
        "-map".to_string(), "0:v".to_string(), // Video from first input
        "-map".to_string(), "1:a".to_string(), // Audio from second input
        "-c:v".to_string(), "copy".to_string(), // Copy video (don't re-encode)
    ]);

    // Audio codec
    match config.audio_codec {
        AudioCodec::AAC => {
            args.extend_from_slice(&[
                "-c:a".to_string(), "aac".to_string(),
                "-b:a".to_string(), format!("{}k", config.audio_bitrate),
            ]);
        }
        AudioCodec::Opus => {
            args.extend_from_slice(&[
                "-c:a".to_string(), "libopus".to_string(),
                "-b:a".to_string(), format!("{}k", config.audio_bitrate),
            ]);
        }
        AudioCodec::MP3 => {
            args.extend_from_slice(&[
                "-c:a".to_string(), "libmp3lame".to_string(),
                "-b:a".to_string(), format!("{}k", config.audio_bitrate),
            ]);
        }
        AudioCodec::Copy => {
            args.extend_from_slice(&[
                "-c:a".to_string(), "copy".to_string(),
            ]);
        }
    }

    // Audio filters for fade in/out
    let mut audio_filters = Vec::new();
    if config.fade_in > 0.0 {
        audio_filters.push(format!("afade=t=in:st=0:d={:.2}", config.fade_in));
    }
    if config.fade_out > 0.0 {
        if let Some(duration) = config.total_duration {
            let fade_start = duration - config.fade_out;
            audio_filters.push(format!("afade=t=out:st={:.2}:d={:.2}", fade_start, config.fade_out));
        }
    }
    if !audio_filters.is_empty() {
        args.extend_from_slice(&[
            "-af".to_string(),
            audio_filters.join(","),
        ]);
    }

    // Shortest flag to match video/audio duration
    args.push("-shortest".to_string());

    // Output file
    args.push(output_path.to_string());

    args
}

/// Extract audio duration from a file using FFprobe
pub fn get_audio_duration(audio_path: &str) -> Result<f64, MuxerError> {
    if !Path::new(audio_path).exists() {
        return Err(MuxerError::AudioNotFound(audio_path.to_string()));
    }

    let output = Command::new("ffprobe")
        .args(&[
            "-v", "error",
            "-show_entries", "format=duration",
            "-of", "default=noprint_wrappers=1:nokey=1",
            audio_path,
        ])
        .output()
        .map_err(|_| MuxerError::FfmpegNotFound)?;

    if output.status.success() {
        let duration_str = String::from_utf8_lossy(&output.stdout);
        duration_str
            .trim()
            .parse::<f64>()
            .map_err(|_| MuxerError::MuxError("Failed to parse duration".to_string()))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(MuxerError::MuxError(format!("FFprobe failed: {}", stderr)))
    }
}

/// Extract video duration from a file using FFprobe
pub fn get_video_duration(video_path: &str) -> Result<f64, MuxerError> {
    if !Path::new(video_path).exists() {
        return Err(MuxerError::VideoNotFound(video_path.to_string()));
    }

    let output = Command::new("ffprobe")
        .args(&[
            "-v", "error",
            "-show_entries", "format=duration",
            "-of", "default=noprint_wrappers=1:nokey=1",
            video_path,
        ])
        .output()
        .map_err(|_| MuxerError::FfmpegNotFound)?;

    if output.status.success() {
        let duration_str = String::from_utf8_lossy(&output.stdout);
        duration_str
            .trim()
            .parse::<f64>()
            .map_err(|_| MuxerError::MuxError("Failed to parse duration".to_string()))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(MuxerError::MuxError(format!("FFprobe failed: {}", stderr)))
    }
}

/// Check if FFmpeg and FFprobe are available
pub fn check_ffmpeg_available() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_muxer_config_default() {
        let config = MuxerConfig::default();
        assert_eq!(config.audio_bitrate, 192);
        assert_eq!(config.audio_offset, 0.0);
    }

    #[test]
    fn test_build_mux_args() {
        let config = MuxerConfig::default();
        let args = build_mux_args("video.mp4", "audio.mp3", "output.mp4", &config);
        
        assert!(args.contains(&"video.mp4".to_string()));
        assert!(args.contains(&"audio.mp3".to_string()));
        assert!(args.contains(&"output.mp4".to_string()));
    }

    #[test]
    fn test_build_mux_args_with_fades() {
        let config = MuxerConfig {
            fade_in: 1.0,
            fade_out: 2.0,
            total_duration: Some(60.0),
            ..Default::default()
        };
        let args = build_mux_args("video.mp4", "audio.mp3", "output.mp4", &config);
        
        assert!(args.contains(&"-af".to_string()));
    }
}

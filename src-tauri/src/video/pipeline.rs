//! Video Rendering Pipeline
//! 
//! Shared logic for rendering video from KLyric documents, used by both
//! the Tauri command handler and the CLI tool.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use serde::{Deserialize, Serialize};

use klyric_renderer::model::KLyricDocumentV2;
use klyric_renderer::Renderer;
use crate::video::encoder::{VideoEncoder, EncoderConfig, VideoCodec, EncoderError};
use crate::video::muxer::{mux_audio_video, MuxerConfig, AudioCodec};

/// Render options
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderOptions {
    /// Output video width in pixels
    pub width: u32,
    /// Output video height in pixels
    pub height: u32,
    /// Frames per second
    pub fps: u32,
    /// Video quality (CRF 0-51, lower is better quality)
    pub quality: u32,
    /// Video codec ("h264", "h265", "vp9", "av1")
    pub codec: String,
    /// Audio offset in seconds (can be negative)
    pub audio_offset: f64,
    /// Use hardware acceleration if available
    #[serde(default)]
    pub use_hw_accel: bool,
    /// Custom duration to render in seconds (for preview)
    pub custom_duration: Option<f64>,
    /// Enable real-time preview during render (default: true)
    #[serde(default = "default_true")]
    pub enable_preview: bool,
}

fn default_true() -> bool {
    true
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            fps: 30,
            quality: 23,
            codec: "h264".to_string(),
            audio_offset: 0.0,
            use_hw_accel: false,
            custom_duration: None,
            enable_preview: true,
        }
    }
}

impl RenderOptions {
    pub fn to_encoder_config(&self) -> EncoderConfig {
        let codec = match self.codec.to_lowercase().as_str() {
            "h265" | "hevc" => VideoCodec::H265,
            "vp9" => VideoCodec::VP9,
            "av1" => VideoCodec::AV1,
            _ => VideoCodec::H264,
        };

        EncoderConfig {
            width: self.width,
            height: self.height,
            fps: self.fps,
            codec,
            quality: self.quality,
            bitrate: None,
            keyframe_interval: self.fps, // One keyframe per second
            pixel_format: "yuv420p".to_string(),
            use_hw_accel: self.use_hw_accel,
        }
    }
}

/// Progress information
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderProgress {
    /// Current frame being rendered
    pub current_frame: u32,
    /// Total number of frames
    pub total_frames: u32,
    /// Progress percentage (0-100)
    pub percentage: f32,
    /// Estimated time remaining in seconds
    pub eta_seconds: f32,
    /// Current phase of rendering
    pub phase: String,
    /// Frames per second rendering speed
    pub render_fps: f32,
}

/// Render result information
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderResult {
    /// Path to the output video file
    pub output_path: String,
    /// Total render time in seconds
    pub render_time: f32,
    /// Total frames rendered
    pub total_frames: u32,
    /// Average render FPS
    pub avg_fps: f32,
}

/// Pipeline errors
#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("Invalid KLyric document: {0}")]
    InvalidDocument(String),
    #[error("Render cancelled")]
    Cancelled,
    #[error("FFmpeg error: {0}")]
    FfmpegError(String),
    #[error("FFmpeg not found")]
    FfmpegNotFound,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Encoding error: {0}")]
    EncodingError(String),
}

impl From<EncoderError> for PipelineError {
    fn from(e: EncoderError) -> Self {
        match e {
            EncoderError::FfmpegNotFound => PipelineError::FfmpegNotFound,
            EncoderError::Cancelled => PipelineError::Cancelled,
            _ => PipelineError::EncodingError(e.to_string()),
        }
    }
}

/// Run the video render pipeline
pub fn run_render_pipeline<F, P>(
    document: KLyricDocumentV2,
    audio_path: Option<String>,
    output_path: String,
    options: RenderOptions,
    cancellation_token: Arc<AtomicBool>,
    progress_callback: F,
    preview_callback: Option<P>,
) -> Result<RenderResult, PipelineError>
where
    F: Fn(RenderProgress) + Send + 'static,
    P: Fn(Vec<u8>) + Send + 'static,
{
    // Check FFmpeg
    if !VideoEncoder::check_ffmpeg() {
        return Err(PipelineError::FfmpegNotFound);
    }

    // Calculate duration
    let project_duration = document.project.duration;
    let duration = if let Some(custom) = options.custom_duration {
        custom.min(project_duration)
    } else {
        project_duration
    };
    
    let fps = options.fps as f64;
    let total_frames = (duration * fps).ceil() as u32;

    println!("🎥 Pipeline: Starting render: {}x{} @ {} fps, {} frames", options.width, options.height, options.fps, total_frames);

    // Temp video path
    let temp_video_path = if audio_path.is_some() {
        let path = std::path::Path::new(&output_path);
        let stem = path.file_stem().unwrap_or_default().to_string_lossy();
        let ext = path.extension().unwrap_or_default().to_string_lossy();
        let parent = path.parent().unwrap_or(std::path::Path::new("."));
        parent.join(format!("{}_temp.{}", stem, ext)).to_string_lossy().to_string()
    } else {
        output_path.clone()
    };

    let encoder_config = options.to_encoder_config();
    let audio_offset = options.audio_offset;

    // Report init
    progress_callback(RenderProgress {
        current_frame: 0,
        total_frames,
        percentage: 0.0,
        eta_seconds: 0.0,
        phase: "Initializing encoder".to_string(),
        render_fps: 0.0,
    });

    // Initialize renderer & encoder
    let mut renderer = Renderer::new(encoder_config.width, encoder_config.height);

    // --- TODO: Implement a more robust system font discovery mechanism ---
    let font_candidates = [
        "../public/fonts/NotoSansSC-Regular.otf",
        "fonts/NotoSansSC-Regular.otf", // Potential production path relative to binary if resources are copied
        "./NotoSansSC-Regular.otf",
    ];

    let mut font_loaded = false;

    for path_str in &font_candidates {
        let path = std::path::Path::new(path_str);
        if let Ok(abs_path) = std::fs::canonicalize(path) {
             println!("🔎 Checking font path: {:?} (Exists)", abs_path);
             
             if let Ok(bytes) = std::fs::read(&abs_path) {
                println!("✅ Pipeline: Loading font from {:?} ({} bytes)", abs_path, bytes.len());
                
                // Load with standard name
                let _ = renderer.text_renderer_mut().load_font_bytes("Noto Sans SC", bytes.clone());
                // Load with CSS-style name (often used in web frontend)
                let _ = renderer.text_renderer_mut().load_font_bytes("NotoSansSC", bytes.clone());
                // Set as default fallback
                let _ = renderer.text_renderer_mut().set_default_font_bytes(bytes);
                
                font_loaded = true;
                break;
             }
        } else {
             println!("🔎 Checking font path: {:?} (Not found)", path);
        }
    }

    if !font_loaded {
        println!("⚠️ Pipeline: Default font file not found. Attempting system font fallback...");
        // Fallback to system fonts (Arial, Segoe UI, Roboto, etc.)
        let fallback_families = ["Arial", "Segoe UI", "Roboto", "Helvetica", "Sans-Serif"];
        for family in &fallback_families {
            if let Some(path) = klyric_renderer::text::TextRenderer::find_font_file(family) {
                 if let Ok(_) = renderer.text_renderer_mut().load_font("Noto Sans SC", &path.to_string_lossy()) {
                     println!("✅ Pipeline: Loaded system fallback font '{}' as 'Noto Sans SC'", family);
                     font_loaded = true;
                     break;
                 }
            }
        }
    }

    if !font_loaded {
        println!("❌ Pipeline: No suitable font found. Text may not render!");
    }

    let mut encoder = VideoEncoder::new(&temp_video_path, encoder_config, total_frames)?;

    let start_time = std::time::Instant::now();
    let mut last_progress_time = start_time;
    let mut last_preview_time = start_time;
    // Preview throttle: 15 FPS (approx 66ms)
    let preview_interval = std::time::Duration::from_millis(66); 

    // Render loop
    for frame_num in 0..total_frames {
        if cancellation_token.load(Ordering::SeqCst) {
            return Err(PipelineError::Cancelled);
        }

        let current_time = frame_num as f64 / fps;

        let frame_data = renderer.render_frame(&document, current_time)
            .map_err(|e| PipelineError::EncodingError(e.to_string()))?;
        
        // --- NEW: Preview Emission ---
        if options.enable_preview {
            // println!("Preview enabled");
            if let Some(ref cb) = preview_callback {
            let now = std::time::Instant::now();
            if now.duration_since(last_preview_time) >= preview_interval {
                // println!("Pipeline: Throttle passed");
                last_preview_time = now;

                // Clone data for processing (avoid holding up render thread too much? actually this is blocking)
                // We do it synchronously here.
                let width = options.width;
                let height = options.height;
                // skia render_frame returns bytes directly
                let data = &frame_data;

                // We need to convert premultiplied RGBA to RGBA for image crate?
                // Tiny-skia uses Premultiplied RGBA8888.
                // Image crate expects Rgba8.
                // We'll trust that for preview, un-premultiplying might be skipped or handled cheaply?
                // Actually tiny-skia has `encode_png` but we want efficient resizing.
                // Let's use `image` crate.
                
                // Construct ImageBuffer from raw data.
                if let Some(img) = image::RgbaImage::from_raw(width, height, data.to_vec()) {
                    // Resize to width 480 (maintain aspect)
                    let target_width = 480;
                    let ratio = height as f32 / width as f32;
                    let target_height = (target_width as f32 * ratio) as u32;

                    let resized = image::imageops::resize(&img, target_width, target_height, image::imageops::FilterType::Nearest);
                    
                    // Convert to RGB8 (drop alpha) because JPEG doesn't support RGBA
                    let rgb_image = image::DynamicImage::ImageRgba8(resized).to_rgb8();
                    
                    // Encode to JPEG using JpegEncoder
                    let mut buffer = Vec::new();
                    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, 70);

                     if let Ok(_) = encoder.encode(rgb_image.as_raw(), rgb_image.width(), rgb_image.height(), image::ColorType::Rgb8.into()) {
                         // println!("Pipeline: Encoding success, calling callback");
                         cb(buffer);
                     } else if let Err(e) = encoder.encode(rgb_image.as_raw(), rgb_image.width(), rgb_image.height(), image::ColorType::Rgb8.into()) {
                         println!("Pipeline: Encoding failed: {}", e);
                     }
                }
            }
        }
        }
        
        encoder.submit_frame(&frame_data)?; // Consume pixmap here

        // Progress update
        let now = std::time::Instant::now();
        if now.duration_since(last_progress_time).as_millis() >= 100 || frame_num == total_frames - 1 {
            last_progress_time = now;
            
            let percentage = (frame_num as f32 / total_frames as f32) * 100.0;
            let elapsed = start_time.elapsed().as_secs_f32();
            let render_fps = if elapsed > 0.0 { frame_num as f32 / elapsed } else { 0.0 };
            let eta = if frame_num > 0 {
                (elapsed / frame_num as f32) * (total_frames - frame_num) as f32
            } else {
                0.0
            };

            progress_callback(RenderProgress {
                current_frame: frame_num,
                total_frames,
                percentage,
                eta_seconds: eta,
                phase: "Rendering frames".to_string(),
                render_fps,
            });
        }
    }

    // Finalize
    progress_callback(RenderProgress {
        current_frame: total_frames,
        total_frames,
        percentage: 99.0,
        eta_seconds: 0.0,
        phase: "Finalizing video".to_string(),
        render_fps: 0.0,
    });

    let video_path = encoder.finalize()?;

    // Mux audio
    if let Some(audio) = audio_path {
        progress_callback(RenderProgress {
            current_frame: total_frames,
            total_frames,
            percentage: 99.5,
            eta_seconds: 0.0,
            phase: "Adding audio".to_string(),
            render_fps: 0.0,
        });

        let muxer_config = MuxerConfig {
            audio_codec: AudioCodec::AAC,
            audio_bitrate: 192,
            audio_offset,
            fade_in: 0.0,
            fade_out: 0.0,
            total_duration: Some(duration),
        };

        mux_audio_video(&video_path, &audio, &output_path, None, &muxer_config)
            .map_err(|e| PipelineError::FfmpegError(e.to_string()))?;
        
        let _ = std::fs::remove_file(&video_path);
    }

    let total_time = start_time.elapsed().as_secs_f32();
    let avg_fps = total_frames as f32 / total_time;

    progress_callback(RenderProgress {
        current_frame: total_frames,
        total_frames,
        percentage: 100.0,
        eta_seconds: 0.0,
        phase: "Complete".to_string(),
        render_fps: avg_fps,
    });

    Ok(RenderResult {
        output_path: output_path.to_string(),
        render_time: total_time,
        total_frames,
        avg_fps,
    })
}

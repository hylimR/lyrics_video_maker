//! Video Rendering Commands
//!
//! Handles video rendering requests from the frontend, including
//! progress tracking and cancellation support.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use tauri::{Emitter, Window};

use klyric_renderer::model::KLyricDocumentV2;
use crate::video::encoder::VideoEncoder; // Still need VideoEncoder for check_ffmpeg
use crate::video::pipeline::{run_render_pipeline, RenderOptions, RenderResult, PipelineError};

// Global render state for cancellation support
static RENDER_CANCELLED: AtomicBool = AtomicBool::new(false);
static RENDER_PROGRESS: AtomicU32 = AtomicU32::new(0);

/// Check if FFmpeg is available
#[tauri::command]
pub fn check_ffmpeg() -> Result<String, String> {
    if let Some(version) = VideoEncoder::get_ffmpeg_version() {
        Ok(version)
    } else {
        Err("FFmpeg not found".to_string())
    }
}

/// Map pipeline error to string for Tauri
impl From<PipelineError> for String {
    fn from(e: PipelineError) -> Self {
        e.to_string()
    }
}

/// Main video rendering command
///
/// This command is called from the frontend to start rendering a video.
/// It uses the shared render pipeline.
#[tauri::command]
pub async fn render_video(
    window: Window,
    klyric_json: String,
    audio_path: Option<String>,
    output_path: String,
    options: RenderOptions,
) -> Result<RenderResult, String> {
    println!("🎥 Rust: render_video command received from window: {}", window.label());

    // Reset cancellation flag
    RENDER_CANCELLED.store(false, Ordering::SeqCst);
    RENDER_PROGRESS.store(0, Ordering::SeqCst);

    println!("🎥 Rust: JSON length: {}", klyric_json.len());
    if klyric_json.len() > 500 {
        println!("🎥 Rust: JSON preview: {}...", &klyric_json[0..500]);
    } else {
        println!("🎥 Rust: JSON content: {}", klyric_json);
    }

    // Parse the KLyric document (v2)
    let document: KLyricDocumentV2 = serde_json::from_str(&klyric_json)
        .map_err(|e| {
            println!("❌ Rust: Invalid KLyric document: {}", e);
            format!("Invalid KLyric document: {}", e)
        })?;

    // Clone values for the blocking task
    let window_clone = window.clone();
    let cancellation_token = Arc::new(AtomicBool::new(false)); // Use Arc for thread safety
    let token_clone = cancellation_token.clone();

    // Spawn a monitoring thread to sync cancellation state
    // (This is a bit of a hack to bridge the global static with the Arc)
    tauri::async_runtime::spawn(async move {
        loop {
            if RENDER_CANCELLED.load(Ordering::SeqCst) {
                token_clone.store(true, Ordering::SeqCst);
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    });

    // Run pipeline in blocking task
    let result = tauri::async_runtime::spawn_blocking(move || {
        let window_clone_preview = window_clone.clone();
        
        run_render_pipeline(
            document,
            audio_path,
            output_path,
            options,
            cancellation_token,
            move |progress| {
                // Update global progress
                // RENDER_PROGRESS.store(progress.current_frame, Ordering::Relaxed); // Global static, hard to access from here safely if modified?
                // Actually static is fine.

                // Emit event
                let _ = window_clone.emit("render-progress", progress);
            },
            Some(move |frame_data: Vec<u8>| {
                use base64::Engine as _;
                let b64 = base64::engine::general_purpose::STANDARD.encode(&frame_data);
                // println!("Rust: Emitting render-frame (len: {})", b64.len());
                if let Err(e) = window_clone_preview.emit("render-frame", b64) {
                    println!("❌ Rust: Failed to emit frame: {}", e);
                } else {
                    // println!("✅ Rust: Frame emitted");
                }
            }),
        )
    }).await.map_err(|e| format!("Task failed: {}", e))?;

    result.map_err(|e| e.to_string())
}

/// Cancel an ongoing render operation
#[tauri::command]
pub fn cancel_render() -> Result<(), String> {
    log::info!("Render cancellation requested");
    RENDER_CANCELLED.store(true, Ordering::SeqCst);
    Ok(())
}

/// Get current render progress
#[tauri::command]
pub fn get_render_progress() -> u32 {
    RENDER_PROGRESS.load(Ordering::SeqCst)
}

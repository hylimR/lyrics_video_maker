//! Video Rendering Commands
//!
//! Handles video rendering requests from the frontend, including
//! progress tracking and cancellation support.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{Emitter, State, Window};

use klyric_renderer::model::KLyricDocumentV2;
use crate::video::encoder::VideoEncoder; // Still need VideoEncoder for check_ffmpeg
use crate::video::pipeline::{run_render_pipeline, RenderOptions, RenderResult, PipelineError};

// Render state managed by Tauri
pub struct RenderState {
    pub cancellation_token: Mutex<Option<Arc<AtomicBool>>>,
    pub progress: Arc<AtomicU32>,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            cancellation_token: Mutex::new(None),
            progress: Arc::new(AtomicU32::new(0)),
        }
    }
}

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
    state: State<'_, RenderState>,
    klyric_json: String,
    audio_path: Option<String>,
    output_path: String,
    options: RenderOptions,
) -> Result<RenderResult, String> {
    println!("🎥 Rust: render_video command received from window: {}", window.label());

    // Reset progress
    state.progress.store(0, Ordering::SeqCst);

    // Create new cancellation token
    let cancellation_token = Arc::new(AtomicBool::new(false));

    // Store token in state
    {
        let mut token_guard = state.cancellation_token.lock().unwrap();
        *token_guard = Some(cancellation_token.clone());
    }

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
    let progress_tracker = state.progress.clone();

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
                progress_tracker.store(progress.current_frame, Ordering::Relaxed);

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
pub fn cancel_render(state: State<'_, RenderState>) -> Result<(), String> {
    log::info!("Render cancellation requested");
    if let Some(token) = &*state.cancellation_token.lock().unwrap() {
        token.store(true, Ordering::SeqCst);
    }
    Ok(())
}

/// Get current render progress
#[tauri::command]
pub fn get_render_progress(state: State<'_, RenderState>) -> u32 {
    state.progress.load(Ordering::SeqCst)
}

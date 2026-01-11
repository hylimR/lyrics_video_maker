//! Export Commands
//!
//! Handles file export operations including single frame export
//! and system font discovery.

use serde::{Serialize, Serializer};
use std::path::PathBuf;
use ttf_parser::Face;

use klyric_renderer::model::KLyricDocumentV2;
use klyric_renderer::Renderer;

/// Export error types
#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("Invalid KLyric document: {0}")]
    InvalidDocument(String),
    #[error("Failed to save file: {0}")]
    SaveError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Image encoding error: {0}")]
    ImageError(String),
    #[error("Render error: {0}")]
    RenderError(String),
}

impl Serialize for ExportError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Font information returned to frontend
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FontInfo {
    pub name: String,   // Full name e.g. "Arial Bold"
    pub family: String, // Family name e.g. "Arial"
    pub path: String,
    pub style: String,  // Style e.g. "Bold"
    pub supports_chinese: bool,
}

/// Export a single frame as an image
///
/// Useful for thumbnails or preview snapshots
#[tauri::command]
pub async fn export_frame(
    klyric_json: String,
    timestamp: f64,
    output_path: String,
    width: u32,
    height: u32,
) -> Result<String, ExportError> {
    // Parse the KLyric v2 document
    let document: KLyricDocumentV2 = serde_json::from_str(&klyric_json)
        .map_err(|e| ExportError::InvalidDocument(e.to_string()))?;

    // Create v2 renderer and render the frame
    let mut renderer = Renderer::new(width, height);
    let frame_data = renderer.render_frame(&document, timestamp)
        .map_err(|e| ExportError::RenderError(e.to_string()))?;

    // Save as PNG
    let path = PathBuf::from(&output_path);
    
    // Create image from raw RGBA data
    let img = image::RgbaImage::from_raw(width, height, frame_data)
        .ok_or_else(|| ExportError::ImageError("Failed to create image buffer".to_string()))?;
    
    img.save(&path)
        .map_err(|e| ExportError::ImageError(e.to_string()))?;

    log::info!("Exported frame at t={:.2}s to {:?}", timestamp, path);
    
    Ok(output_path)
}

/// Get list of system fonts available for rendering
///
/// This scans common font directories and returns available fonts
#[tauri::command]
pub fn get_system_fonts() -> Vec<FontInfo> {
    let mut fonts = Vec::new();

    // Common font directories by platform
    #[cfg(target_os = "windows")]
    let font_dirs = vec![
        PathBuf::from(std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string()))
            .join("Fonts"),
        dirs::font_dir().unwrap_or_else(|| PathBuf::from("C:\\Windows\\Fonts")),
        dirs::data_local_dir().map(|d| d.join("Microsoft\\Windows\\Fonts")).unwrap_or_default(),
    ];

    #[cfg(target_os = "macos")]
    let font_dirs = vec![
        PathBuf::from("/System/Library/Fonts"),
        PathBuf::from("/Library/Fonts"),
        dirs::home_dir()
            .map(|h| h.join("Library/Fonts"))
            .unwrap_or_else(|| PathBuf::from("/Library/Fonts")),
    ];

    #[cfg(target_os = "linux")]
    let font_dirs = vec![
        PathBuf::from("/usr/share/fonts"),
        PathBuf::from("/usr/local/share/fonts"),
        dirs::home_dir()
            .map(|h| h.join(".fonts"))
            .unwrap_or_else(|| PathBuf::from("/usr/share/fonts")),
    ];

    // Scan directories for font files
    for dir in font_dirs {
        log::info!("Scanning font directory: {:?}", dir);
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    let ext = ext.to_string_lossy().to_lowercase();
                    if ext == "ttf" || ext == "otf" || ext == "ttc" {
                        // Debug specific font
                        if path.to_string_lossy().to_lowercase().contains("dfhannotate") {
                            log::info!("Found target font candidate: {:?}", path);
                        }

                        // Read file and parse metadata
                        if let Ok(data) = std::fs::read(&path) {
                            // Helper to extract name
                            let extract_name = |face: &Face, id: u16| -> Option<String> {
                                face.names().into_iter()
                                    .find(|n| n.name_id == id && n.is_unicode())
                                    .and_then(|n| n.to_string())
                            };

                            // Iterate over faces (TTC might have multiple)
                            let count = ttf_parser::fonts_in_collection(&data).unwrap_or(1);
                            
                            for i in 0..count {
                                if let Ok(face) = Face::parse(&data, i) {
                                    // Try PostScript name (ID 6) if Family (ID 1) is missing/weird
                                    // Some fonts like DfHannotate might use different IDs or encoding issues
                                    let family = extract_name(&face, ttf_parser::name_id::FAMILY)
                                        .or_else(|| extract_name(&face, ttf_parser::name_id::TYPOGRAPHIC_FAMILY)) // ID 16
                                        .or_else(|| extract_name(&face, ttf_parser::name_id::POST_SCRIPT_NAME).map(|s| s.split('-').next().unwrap_or(&s).to_string())) // ID 6
                                        .or_else(|| path.file_stem().map(|s| s.to_string_lossy().to_string()))
                                        .unwrap_or_else(|| "Unknown".to_string());
                                        
                                    let style = extract_name(&face, ttf_parser::name_id::SUBFAMILY)
                                        .or_else(|| extract_name(&face, ttf_parser::name_id::TYPOGRAPHIC_SUBFAMILY)) // ID 17
                                        .unwrap_or_else(|| "Regular".to_string());
                                    
                                    let full_name = format!("{} {}", family, style).trim().to_string();

                                    // Check for Chinese support (CJK Unified Ideographs)
                                    // We check for '一' (One) or '永' (Forever) or '好' (Good)
                                    let supports_chinese = face.glyph_index('一').is_some() || face.glyph_index('永').is_some();

                                    if path.to_string_lossy().to_lowercase().contains("dfhannotate") {
                                        log::info!("Parsed DfHannotate: Family='{}', Style='{}', Name='{}', CJK={}", family, style, full_name, supports_chinese);
                                    }

                                    fonts.push(FontInfo {
                                        name: full_name,
                                        family,
                                        path: path.to_string_lossy().to_string(),
                                        style,
                                        supports_chinese,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Sort by name
    fonts.sort_by(|a, b| a.name.cmp(&b.name));
    fonts.dedup_by(|a, b| a.name == b.name);

    log::info!("Found {} system fonts", fonts.len());
    fonts
}

/// Download FFmpeg using ffmpeg-sidecar
///
/// This downloads the appropriate FFmpeg build for the current platform
#[tauri::command]
pub async fn download_ffmpeg() -> Result<String, ExportError> {
    use ffmpeg_sidecar::download::auto_download;
    
    log::info!("Downloading FFmpeg...");
    
    auto_download()
        .map_err(|e| ExportError::SaveError(format!("Failed to download FFmpeg: {}", e)))?;
    
    log::info!("FFmpeg downloaded successfully");
    
    // Return the path where FFmpeg was installed
    if let Some(home) = dirs::home_dir() {
        #[cfg(target_os = "windows")]
        let ffmpeg_path = home.join(".ffmpeg-sidecar").join("ffmpeg.exe");
        #[cfg(not(target_os = "windows"))]
        let ffmpeg_path = home.join(".ffmpeg-sidecar").join("ffmpeg");
        
        return Ok(ffmpeg_path.to_string_lossy().to_string());
    }
    
    Ok("FFmpeg installed".to_string())
}

/// Check if FFmpeg via sidecar is already downloaded
#[tauri::command]
pub fn check_ffmpeg_downloaded() -> bool {
    if let Some(home) = dirs::home_dir() {
        #[cfg(target_os = "windows")]
        let ffmpeg_path = home.join(".ffmpeg-sidecar").join("ffmpeg.exe");
        #[cfg(not(target_os = "windows"))]
        let ffmpeg_path = home.join(".ffmpeg-sidecar").join("ffmpeg");
        
        return ffmpeg_path.exists();
    }
    false
}

/// Read a font file and return its bytes
///
/// Used by the frontend to load system fonts into the WASM renderer
#[tauri::command]
pub async fn read_font_file(path: String) -> Result<Vec<u8>, String> {
    // Basic security check: ensure it looks like a font file
    let lower_path = path.to_lowercase();
    if !lower_path.ends_with(".ttf") 
        && !lower_path.ends_with(".otf") 
        && !lower_path.ends_with(".ttc") {
         return Err("Invalid font file extension".to_string());
    }
    
    // Read file asynchronously
    tokio::fs::read(&path).await.map_err(|e| format!("Failed to read font file: {}", e))
}

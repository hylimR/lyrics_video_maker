//! Text Rendering Utilities
//!
//! Provides text rendering capabilities using ab_glyph for font loading
//! and glyph rasterization.

use ab_glyph::{FontRef, Font, ScaleFont, PxScale};
use std::collections::HashMap;

#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

/// Text renderer with font caching
pub struct TextRenderer {
    /// Cached font data by font name/path
    font_cache: HashMap<String, Vec<u8>>,
    /// Default font data (embedded fallback)
    default_font: Option<Vec<u8>>,
}

impl TextRenderer {
    /// Create a new text renderer
    pub fn new() -> Self {
        Self {
            font_cache: HashMap::new(),
            default_font: None,
        }
    }

    /// Load a font from a file path (not available in WASM)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_font(&mut self, name: &str, path: &str) -> Result<(), TextRenderError> {
        let data = std::fs::read(path)
            .map_err(|e| TextRenderError::FontLoadError(format!("{}: {}", path, e)))?;
        
        // Validate font data
        let _ = FontRef::try_from_slice(&data)
            .map_err(|_| TextRenderError::InvalidFont(path.to_string()))?;
        
        self.font_cache.insert(name.to_string(), data);
        log::info!("Loaded font '{}' from {}", name, path);
        Ok(())
    }

    /// Load a font from raw bytes
    pub fn load_font_bytes(&mut self, name: &str, data: Vec<u8>) -> Result<(), TextRenderError> {
        // Validate font data
        let _ = FontRef::try_from_slice(&data)
            .map_err(|_| TextRenderError::InvalidFont(name.to_string()))?;
        
        self.font_cache.insert(name.to_string(), data);
        log::info!("Loaded font '{}' from memory", name);
        Ok(())
    }

    /// Set the default fallback font from bytes
    pub fn set_default_font_bytes(&mut self, data: Vec<u8>) -> Result<(), TextRenderError> {
        // Validate font data
        let _ = FontRef::try_from_slice(&data)
            .map_err(|_| TextRenderError::InvalidFont("Default font".to_string()))?;
        
        self.default_font = Some(data);
        log::info!("Set default fallback font");
        Ok(())
    }

    /// Get a font reference by name
    pub fn get_font(&self, name: &str) -> Option<FontRef<'_>> {
        self.font_cache
            .get(name)
            .and_then(|data| FontRef::try_from_slice(data).ok())
    }

    /// Get the default/fallback font
    pub fn get_default_font(&self) -> Option<FontRef<'_>> {
        self.default_font
            .as_ref()
            .and_then(|data| FontRef::try_from_slice(data).ok())
    }

    /// Measure text width at given scale
    pub fn measure_text(&self, font: &FontRef<'_>, text: &str, size: f32) -> f32 {
        let scaled = font.as_scaled(PxScale::from(size));
        
        let mut width = 0.0;
        let mut prev_glyph_id = None;
        
        for ch in text.chars() {
            let glyph_id = scaled.glyph_id(ch);
            
            if let Some(prev) = prev_glyph_id {
                width += scaled.kern(prev, glyph_id);
            }
            
            width += scaled.h_advance(glyph_id);
            prev_glyph_id = Some(glyph_id);
        }
        
        width
    }

    /// Measure single character
    pub fn measure_char(&self, font: &FontRef<'_>, ch: char, size: f32) -> (f32, f32) {
        let scaled = font.as_scaled(PxScale::from(size));
        let glyph_id = scaled.glyph_id(ch);
        
        let width = scaled.h_advance(glyph_id);
        let height = scaled.height();
        
        (width, height)
    }

    /// Rasterize a single character to grayscale pixels
    ///
    /// Returns (width, height, pixels) where pixels is a Vec<u8> of alpha values
    pub fn rasterize_char(
        &self,
        font: &FontRef<'_>,
        ch: char,
        size: f32,
    ) -> Option<(u32, u32, Vec<u8>)> {
        let scaled = font.as_scaled(PxScale::from(size));
        let glyph = scaled.scaled_glyph(ch);
        
        if let Some(outlined) = scaled.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            let width = bounds.width().ceil() as u32;
            let height = bounds.height().ceil() as u32;
            
            if width == 0 || height == 0 {
                return None;
            }
            
            let mut pixels = vec![0u8; (width * height) as usize];
            
            outlined.draw(|x, y, c| {
                let idx = (y * width + x) as usize;
                if idx < pixels.len() {
                    pixels[idx] = (c * 255.0) as u8;
                }
            });
            
            Some((width, height, pixels))
        } else {
            None
        }
    }

    /// Get system font directories (not available in WASM)
    #[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
    pub fn get_system_font_dirs() -> Vec<PathBuf> {
        vec![
            PathBuf::from(
                std::env::var("WINDIR")
                    .unwrap_or_else(|_| "C:\\Windows".to_string())
            ).join("Fonts"),
        ]
    }

    #[cfg(all(not(target_arch = "wasm32"), target_os = "macos"))]
    pub fn get_system_font_dirs() -> Vec<PathBuf> {
        vec![
            PathBuf::from("/System/Library/Fonts"),
            PathBuf::from("/Library/Fonts"),
        ]
    }

    #[cfg(all(not(target_arch = "wasm32"), target_os = "linux"))]
    pub fn get_system_font_dirs() -> Vec<PathBuf> {
        vec![
            PathBuf::from("/usr/share/fonts"),
            PathBuf::from("/usr/local/share/fonts"),
        ]
    }

    /// Find a font file by family name (not available in WASM)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn find_font_file(family: &str) -> Option<PathBuf> {
        let normalized = family.to_lowercase().replace(' ', "");
        
        for dir in Self::get_system_font_dirs() {
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(stem) = path.file_stem() {
                        let name = stem.to_string_lossy().to_lowercase().replace(' ', "");
                        if name.contains(&normalized) {
                            if let Some(ext) = path.extension() {
                                let ext = ext.to_string_lossy().to_lowercase();
                                if ext == "ttf" || ext == "otf" {
                                    return Some(path);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        None
    }
}

impl Default for TextRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Text rendering errors
#[derive(Debug, thiserror::Error)]
pub enum TextRenderError {
    #[error("Failed to load font: {0}")]
    FontLoadError(String),
    #[error("Invalid font file: {0}")]
    InvalidFont(String),
    #[error("Font not found: {0}")]
    FontNotFound(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_renderer_creation() {
        let renderer = TextRenderer::new();
        assert!(renderer.font_cache.is_empty());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_system_font_dirs() {
        let dirs = TextRenderer::get_system_font_dirs();
        assert!(!dirs.is_empty());
    }
}

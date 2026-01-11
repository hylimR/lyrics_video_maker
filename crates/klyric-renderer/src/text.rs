//! Text Rendering Utilities using Skia
//!

use skia_safe::{Font, Typeface, FontMgr, FontStyle, Data, Point, Path};
use std::collections::HashMap;

#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

/// Text renderer with font caching using Skia
pub struct TextRenderer {
    font_mgr: FontMgr,
    /// Cached font data by font name
    font_cache: HashMap<String, Typeface>,
    /// Default typeface
    default_typeface: Option<Typeface>,
}

impl TextRenderer {
    pub fn new() -> Self {
        Self {
            font_mgr: FontMgr::new(),
            font_cache: HashMap::new(),
            default_typeface: None,
        }
    }

    /// Load a font from a file path (Native only)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_font(&mut self, name: &str, path: &str) -> Result<(), TextRenderError> {
        let data = std::fs::read(path)
            .map_err(|e| TextRenderError::FontLoadError(format!("{}: {}", path, e)))?;
        
        // Create Data from bytes
        let sk_data = Data::new_copy(&data);
        
        let typeface = self.font_mgr.new_from_data(&sk_data, None)
            .ok_or_else(|| TextRenderError::InvalidFont(path.to_string()))?;
        
        self.font_cache.insert(name.to_string(), typeface);
        log::info!("Loaded font '{}' from {}", name, path);
        Ok(())
    }

    /// Load a font from raw bytes
    pub fn load_font_bytes(&mut self, name: &str, data: Vec<u8>) -> Result<(), TextRenderError> {
        let sk_data = Data::new_copy(&data);
        let typeface = self.font_mgr.new_from_data(&sk_data, None)
            .ok_or_else(|| TextRenderError::InvalidFont(name.to_string()))?;
        
        self.font_cache.insert(name.to_string(), typeface);
        log::info!("Loaded font '{}' from memory", name);
        Ok(())
    }

    /// Set the default fallback font
    pub fn set_default_font_bytes(&mut self, data: Vec<u8>) -> Result<(), TextRenderError> {
        let sk_data = Data::new_copy(&data);
        let typeface = self.font_mgr.new_from_data(&sk_data, None)
            .ok_or_else(|| TextRenderError::InvalidFont("Default font".to_string()))?;
        
        self.default_typeface = Some(typeface);
        log::info!("Set default fallback font");
        Ok(())
    }

    /// Get a typeface by name
    pub fn get_typeface(&self, name: &str) -> Option<Typeface> {
        // First check cache
        if let Some(tf) = self.font_cache.get(name) {
            return Some(tf.clone());
        }
        
        // Then try system fonts via FontMgr
        self.font_mgr.match_family_style(name, FontStyle::normal())
    }

    pub fn get_default_typeface(&self) -> Option<Typeface> {
        self.default_typeface.clone()
    }

    /// Measure single character
    pub fn measure_char(&self, typeface: &Typeface, ch: char, size: f32) -> (f32, f32) {
        let font = Font::from_typeface(typeface, size);
        
        // width
        let glyph_id = font.unichar_to_glyph(ch as i32);
        let width = font.measure_text(&ch.to_string(), None).0; // simple measure
        
        // height? 
        let metrics = font.metrics().1;
        let height = metrics.descent - metrics.ascent; // approximate height
        
        (width, height)
    }

    /// Get the vector path for a character
    pub fn get_glyph_path(
        &self,
        typeface: &Typeface,
        ch: char,
        size: f32,
    ) -> Option<Path> {
        let font = Font::from_typeface(typeface, size);
        let glyph_id = font.unichar_to_glyph(ch as i32);
        
        font.get_path(glyph_id)
    }
    
    // Legacy helper for system font dirs (can be removed if we trust Skia FontMgr)
    #[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
    pub fn get_system_font_dirs() -> Vec<PathBuf> {
        vec![
            PathBuf::from(std::env::var("WINDIR").unwrap_or("C:\\Windows".to_string())).join("Fonts"),
            PathBuf::from(std::env::var("LOCALAPPDATA").unwrap_or(".".to_string())).join("Microsoft\\Windows\\Fonts"),
        ]
    }
    
    #[cfg(all(not(target_arch = "wasm32"), target_os = "macos"))]
    pub fn get_system_font_dirs() -> Vec<PathBuf> {
        vec![PathBuf::from("/System/Library/Fonts"), PathBuf::from("/Library/Fonts")]
    }

    #[cfg(all(not(target_arch = "wasm32"), target_os = "linux"))]
    pub fn get_system_font_dirs() -> Vec<PathBuf> {
        vec![PathBuf::from("/usr/share/fonts"), PathBuf::from("/usr/local/share/fonts")]
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    pub fn find_font_file(family: &str) -> Option<PathBuf> {
         // Manual search fallback if needed, but Skia FontMgr is better.
         None 
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

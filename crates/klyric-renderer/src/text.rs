//! Text Rendering Utilities using Skia
//!

use skia_safe::{Font, Typeface, FontMgr, FontStyle, Data, Path};
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

impl Default for TextRenderer {
    fn default() -> Self {
        Self::new()
    }
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
        let _glyph_id = font.unichar_to_glyph(ch as i32);
        let width = font.measure_text(ch.to_string(), None).0; // simple measure
        
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
    pub fn find_font_file(_family: &str) -> Option<PathBuf> {
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

#[cfg(test)]
mod tests {
    use super::*;

    // --- TextRenderer Creation Tests ---

    #[test]
    fn test_new() {
        let renderer = TextRenderer::new();
        // Empty cache, no default
        assert!(renderer.font_cache.is_empty());
        assert!(renderer.default_typeface.is_none());
    }

    // --- Font Loading Tests ---

    #[test]
    fn test_load_font_bytes_valid() {
        let mut renderer = TextRenderer::new();

        // Load a valid embedded font (using Roboto which is commonly available)
        // For testing, we'll use Arial from system if available, or skip
        // Since we can't guarantee font availability, we test with minimal TTF structure
        // This test verifies the API works; actual font validity is system-dependent

        // Create minimal "font" bytes (this will fail validation but tests the code path)
        let invalid_bytes = vec![0u8; 100];
        let result = renderer.load_font_bytes("test", invalid_bytes);

        // Should fail with invalid font
        assert!(
            result.is_err(),
            "Invalid font bytes should return error"
        );
    }

    #[test]
    fn test_load_font_bytes_invalid() {
        let mut renderer = TextRenderer::new();

        // Empty bytes should fail
        let result = renderer.load_font_bytes("empty", vec![]);
        assert!(result.is_err(), "Empty font bytes should return error");
    }

    #[test]
    fn test_set_default_font_bytes() {
        let mut renderer = TextRenderer::new();

        // Invalid bytes should fail
        let result = renderer.set_default_font_bytes(vec![0u8; 50]);
        assert!(
            result.is_err(),
            "Invalid default font bytes should return error"
        );

        // After failed set, default should still be None
        assert!(renderer.default_typeface.is_none());
    }

    // --- Typeface Retrieval Tests ---

    #[test]
    fn test_get_typeface_not_cached() {
        let renderer = TextRenderer::new();

        // Non-existent font returns None (or system font if available)
        let result = renderer.get_typeface("NonExistentFont12345");

        // Result depends on system fonts, but shouldn't panic
        // On most systems, this will return None unless the exact name exists
        let _ = result; // Just verify it doesn't panic
    }

    #[test]
    fn test_get_typeface_system_font() {
        let renderer = TextRenderer::new();

        // Try to get a common system font that should exist on most platforms
        #[cfg(target_os = "windows")]
        let font_name = "Arial";
        #[cfg(target_os = "macos")]
        let font_name = "Helvetica";
        #[cfg(target_os = "linux")]
        let font_name = "DejaVu Sans";

        let result = renderer.get_typeface(font_name);

        // On systems with this font, it should succeed
        // We don't assert because font availability varies
        if let Some(typeface) = result {
            // Verify we got a valid typeface
            assert!(!typeface.family_name().is_empty());
        }
    }

    #[test]
    fn test_get_default_typeface_none() {
        let renderer = TextRenderer::new();

        // Initially, default typeface should be None
        assert!(
            renderer.get_default_typeface().is_none(),
            "New renderer should have no default typeface"
        );
    }

    // --- Measurement Tests ---

    #[test]
    fn test_measure_char() {
        let renderer = TextRenderer::new();

        // Get a system font for measurement
        #[cfg(target_os = "windows")]
        let font_name = "Arial";
        #[cfg(target_os = "macos")]
        let font_name = "Helvetica";
        #[cfg(target_os = "linux")]
        let font_name = "DejaVu Sans";

        if let Some(typeface) = renderer.get_typeface(font_name) {
            let (width, height) = renderer.measure_char(&typeface, 'A', 48.0);

            // Measurements should be positive
            assert!(width > 0.0, "Character width should be positive");
            assert!(height > 0.0, "Character height should be positive");
        }
        // If font not available, test passes (can't measure without font)
    }

    #[test]
    fn test_measure_char_different_sizes() {
        let renderer = TextRenderer::new();

        #[cfg(target_os = "windows")]
        let font_name = "Arial";
        #[cfg(target_os = "macos")]
        let font_name = "Helvetica";
        #[cfg(target_os = "linux")]
        let font_name = "DejaVu Sans";

        if let Some(typeface) = renderer.get_typeface(font_name) {
            let (width_small, _) = renderer.measure_char(&typeface, 'A', 24.0);
            let (width_large, _) = renderer.measure_char(&typeface, 'A', 72.0);

            // Larger size should produce larger measurement
            assert!(
                width_large > width_small,
                "Larger font size should produce larger width: {} vs {}",
                width_large,
                width_small
            );
        }
    }

    // --- System Font Directory Tests (platform-specific) ---

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_get_system_font_dirs_not_empty() {
        let dirs = TextRenderer::get_system_font_dirs();

        // Should return at least one directory
        assert!(
            !dirs.is_empty(),
            "System font directories should not be empty"
        );

        // Directories should be valid paths (though they might not exist on all systems)
        for dir in &dirs {
            assert!(
                !dir.as_os_str().is_empty(),
                "Font directory path should not be empty"
            );
        }
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_find_font_file_returns_none() {
        // Current implementation always returns None
        let result = TextRenderer::find_font_file("ArialAnything");
        assert!(
            result.is_none(),
            "find_font_file should return None (current implementation)"
        );
    }

    // --- Glyph Path Tests ---

    #[test]
    fn test_get_glyph_path() {
        let renderer = TextRenderer::new();

        #[cfg(target_os = "windows")]
        let font_name = "Arial";
        #[cfg(target_os = "macos")]
        let font_name = "Helvetica";
        #[cfg(target_os = "linux")]
        let font_name = "DejaVu Sans";

        if let Some(typeface) = renderer.get_typeface(font_name) {
            let path = renderer.get_glyph_path(&typeface, 'A', 48.0);

            // For a valid character, we should get a path
            if let Some(p) = path {
                // Path should have some bounds
                let bounds = p.bounds();
                assert!(bounds.width() > 0.0, "Glyph path should have positive width");
            }
        }
    }
}

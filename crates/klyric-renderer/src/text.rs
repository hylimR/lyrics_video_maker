//! Text Rendering Utilities using Skia
//!

use skia_safe::{Data, Font, FontMgr, FontStyle, Path, Typeface};
use std::collections::HashMap;

#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

/// Resolved font wrapper for optimized measurement
pub struct ResolvedFont {
    pub(crate) font: Font,
    pub(crate) height: f32,
}

/// Text renderer with font caching using Skia
pub struct TextRenderer {
    font_mgr: FontMgr,
    /// Cached font data by font name
    font_cache: HashMap<String, Typeface>,
    /// Cache for resolved fonts: (typeface_id, size_bits) -> Font
    resolved_font_cache: HashMap<(u32, u32), Font>,
    /// Cache for glyph paths: (typeface_id, size_bits, glyph_id) -> Path
    path_cache: HashMap<(u32, u32, u16), Path>,
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
            resolved_font_cache: HashMap::new(),
            path_cache: HashMap::new(),
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

        let typeface = self
            .font_mgr
            .new_from_data(&sk_data, None)
            .ok_or_else(|| TextRenderError::InvalidFont(path.to_string()))?;

        self.font_cache.insert(name.to_string(), typeface);
        log::info!("Loaded font '{}' from {}", name, path);
        Ok(())
    }

    /// Load a font from raw bytes
    pub fn load_font_bytes(&mut self, name: &str, data: Vec<u8>) -> Result<(), TextRenderError> {
        let sk_data = Data::new_copy(&data);
        let typeface = self
            .font_mgr
            .new_from_data(&sk_data, None)
            .ok_or_else(|| TextRenderError::InvalidFont(name.to_string()))?;

        self.font_cache.insert(name.to_string(), typeface);
        log::info!("Loaded font '{}' from memory", name);
        Ok(())
    }

    /// Set the default fallback font
    pub fn set_default_font_bytes(&mut self, data: Vec<u8>) -> Result<(), TextRenderError> {
        let sk_data = Data::new_copy(&data);
        let typeface = self
            .font_mgr
            .new_from_data(&sk_data, None)
            .ok_or_else(|| TextRenderError::InvalidFont("Default font".to_string()))?;

        self.default_typeface = Some(typeface);
        log::info!("Set default fallback font");
        Ok(())
    }

    /// Get a typeface by name
    pub fn get_typeface(&mut self, name: &str) -> Option<Typeface> {
        // First check cache
        if let Some(tf) = self.font_cache.get(name) {
            return Some(tf.clone());
        }

        // Then try system fonts via FontMgr
        if let Some(tf) = self.font_mgr.match_family_style(name, FontStyle::normal()) {
            self.font_cache.insert(name.to_string(), tf.clone());
            return Some(tf);
        }

        None
    }

    pub fn get_default_typeface(&self) -> Option<Typeface> {
        self.default_typeface.clone()
    }

    /// Get a resolved font from cache or create it
    pub fn get_font(&mut self, typeface: &Typeface, size: f32) -> Font {
        let key = (typeface.unique_id().into(), size.to_bits());
        if let Some(font) = self.resolved_font_cache.get(&key) {
            return font.clone();
        }

        let font = Font::from_typeface(typeface.clone(), size);
        self.resolved_font_cache.insert(key, font.clone());
        font
    }

    /// Clear the resolved font cache to free memory (e.g. at end of frame)
    pub fn clear_font_cache(&mut self) {
        // Only clear if we have too many entries to prevent memory leak
        // while preserving cache for static text
        const MAX_CACHE_SIZE: usize = 500;
        if self.resolved_font_cache.len() > MAX_CACHE_SIZE {
            self.resolved_font_cache.clear();
        }

        const MAX_PATH_CACHE_SIZE: usize = 5000;
        if self.path_cache.len() > MAX_PATH_CACHE_SIZE {
            self.path_cache.clear();
        }
    }

    /// Create a resolved font for optimized measurement
    pub fn create_font(&self, typeface: &Typeface, size: f32) -> ResolvedFont {
        let font = Font::from_typeface(typeface, size);
        let metrics = font.metrics().1;
        let height = metrics.descent - metrics.ascent;
        ResolvedFont { font, height }
    }

    /// Measure single character using a resolved font
    pub fn measure_char_with_font(
        &self,
        resolved_font: &ResolvedFont,
        ch: char,
    ) -> (f32, f32, u16) {
        let font = &resolved_font.font;

        // width
        let glyph_id = font.unichar_to_glyph(ch as i32);
        let mut width = [0.0];
        font.get_widths(&[glyph_id], &mut width);

        (width[0], resolved_font.height, glyph_id)
    }

    /// Measure single character
    pub fn measure_char(&self, typeface: &Typeface, ch: char, size: f32) -> (f32, f32, u16) {
        let resolved_font = self.create_font(typeface, size);
        self.measure_char_with_font(&resolved_font, ch)
    }

    /// Get the vector path for a character
    pub fn get_glyph_path(&self, typeface: &Typeface, ch: char, size: f32) -> Option<Path> {
        let font = Font::from_typeface(typeface, size);
        let glyph_id = font.unichar_to_glyph(ch as i32);

        font.get_path(glyph_id)
    }

    /// Get a cached path for a glyph
    pub fn get_path_cached(
        &mut self,
        typeface: &Typeface,
        size: f32,
        glyph_id: u16,
    ) -> Option<&Path> {
        let key = (typeface.unique_id().into(), size.to_bits(), glyph_id);
        if self.path_cache.contains_key(&key) {
            return self.path_cache.get(&key);
        }

        let font = self.get_font(typeface, size);
        if let Some(path) = font.get_path(glyph_id) {
            self.path_cache.insert(key, path);
            self.path_cache.get(&key)
        } else {
            None
        }
    }

    // Legacy helper for system font dirs (can be removed if we trust Skia FontMgr)
    #[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
    pub fn get_system_font_dirs() -> Vec<PathBuf> {
        vec![
            PathBuf::from(std::env::var("WINDIR").unwrap_or("C:\\Windows".to_string()))
                .join("Fonts"),
            PathBuf::from(std::env::var("LOCALAPPDATA").unwrap_or(".".to_string()))
                .join("Microsoft\\Windows\\Fonts"),
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
        assert!(result.is_err(), "Invalid font bytes should return error");
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
        let mut renderer = TextRenderer::new();

        // Non-existent font returns None (or system font if available)
        let result = renderer.get_typeface("NonExistentFont12345");

        // Result depends on system fonts, but shouldn't panic
        // On most systems, this will return None unless the exact name exists
        let _ = result; // Just verify it doesn't panic
    }

    #[test]
    fn test_get_typeface_system_font() {
        let mut renderer = TextRenderer::new();

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
        let mut renderer = TextRenderer::new();

        // Get a system font for measurement
        #[cfg(target_os = "windows")]
        let font_name = "Arial";
        #[cfg(target_os = "macos")]
        let font_name = "Helvetica";
        #[cfg(target_os = "linux")]
        let font_name = "DejaVu Sans";

        if let Some(typeface) = renderer.get_typeface(font_name) {
            let (width, height, _) = renderer.measure_char(&typeface, 'A', 48.0);

            // Measurements should be positive
            assert!(width > 0.0, "Character width should be positive");
            assert!(height > 0.0, "Character height should be positive");
        }
        // If font not available, test passes (can't measure without font)
    }

    #[test]
    fn test_measure_char_different_sizes() {
        let mut renderer = TextRenderer::new();

        #[cfg(target_os = "windows")]
        let font_name = "Arial";
        #[cfg(target_os = "macos")]
        let font_name = "Helvetica";
        #[cfg(target_os = "linux")]
        let font_name = "DejaVu Sans";

        if let Some(typeface) = renderer.get_typeface(font_name) {
            let (width_small, _, _) = renderer.measure_char(&typeface, 'A', 24.0);
            let (width_large, _, _) = renderer.measure_char(&typeface, 'A', 72.0);

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
        let mut renderer = TextRenderer::new();

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
                assert!(
                    bounds.width() > 0.0,
                    "Glyph path should have positive width"
                );
            }
        }
    }

    // --- Cache Eviction Tests ---

    #[test]
    fn test_clear_font_cache_threshold() {
        let mut renderer = TextRenderer::new();

        #[cfg(target_os = "windows")]
        let font_name = "Arial";
        #[cfg(target_os = "macos")]
        let font_name = "Helvetica";
        #[cfg(target_os = "linux")]
        let font_name = "DejaVu Sans";

        if let Some(typeface) = renderer.get_typeface(font_name) {
            // Add a font to cache
            let _ = renderer.get_font(&typeface, 24.0);
            assert_eq!(renderer.resolved_font_cache.len(), 1);

            // Call clear - should NOT clear because 1 < 500
            renderer.clear_font_cache();
            assert_eq!(renderer.resolved_font_cache.len(), 1);

            // We can't easily test the upper bound eviction without adding 501 fonts
            // which requires 501 different sizes or typefaces.
            // But this proves the optimization (NOT clearing small caches) works.
        }
    }

    #[test]
    fn test_path_caching() {
        let mut renderer = TextRenderer::new();

        #[cfg(target_os = "windows")]
        let font_name = "Arial";
        #[cfg(target_os = "macos")]
        let font_name = "Helvetica";
        #[cfg(target_os = "linux")]
        let font_name = "DejaVu Sans";

        if let Some(typeface) = renderer.get_typeface(font_name) {
            let size = 48.0;
            // Get glyph ID for 'A'
            let font = Font::from_typeface(typeface.clone(), size);
            let glyph_id = font.unichar_to_glyph('A' as i32);

            // 1. Initial Call - Miss
            let path1 = renderer.get_path_cached(&typeface, size, glyph_id);
            assert!(path1.is_some());
            assert_eq!(renderer.path_cache.len(), 1);

            // 2. Second Call - Hit
            let path2 = renderer.get_path_cached(&typeface, size, glyph_id);
            assert!(path2.is_some());
            assert_eq!(renderer.path_cache.len(), 1); // Should still be 1

            // 3. Different Size - Miss
            let size2 = 24.0;
            let path3 = renderer.get_path_cached(&typeface, size2, glyph_id);
            assert!(path3.is_some());
            assert_eq!(renderer.path_cache.len(), 2);
        }
    }
}

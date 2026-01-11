use super::model::{Line, Style, Align};
use crate::text::TextRenderer;

#[derive(Debug, Clone)]
pub struct GlyphInfo {
    pub char: char,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub advance: f32,
    /// Index into the line.chars array this glyph belongs to
    pub char_index: usize,
}

pub struct LayoutEngine;

impl LayoutEngine {
    /// Calculate glyph positions for a line of text
    pub fn layout_line(
        line: &Line,
        resolved_style: &Style,
        renderer: &mut TextRenderer
    ) -> Vec<GlyphInfo> {
        // Base (Style) Defaults
        let style_family = resolved_style.font.as_ref().map(|f| f.family.as_str()).unwrap_or("Noto Sans SC");
        let style_size = resolved_style.font.as_ref().map(|f| f.size).unwrap_or(72.0);

        let mut glyphs = Vec::new();
        let mut cursor_x = 0.0;
        let gap = line.layout.as_ref().map(|l| l.gap).unwrap_or(0.0);
        
        // Iterate over character objects in the line
        for (i, char_data) in line.chars.iter().enumerate() {
            let ch_str = &char_data.char;
            
            // Resolve Font for this Char Unit (Char > Line > Style)
            let (family, size) = if let Some(cf) = &char_data.font {
                 (cf.family.as_str(), cf.size)
            } else if let Some(lf) = &line.font {
                 (lf.family.as_str(), lf.size)
            } else {
                 (style_family, style_size)
            };

            // For each character in the string
            for ch in ch_str.chars() {
                // Try to get font
                let font_ref = renderer.get_typeface(family)
                    .or_else(|| renderer.get_default_typeface());
                
                if let Some(typeface) = font_ref {
                    // Measure character using renderer
                    let (advance, height) = renderer.measure_char(&typeface, ch, size);
                    
                    let width = advance; 
                    
                    glyphs.push(GlyphInfo {
                        char: ch,
                        x: cursor_x,
                        y: 0.0, 
                        width,
                        height,
                        advance,
                        char_index: i
                    });
                    
                    cursor_x += advance;
                } else {
                     // Log warning only once?
                     // For now just skip or add zero width space
                     cursor_x += size * 0.5; // Fallback advance
                }
            }
            
            // Add gap after each KLyric char unit
            cursor_x += gap;
        }

        // Remove trailing gap for width calculation
        let total_width = if cursor_x > gap { cursor_x - gap } else { 0.0 };

        // Handle Alignment
        let align = line.layout.as_ref()
            .map(|l| l.align.clone())
            .unwrap_or(Align::Center);
            
        let align_offset = match align {
            Align::Center => -total_width / 2.0,
            Align::Right => -total_width,
            Align::Left => 0.0,
        };
        
        // Apply alignment offset
        for glyph in &mut glyphs {
            glyph.x += align_offset;
        }

        glyphs
    }
}

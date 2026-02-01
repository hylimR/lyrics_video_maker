use super::model::{Align, Line, Style};
use crate::text::TextRenderer;

#[cfg(not(target_arch = "wasm32"))]
use skia_safe::Typeface;
#[cfg(target_arch = "wasm32")]
use crate::text::Typeface;

#[cfg(not(target_arch = "wasm32"))]
use crate::text::ResolvedFont;

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
    /// The glyph ID in the font
    pub glyph_id: u16,
}

pub struct LayoutEngine;

impl LayoutEngine {
    /// Calculate glyph positions for a line of text
    pub fn layout_line(
        line: &Line,
        resolved_style: &Style,
        renderer: &mut TextRenderer,
    ) -> Vec<GlyphInfo> {
        // Base (Style) Defaults
        let style_family = resolved_style
            .font
            .as_ref()
            .and_then(|f| f.family.as_deref())
            .unwrap_or("Noto Sans SC");
        let style_size = resolved_style
            .font
            .as_ref()
            .and_then(|f| f.size)
            .unwrap_or(72.0);

        // Pre-allocate to avoid reallocations. usually one glyph per Char.
        let mut glyphs = Vec::with_capacity(line.chars.len());
        let mut cursor_x = 0.0;
        let gap = line.layout.as_ref().map(|l| l.gap).unwrap_or(0.0);

        // 1. Resolve Line-level fallback family
        let line_family_def = line
            .font
            .as_ref()
            .and_then(|f| f.family.as_deref())
            .unwrap_or(style_family);

        // 2. Resolve Typeface for Line-level fallback
        let line_typeface = renderer
            .get_typeface(line_family_def)
            .or_else(|| renderer.get_default_typeface());

        // Optimization: Hoist line font size resolution
        let line_font_size = line
            .font
            .as_ref()
            .and_then(|f| f.size)
            .unwrap_or(style_size);

        // Cache for ResolvedFont to avoid recreation overhead
        #[cfg(not(target_arch = "wasm32"))]
        let mut cached_font: Option<ResolvedFont> = None;
        #[cfg(not(target_arch = "wasm32"))]
        let mut cached_font_key: Option<(u32, u32)> = None;

        // Cache for Font Override
        let mut cached_family_override: Option<&str> = None;
        let mut cached_typeface_override: Option<Option<Typeface>> = None;

        // Iterate over character objects in the line
        for (i, char_data) in line.chars.iter().enumerate() {
            let ch_str = &char_data.char;

            // Check if char has family override
            let char_family_override = char_data.font.as_ref().and_then(|f| f.family.as_deref());

            // Resolve Typeface (Char override or Line default)
            let font_ref = if let Some(fam) = char_family_override {
                if cached_family_override != Some(fam) {
                    let tf = renderer
                        .get_typeface(fam)
                        .or_else(|| renderer.get_default_typeface());

                    cached_family_override = Some(fam);
                    cached_typeface_override = Some(tf);
                }
                cached_typeface_override.as_ref().unwrap()
            } else {
                &line_typeface
            };

            let size = char_data
                .font
                .as_ref()
                .and_then(|f| f.size)
                .unwrap_or(line_font_size);

            #[cfg(not(target_arch = "wasm32"))]
            {
                if let Some(tf) = font_ref.as_ref() {
                    let tf_id: u32 = tf.unique_id().into();
                    let size_bits = size.to_bits();

                    let can_reuse = if let Some((last_id, last_size)) = cached_font_key {
                        last_id == tf_id && last_size == size_bits
                    } else {
                        false
                    };

                    if !can_reuse {
                        cached_font = Some(renderer.create_font(tf, size));
                        cached_font_key = Some((tf_id, size_bits));
                    }
                } else {
                    cached_font = None;
                    cached_font_key = None;
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            let resolved_font_ref = cached_font.as_ref();

            // For each character in the string
            for ch in ch_str.chars() {
                // Try to get font
                if font_ref.is_some() {
                    // Measure character using renderer
                    #[cfg(not(target_arch = "wasm32"))]
                    let (advance, height, glyph_id) = if let Some(font) = resolved_font_ref {
                        renderer.measure_char_with_font(font, ch)
                    } else {
                        (0.0, 0.0, 0)
                    };

                    #[cfg(target_arch = "wasm32")]
                    let (advance, height, glyph_id) = if let Some(tf) = font_ref {
                        renderer.measure_char(tf, ch, size)
                    } else {
                        (0.0, 0.0, 0)
                    };

                    let width = advance;

                    glyphs.push(GlyphInfo {
                        char: ch,
                        x: cursor_x,
                        y: 0.0,
                        width,
                        height,
                        advance,
                        char_index: i,
                        glyph_id,
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
        let align = line
            .layout
            .as_ref()
            .map(|l| l.align)
            .unwrap_or(Align::Center);

        let align_offset = match align {
            Align::Center => -total_width / 2.0,
            Align::Right => -total_width,
            Align::Left => 0.0,
        };

        // Apply alignment offset (Optimized: skip if zero)
        if align_offset.abs() > f32::EPSILON {
            for glyph in &mut glyphs {
                glyph.x += align_offset;
            }
        }

        glyphs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Char, Font};

    /// Create a minimal Style for testing
    fn test_style() -> Style {
        Style {
            extends: None,
            font: Some(Font {
                family: Some("Arial".to_string()),
                size: Some(48.0),
                weight: None,
                style: None,
                letter_spacing: None,
            }),
            colors: None,
            stroke: None,
            shadow: None,
            glow: None,
            transform: None,
        }
    }

    /// Create a minimal Line for testing
    fn test_line(chars: Vec<&str>) -> Line {
        Line {
            id: None,
            start: 0.0,
            end: 5.0,
            text: Some(chars.join("")),
            style: None,
            font: None,
            stroke: None,
            shadow: None,
            effects: Vec::new(),
            position: None,
            transform: None,
            layout: None,
            chars: chars
                .iter()
                .enumerate()
                .map(|(i, c)| Char {
                    char: c.to_string(),
                    start: i as f64,
                    end: (i + 1) as f64,
                    style: None,
                    font: None,
                    stroke: None,
                    shadow: None,
                    effects: Vec::new(),
                    transform: None,
                })
                .collect(),
        }
    }

    // Helper to assert float approximately equal
    fn approx_eq(a: f32, b: f32, epsilon: f32) -> bool {
        (a - b).abs() < epsilon
    }

    // --- Layout Basic Tests ---

    #[test]
    fn test_layout_empty() {
        // Empty line should produce empty glyphs
        let line = test_line(vec![]);
        let style = test_style();
        let mut renderer = TextRenderer::new();

        let glyphs = LayoutEngine::layout_line(&line, &style, &mut renderer);
        assert!(glyphs.is_empty(), "Empty line should produce no glyphs");
    }

    #[test]
    fn test_layout_single_char() {
        // Single character line
        let line = test_line(vec!["A"]);
        let style = test_style();
        let mut renderer = TextRenderer::new();

        let glyphs = LayoutEngine::layout_line(&line, &style, &mut renderer);
        assert_eq!(glyphs.len(), 1, "Single char line should produce one glyph");
        assert_eq!(glyphs[0].char, 'A');
        assert_eq!(glyphs[0].char_index, 0);
    }

    #[test]
    fn test_layout_basic() {
        // Two characters should produce two glyphs in order
        let line = test_line(vec!["H", "i"]);
        let style = test_style();
        let mut renderer = TextRenderer::new();

        let glyphs = LayoutEngine::layout_line(&line, &style, &mut renderer);
        assert_eq!(glyphs.len(), 2, "Two chars should produce two glyphs");
        assert_eq!(glyphs[0].char, 'H');
        assert_eq!(glyphs[1].char, 'i');
        assert_eq!(glyphs[0].char_index, 0);
        assert_eq!(glyphs[1].char_index, 1);
    }

    // --- Alignment Tests ---

    #[test]
    fn test_alignment_left() {
        // Left alignment: first glyph at x=0
        let mut line = test_line(vec!["A", "B"]);
        line.layout = Some(crate::model::Layout {
            mode: crate::model::LayoutMode::Horizontal,
            align: Align::Left,
            justify: crate::model::Justify::Middle,
            gap: 0.0,
            wrap: false,
            max_width: None,
        });

        let style = test_style();
        let mut renderer = TextRenderer::new();

        let glyphs = LayoutEngine::layout_line(&line, &style, &mut renderer);
        // With left alignment, first glyph x should be 0 (or very close)
        assert!(
            approx_eq(glyphs[0].x, 0.0, 1.0),
            "Left-aligned first glyph x should be ~0, got {}",
            glyphs[0].x
        );
    }

    #[test]
    fn test_alignment_center() {
        // Center alignment: glyphs centered around origin
        let mut line = test_line(vec!["A", "B"]);
        line.layout = Some(crate::model::Layout {
            mode: crate::model::LayoutMode::Horizontal,
            align: Align::Center,
            justify: crate::model::Justify::Middle,
            gap: 0.0,
            wrap: false,
            max_width: None,
        });

        let style = test_style();
        let mut renderer = TextRenderer::new();

        let glyphs = LayoutEngine::layout_line(&line, &style, &mut renderer);
        // With center alignment, the first glyph x should be negative
        // (total width / 2 to the left of origin)
        if !glyphs.is_empty() {
            let total_width: f32 = glyphs.iter().map(|g| g.advance).sum();
            // First glyph should be at approximately -total_width/2
            let expected_start = -total_width / 2.0;
            assert!(
                approx_eq(glyphs[0].x, expected_start, 5.0),
                "Center-aligned first glyph x should be ~{}, got {}",
                expected_start,
                glyphs[0].x
            );
        }
    }

    #[test]
    fn test_alignment_right() {
        // Right alignment: last glyph ends at x=0
        let mut line = test_line(vec!["A", "B"]);
        line.layout = Some(crate::model::Layout {
            mode: crate::model::LayoutMode::Horizontal,
            align: Align::Right,
            justify: crate::model::Justify::Middle,
            gap: 0.0,
            wrap: false,
            max_width: None,
        });

        let style = test_style();
        let mut renderer = TextRenderer::new();

        let glyphs = LayoutEngine::layout_line(&line, &style, &mut renderer);
        // With right alignment, glyphs should end at x=0
        // Last glyph x + width should be approximately 0
        if !glyphs.is_empty() {
            let last = glyphs.last().unwrap();
            let right_edge = last.x + last.advance;
            assert!(
                approx_eq(right_edge, 0.0, 5.0),
                "Right-aligned last glyph right edge should be ~0, got {}",
                right_edge
            );
        }
    }

    // --- Spacing Tests ---

    #[test]
    fn test_gap_between_chars() {
        // Gap spacing should be applied between characters
        let mut line = test_line(vec!["A", "B", "C"]);
        let gap = 10.0;
        line.layout = Some(crate::model::Layout {
            mode: crate::model::LayoutMode::Horizontal,
            align: Align::Left,
            justify: crate::model::Justify::Middle,
            gap,
            wrap: false,
            max_width: None,
        });

        let style = test_style();
        let mut renderer = TextRenderer::new();

        let glyphs = LayoutEngine::layout_line(&line, &style, &mut renderer);
        // With gap, the distance between successive glyph starts should include gap
        if glyphs.len() >= 2 {
            let distance = glyphs[1].x - glyphs[0].x;
            // Distance should be at least the first glyph advance + gap
            let expected_min = glyphs[0].advance + gap;
            assert!(
                distance >= expected_min - 1.0,
                "Gap should be applied: distance {} should be >= {} (advance {} + gap {})",
                distance,
                expected_min,
                glyphs[0].advance,
                gap
            );
        }
    }

    #[test]
    fn test_no_gap() {
        // Zero gap should work correctly
        let mut line = test_line(vec!["A", "B"]);
        line.layout = Some(crate::model::Layout {
            mode: crate::model::LayoutMode::Horizontal,
            align: Align::Left,
            justify: crate::model::Justify::Middle,
            gap: 0.0,
            wrap: false,
            max_width: None,
        });

        let style = test_style();
        let mut renderer = TextRenderer::new();

        let glyphs = LayoutEngine::layout_line(&line, &style, &mut renderer);
        if glyphs.len() >= 2 {
            // Second glyph should start immediately after first glyph advance
            let expected_x = glyphs[0].advance;
            assert!(
                approx_eq(glyphs[1].x - glyphs[0].x, expected_x, 1.0),
                "No gap: second glyph should start at first glyph x + advance"
            );
        }
    }

    // --- Font Cascade Tests ---

    #[test]
    fn test_font_cascade_char_level() {
        // Test that char-level font settings are properly resolved in the cascade
        // Char > Line > Style priority
        let mut line = test_line(vec!["A"]);

        // Set line-level font
        line.font = Some(Font {
            family: Some("Times".to_string()),
            size: Some(24.0),
            ..Default::default()
        });

        // Set char-level font override
        line.chars[0].font = Some(Font {
            family: Some("Helvetica".to_string()),
            size: Some(72.0),
            ..Default::default()
        });

        let style = test_style();
        let mut renderer = TextRenderer::new();

        // This tests the code path - layout should complete without panic
        // Even if fonts aren't available, the cursor advances via fallback
        let glyphs = LayoutEngine::layout_line(&line, &style, &mut renderer);

        // The test verifies the cascade logic executes:
        // - If system fonts available: glyphs produced with char-level font
        // - If not: fallback cursor advance happens
        // Either way, we verify no panic and layout completes
        let _ = glyphs; // Layout completed successfully
    }

    #[test]
    fn test_font_cascade_line_level() {
        // Test that line-level font settings override style-level
        let mut line = test_line(vec!["A"]);

        // Set line-level font
        line.font = Some(Font {
            family: Some("Courier".to_string()),
            size: Some(36.0),
            ..Default::default()
        });

        let style = test_style();
        let mut renderer = TextRenderer::new();

        // This tests the code path - verify cascade logic executes
        let glyphs = LayoutEngine::layout_line(&line, &style, &mut renderer);

        // Verify layout completes (cascade logic exercised)
        let _ = glyphs;
    }
}

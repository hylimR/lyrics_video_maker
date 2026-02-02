use klyric_renderer::layout::LayoutEngine;
use klyric_renderer::model::{Char, Line, Shadow, Stroke, Style};
use klyric_renderer::text::TextRenderer;
use skia_safe::Color;

#[test]
fn test_layout_color_parsing_optimization() {
    // Setup a line with multiple characters having the same shadow color override
    let shadow_hex = "#FF0000"; // Red
    let stroke_hex = "#00FF00"; // Green

    let chars: Vec<Char> = (0..5)
        .map(|i| Char {
            char: "A".to_string(),
            start: i as f64,
            end: (i + 1) as f64,
            style: None,
            font: None,
            stroke: Some(Stroke {
                color: Some(stroke_hex.to_string()),
                width: Some(2.0),
            }),
            shadow: Some(Shadow {
                color: Some(shadow_hex.to_string()),
                x: Some(2.0),
                y: Some(2.0),
                blur: None,
            }),
            effects: Vec::new(),
            transform: None,
        })
        .collect();

    let line = Line {
        id: None,
        start: 0.0,
        end: 5.0,
        text: Some("AAAAA".to_string()),
        style: None,
        font: None,
        stroke: None,
        shadow: None,
        effects: Vec::new(),
        position: None,
        transform: None,
        layout: None,
        chars,
    };

    let style = Style::default();
    let mut renderer = TextRenderer::new();

    // Run layout
    let glyphs = LayoutEngine::layout_line(&line, &style, &mut renderer);

    // Verify
    assert!(!glyphs.is_empty());

    // Check if colors are correctly populated in GlyphInfo
    // Note: This test passes without optimization too, but verifies correctness.
    // The optimization is purely internal to avoid parsing string repeatedly.
    for (i, glyph) in glyphs.iter().enumerate() {
        // Verify Shadow
        if let Some(color) = glyph.override_shadow_color {
            let (a, r, g, b) = color.to_argb();
            assert_eq!(r, 255, "Shadow Red mismatch at char {}", i);
            assert_eq!(g, 0, "Shadow Green mismatch at char {}", i);
            assert_eq!(b, 0, "Shadow Blue mismatch at char {}", i);
            assert_eq!(a, 255, "Shadow Alpha mismatch at char {}", i);
        } else {
            panic!("Shadow color missing for char {}", i);
        }

        // Verify Stroke
        if let Some(color) = glyph.override_stroke_color {
            let (a, r, g, b) = color.to_argb();
            assert_eq!(r, 0, "Stroke Red mismatch at char {}", i);
            assert_eq!(g, 255, "Stroke Green mismatch at char {}", i);
            assert_eq!(b, 0, "Stroke Blue mismatch at char {}", i);
            assert_eq!(a, 255, "Stroke Alpha mismatch at char {}", i);
        } else {
            panic!("Stroke color missing for char {}", i);
        }
    }
}

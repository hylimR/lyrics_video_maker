use klyric_renderer::{Renderer, KLyricDocumentV2, model::*};
use std::collections::HashMap;

#[test]
fn test_stroke_reveal_with_repetitive_text() {
    let width = 200;
    let height = 100;
    let mut renderer = Renderer::new(width, height);

    let mut doc = KLyricDocumentV2::default();
    doc.version = "2.0".to_string();
    doc.project.resolution = Resolution { width, height };
    doc.theme = Some(Theme {
        background: Some(Background {
            bg_type: BackgroundType::Solid,
            color: Some("#000000".to_string()),
            gradient: None,
            image: None,
            video: None,
            opacity: 1.0,
        }),
        default_style: None,
    });

    // Define StrokeReveal effect
    let mut effect = Effect::default();
    effect.effect_type = EffectType::StrokeReveal;
    effect.duration = Some(2.0);
    effect.delay = 0.0;
    doc.effects.insert("reveal".to_string(), effect);

    // Create a line with repetitive text
    // "aaaaaaaaaa" (10 chars)
    let text = "aaaaaaaaaa";
    let mut line = Line::default();
    line.text = Some(text.to_string());
    line.start = 0.0;
    line.end = 2.0;
    line.effects = vec!["reveal".to_string()];

    // Manually populate chars to ensure they are processed
    // (In real app, Importer does this)
    line.chars = text.chars().enumerate().map(|(i, c)| Char {
        char: c.to_string(),
        start: 0.0,
        end: 2.0,
        style: None,
        font: None,
        stroke: None,
        shadow: None,
        effects: Vec::new(),
        transform: None,
    }).collect();

    doc.lines.push(line);

    // Render at 1.0s (50% progress)
    // This should trigger the segment cache logic if implemented.
    // The test passes if it renders without error (and we assume optimization works via code review).
    let result = renderer.render_frame(&doc, 1.0);
    assert!(result.is_ok(), "Render failed: {:?}", result.err());
}

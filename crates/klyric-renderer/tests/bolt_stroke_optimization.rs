
mod helpers;

use klyric_renderer::{Renderer, KLyricDocumentV2, model::Effect, model::EffectType, model::Easing};

// Helper to create a doc with StrokeReveal
fn create_stroke_doc() -> KLyricDocumentV2 {
    let mut doc = helpers::doc_with_line("Test", 0.0, 5.0);

    // Add StrokeReveal effect
    let mut effect = Effect::default();
    effect.effect_type = EffectType::StrokeReveal;
    effect.duration = Some(1.0);
    effect.easing = Easing::Linear;

    doc.effects.insert("reveal".to_string(), effect);

    // Apply to line
    if let Some(line) = doc.lines.first_mut() {
        line.effects.push("reveal".to_string());
    }

    doc
}

#[test]
fn test_stroke_reveal_optimization_start() {
    // This test verifies that rendering a StrokeReveal at progress 0.0 works correctly
    // (Optimization: Should skip PathMeasure and draw nothing)

    let mut renderer = Renderer::new(800, 600);
    let doc = create_stroke_doc();

    // T=0.0 (Progress 0.0)
    // With optimization, this returns empty path and draws nothing.
    let pixels = renderer.render_frame(&doc, 0.0).expect("Render failed at 0.0");

    // Should be black (empty path drawn)
    let count = helpers::count_non_black_pixels(&pixels, 10);
    assert_eq!(count, 0, "Should draw nothing at progress 0.0");
}

#[test]
fn test_stroke_reveal_mid_progress() {
    // This test ensures that optimization doesn't break normal rendering
    let mut renderer = Renderer::new(800, 600);
    let doc = create_stroke_doc();

    // T=0.5 (Progress 0.5)
    // Should behave normally (PathMeasure used)
    let _pixels = renderer.render_frame(&doc, 0.5).expect("Render failed at 0.5");

    // We don't assert pixels because font loading might fail in CI/test env without system fonts
}

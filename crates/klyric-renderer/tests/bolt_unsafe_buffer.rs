use klyric_renderer::{Renderer, KLyricDocumentV2, model::*};

#[test]
fn test_render_buffer_integrity() {
    let width = 100;
    let height = 100;
    let mut renderer = Renderer::new(width, height);

    // Create doc with solid red background
    let mut doc = KLyricDocumentV2::default();
    doc.version = "2.0".to_string();
    doc.project.resolution = Resolution { width, height };

    doc.theme = Some(Theme {
        background: Some(Background {
            bg_type: BackgroundType::Solid,
            color: Some("#FF0000".to_string()), // Red
            gradient: None,
            image: None,
            video: None,
            opacity: 1.0,
        }),
        default_style: None,
    });

    // Render
    let pixels = renderer.render_frame(&doc, 0.0).expect("Render failed");

    // Verify size
    assert_eq!(pixels.len(), (width * height * 4) as usize);

    // Verify every pixel is Red (255, 0, 0, 255)
    // Note: Premultiplied Alpha. Solid opaque red is (255, 0, 0, 255).
    // Allow small tolerance for GPU/Skia precision issues, though solid color should be exact.
    for (i, chunk) in pixels.chunks(4).enumerate() {
        let r = chunk[0];
        let g = chunk[1];
        let b = chunk[2];
        let a = chunk[3];

        // Strict check for Red
        if r < 250 || g > 5 || b > 5 || a < 250 {
             panic!("Pixel {} mismatch: expected red, got {:?}", i, chunk);
        }
    }
}

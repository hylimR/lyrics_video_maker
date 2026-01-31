use klyric_renderer::{parse_document, KLyricDocumentV2, Renderer};
use std::path::Path;

fn create_shadow_test_doc(font_family: &str) -> KLyricDocumentV2 {
    let json = format!(
        r##"{{
        "version": "2.0",
        "project": {{
            "title": "Shadow Test",
            "resolution": {{ "width": 800, "height": 600 }},
            "duration": 10.0
        }},
        "lines": [
            {{
                "text": "Override",
                "start": 1.0,
                "end": 3.0,
                "chars": [
                    {{ "char": "A", "start": 1.0, "end": 1.2, "shadow": {{ "color": "#00FF00" }} }},
                    {{ "char": "B", "start": 1.2, "end": 1.4 }},
                    {{ "char": "C", "start": 1.4, "end": 1.6 }}
                ],
                "shadow": {{ "color": "#0000FF", "x": 2.0, "y": 2.0 }},
                "style": "base"
            }}
        ],
        "styles": {{
            "base": {{
                "font": {{ "family": "{}", "size": 60.0 }},
                "shadow": {{ "color": "#FF0000", "x": 5.0, "y": 5.0 }},
                "colors": {{
                    "active": {{ "fill": "#FFFFFF" }}
                }}
            }}
        }}
    }}"##,
        font_family
    );

    parse_document(&json).expect("Failed to parse test document")
}

fn setup_renderer(width: u32, height: u32) -> (Renderer, Option<String>) {
    let mut renderer = Renderer::new(width, height);
    let mut font_name: Option<String> = None;

    // Minimal font loading for Linux/Windows/Mac
    #[cfg(target_os = "linux")]
    let paths = [
        (
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "DejaVu Sans",
        ),
        ("/usr/share/fonts/TTF/DejaVuSans.ttf", "DejaVu Sans"),
    ];
    #[cfg(target_os = "windows")]
    let paths = [("C:\\Windows\\Fonts\\arial.ttf", "Arial")];
    #[cfg(target_os = "macos")]
    let paths = [("/Library/Fonts/Arial.ttf", "Arial")];

    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    for (path, name) in paths {
        if Path::new(path).exists() {
            if renderer.text_renderer_mut().load_font(name, path).is_ok() {
                font_name = Some(name.to_string());
                break;
            }
        }
    }

    (renderer, font_name)
}

#[test]
fn test_shadow_rendering() {
    let (mut renderer, font_name) = setup_renderer(800, 600);
    if font_name.is_none() {
        println!("Skipping shadow test due to missing font");
        return;
    }

    let doc = create_shadow_test_doc(font_name.as_deref().unwrap());

    // Render at 1.5s
    let pixels = renderer.render_frame(&doc, 1.5).expect("Failed to render");

    // We can't easily assert colors because of antialiasing and exact positions.
    // But if this runs without panic, it means the logic is sound.
    // In a real scenario, we might want to check that we have Green pixels (Char A),
    // Blue pixels (Char B, C - from Line), and NO Red pixels (Style is overridden by Line).

    // Check for Green (Char Override)
    let has_green = pixels
        .chunks_exact(4)
        .any(|p| p[1] > 200 && p[0] < 50 && p[2] < 50);

    // Check for Blue (Line Override)
    let has_blue = pixels
        .chunks_exact(4)
        .any(|p| p[2] > 200 && p[0] < 50 && p[1] < 50);

    // Check for Red (Style - Should be HIDDEN because Line overrides it)
    // Actually, Line "shadow" replaces Style "shadow". So Style shadow (Red) should NOT appear.
    // Note: Text is White.
    let has_red = pixels
        .chunks_exact(4)
        .any(|p| p[0] > 200 && p[1] < 50 && p[2] < 50);

    println!("Has Green (Char): {}", has_green);
    println!("Has Blue (Line): {}", has_blue);
    println!("Has Red (Style): {}", has_red);

    assert!(has_green, "Should have green shadow from Char override");
    assert!(has_blue, "Should have blue shadow from Line override");
    assert!(
        !has_red,
        "Should NOT have red shadow (Style is overridden by Line)"
    );
}

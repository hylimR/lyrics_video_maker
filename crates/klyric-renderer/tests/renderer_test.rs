use klyric_renderer::{Renderer, parse_document, KLyricDocumentV2};
use std::path::Path;

fn create_test_doc(font_family: &str) -> KLyricDocumentV2 {
    let json = format!(r##"{{
        "version": "2.0",
        "project": {{
            "title": "Test",
            "artist": "Test Artist",
            "duration": 180.0,
            "resolution": {{ "width": 1920, "height": 1080 }}
        }},
        "lines": [
            {{
                "text": "Hello",
                "start": 1.0,
                "end": 3.0,
                "chars": [
                    {{ "char": "H", "start": 1.0, "end": 1.2 }},
                    {{ "char": "e", "start": 1.2, "end": 1.4 }},
                    {{ "char": "l", "start": 1.4, "end": 1.6 }},
                    {{ "char": "l", "start": 1.6, "end": 1.8 }},
                    {{ "char": "o", "start": 1.8, "end": 2.0 }}
                ],
                "style": "base"
            }}
        ],
        "styles": {{
            "base": {{
                "font": {{ "family": "{}", "size": 60.0 }},
                "colors": {{
                    "active": {{ "fill": "#FFFFFF" }},
                    "inactive": {{ "fill": "#888888" }},
                    "complete": {{ "fill": "#FFFFFF" }}
                }}
            }}
        }}
    }}"##, font_family);

    parse_document(&json).expect("Failed to parse test document")
}

/// Returns (Renderer, font_name) tuple where font_name is Some if font was loaded
fn setup_renderer(width: u32, height: u32) -> (Renderer, Option<String>) {
    let mut renderer = Renderer::new(width, height);
    let mut font_name: Option<String> = None;

    // Try platform-specific font paths
    #[cfg(target_os = "windows")]
    {
        let font_paths: [(&str, &str); 3] = [
            ("C:\\Windows\\Fonts\\arial.ttf", "Arial"),
            ("C:\\Windows\\Fonts\\segoeui.ttf", "Segoe UI"),
            ("C:\\Windows\\Fonts\\calibri.ttf", "Calibri"),
        ];
        for (path, name) in font_paths {
            if Path::new(path).exists() {
                if renderer.text_renderer_mut().load_font(name, path).is_ok() {
                    font_name = Some(name.to_string());
                    println!("Loaded font '{}' from: {}", name, path);
                    break;
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let font_paths: [(&str, &str); 3] = [
            ("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", "DejaVu Sans"),
            ("/usr/share/fonts/TTF/DejaVuSans.ttf", "DejaVu Sans"),
            ("/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf", "Liberation Sans"),
        ];
        for (path, name) in font_paths {
            if Path::new(path).exists() {
                if renderer.text_renderer_mut().load_font(name, path).is_ok() {
                    font_name = Some(name.to_string());
                    println!("Loaded font '{}' from: {}", name, path);
                    break;
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let font_paths: [(&str, &str); 3] = [
            ("/System/Library/Fonts/Helvetica.ttc", "Helvetica"),
            ("/System/Library/Fonts/SFNSText.ttf", "SF NS Text"),
            ("/Library/Fonts/Arial.ttf", "Arial"),
        ];
        for (path, name) in font_paths {
            if Path::new(path).exists() {
                if renderer.text_renderer_mut().load_font(name, path).is_ok() {
                    font_name = Some(name.to_string());
                    println!("Loaded font '{}' from: {}", name, path);
                    break;
                }
            }
        }
    }

    if font_name.is_none() {
        println!("WARNING: No font loaded. Text rendering tests may be skipped.");
    }

    (renderer, font_name)
}

/// Get pixel at (x, y) from raw RGBA buffer
fn get_pixel(pixels: &[u8], width: u32, x: u32, y: u32) -> (u8, u8, u8, u8) {
    let idx = ((y * width + x) * 4) as usize;
    (pixels[idx], pixels[idx + 1], pixels[idx + 2], pixels[idx + 3])
}

#[test]
fn test_basic_render() {
    let (mut renderer, font_name) = setup_renderer(800, 600);
    let doc = create_test_doc(font_name.as_deref().unwrap_or("Arial"));

    let pixels = renderer.render_frame(&doc, 0.0).expect("Failed to render frame");

    // Verify buffer size: width * height * 4 (RGBA)
    assert_eq!(pixels.len(), 800 * 600 * 4);

    // Check background (black) at center
    let (r, g, b, a) = get_pixel(&pixels, 800, 400, 300);
    // Default background is black (0, 0, 0, 255)
    assert_eq!(r, 0, "Red channel should be 0");
    assert_eq!(g, 0, "Green channel should be 0");
    assert_eq!(b, 0, "Blue channel should be 0");
    assert_eq!(a, 255, "Alpha channel should be 255");
}

#[test]
fn test_lyrics_presence() {
    let (mut renderer, font_name) = setup_renderer(800, 600);

    // Skip text visibility test if no font was loaded
    if font_name.is_none() {
        println!("SKIPPING: test_lyrics_presence - no font available");
        let doc = create_test_doc("Arial");
        // Still verify rendering works without crashing
        let pixels = renderer.render_frame(&doc, 1.5).expect("Failed to render frame");
        assert_eq!(pixels.len(), 800 * 600 * 4, "Should still produce correct buffer size");
        return;
    }

    let doc = create_test_doc(font_name.as_deref().unwrap());

    // Check t=1.5s ("Hello" should be visible)
    {
        let pixels = renderer.render_frame(&doc, 1.5).expect("Failed to render frame");

        let mut drawn_pixels = 0;
        for chunk in pixels.chunks_exact(4) {
            let (r, g, b, _a) = (chunk[0], chunk[1], chunk[2], chunk[3]);
            // Check if any pixel is non-black
            if r > 20 || g > 20 || b > 20 {
                drawn_pixels += 1;
            }
        }

        println!("At 1.5s, found {} drawn pixels", drawn_pixels);
        assert!(drawn_pixels > 0, "Lyrics should be visible at 1.5s");
    }

    // Check t=5.0s (Lyrics ended)
    {
        let pixels = renderer.render_frame(&doc, 5.0).expect("Failed to render frame");

        let mut drawn_pixels = 0;
        for chunk in pixels.chunks_exact(4) {
            let (r, g, b, _a) = (chunk[0], chunk[1], chunk[2], chunk[3]);
            if r > 20 || g > 20 || b > 20 {
                drawn_pixels += 1;
            }
        }

        println!("At 5.0s, found {} drawn pixels", drawn_pixels);
        assert_eq!(drawn_pixels, 0, "Lyrics should vanish after end time");
    }
}

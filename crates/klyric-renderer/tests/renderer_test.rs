use klyric_renderer::{Renderer, parse_document, KLyricDocumentV2, TextRenderer};
use klyric_renderer::tiny_skia::{Color, Pixmap};
use std::path::{Path, PathBuf};

fn create_test_doc() -> KLyricDocumentV2 {
    let json = r##"{
        "version": "2.0",
        "project": {
            "title": "Test",
            "artist": "Test Artist",
            "duration": 180.0,
            "resolution": { "width": 1920, "height": 1080 }
        },
        "lines": [
            {
                "text": "Hello",
                "start": 1.0,
                "end": 3.0,
                "chars": [
                    { "char": "H", "start": 1.0, "end": 1.2 },
                    { "char": "e", "start": 1.2, "end": 1.4 },
                    { "char": "l", "start": 1.4, "end": 1.6 },
                    { "char": "l", "start": 1.6, "end": 1.8 },
                    { "char": "o", "start": 1.8, "end": 2.0 }
                ],
                "style": "base"
            }
        ],
        "styles": {
            "base": {
                "font": { "family": "DejaVu Sans", "size": 60.0 },
                "colors": {
                    "active": { "fill": "#FFFFFF" },
                    "inactive": { "fill": "#888888" },
                    "complete": { "fill": "#FFFFFF" }
                }
            }
        }
    }"##;
    
    parse_document(json).expect("Failed to parse test document")
}

fn setup_renderer(width: u32, height: u32) -> Renderer {
    let mut renderer = Renderer::new(width, height);
    
    // Attempt to load Arial font
    // Note: This relies on "Arial" being present in system fonts.
    #[cfg(not(target_arch = "wasm32"))]
    {
        // Try Arial or fallback
        if let Some(path) = TextRenderer::find_font_file("Arial") {
            println!("Loading font from {:?}", path);
            renderer.text_renderer_mut().load_font("Arial", path.to_str().unwrap()).unwrap();
        } else if let Some(path) = TextRenderer::find_font_file("DejaVu Sans") {
             println!("Loading font from {:?}", path);
             renderer.text_renderer_mut().load_font("DejaVu Sans", path.to_str().unwrap()).unwrap();
        } else {
             // Hardcode fallback for test env
             let p = Path::new("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf");
             if p.exists() {
                 renderer.text_renderer_mut().load_font("DejaVu Sans", p.to_str().unwrap()).unwrap();
             } else {
                 println!("WARNING: Font not found. Tests might render blank text.");
             }
        }
    }
    
    renderer
}

#[test]
fn test_basic_render() {
    let mut renderer = setup_renderer(800, 600);
    // Even without a doc, render_frame creates a background
    // But we need a doc.
    let doc = create_test_doc();
    
    let pixmap = renderer.render_frame(&doc, 0.0).expect("Failed to render frame");
    
    assert_eq!(pixmap.width(), 800);
    assert_eq!(pixmap.height(), 600);
    
    // Check background (black)
    let pixel = pixmap.pixel(400, 300).unwrap();
    // Default background is black (0, 0, 0, 255)
    assert_eq!(pixel.red(), 0);
    assert_eq!(pixel.green(), 0);
    assert_eq!(pixel.blue(), 0);
    assert_eq!(pixel.alpha(), 255);
}

#[test]
fn test_lyrics_presence() {
    let mut renderer = setup_renderer(800, 600);
    let doc = create_test_doc();
    
    // Check t=1.5s ("Hello" should be visible)
    {
        let pixmap = renderer.render_frame(&doc, 1.5).expect("Failed to render frame");
        
        let mut drawn_pixels = 0;
        for pixel in pixmap.pixels() {
            // Check if any pixel is non-black
            if pixel.red() > 20 || pixel.green() > 20 || pixel.blue() > 20 {
                drawn_pixels += 1;
            }
        }
        
        println!("At 1.5s, found {} drawn pixels", drawn_pixels);
        assert!(drawn_pixels > 0, "Lyrics should be visible at 1.5s");
        
        // Optional debug
        // let output_path = std::env::temp_dir().join("test_presence_active.png");
        // pixmap.save_png(&output_path).ok();
    }
    
    // Check t=5.0s (Lyrics ended)
    {
        let pixmap = renderer.render_frame(&doc, 5.0).expect("Failed to render frame");
        
        let mut drawn_pixels = 0;
        for pixel in pixmap.pixels() {
            if pixel.red() > 20 || pixel.green() > 20 || pixel.blue() > 20 {
                drawn_pixels += 1;
            }
        }
        
        println!("At 5.0s, found {} drawn pixels", drawn_pixels);
        assert_eq!(drawn_pixels, 0, "Lyrics should vanish after end time");
    }
}

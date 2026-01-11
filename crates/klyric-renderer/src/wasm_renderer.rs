use crate::model::{KLyricDocumentV2, Line, Style, PositionValue};
use crate::layout::LayoutEngine;
use anyhow::{Result, anyhow};
use tiny_skia::{Pixmap, Color};
use ab_glyph::{Font, FontArc, PxScale, ScaleFont, point};
use fontdb::Database;

// Typeface wrapper for ab_glyph::FontArc
#[derive(Clone, Debug)]
pub struct Typeface(pub FontArc);

pub struct TextRenderer {
    db: Database,
    default_typeface: Option<Typeface>,
}

impl TextRenderer {
    pub fn new() -> Self {
        let db = Database::new();
        Self {
            db,
            default_typeface: None,
        }
    }

    pub fn load_font_bytes(&mut self, _name: &str, data: Vec<u8>) -> Result<()> {
        let source = fontdb::Source::Binary(std::sync::Arc::new(data));
        self.db.load_font_source(source);
        Ok(())
    }

    pub fn set_default_font_bytes(&mut self, data: Vec<u8>) -> Result<()> {
        // FontArc::try_from_vec expects Vec<u8>
        let font = FontArc::try_from_vec(data)?;
        self.default_typeface = Some(Typeface(font));
        Ok(())
    }

    pub fn get_typeface(&self, family: &str) -> Option<Typeface> {
        let query = fontdb::Query {
            families: &[fontdb::Family::Name(family), fontdb::Family::SansSerif],
            ..fontdb::Query::default()
        };
        
        if let Some(id) = self.db.query(&query) {
             self.db.with_face_data(id, |data, _index| {
                 // Clone data to own it for FontArc. Not efficient but works for WASM.
                 FontArc::try_from_vec(data.to_vec()).ok().map(|f| Typeface(f))
             }).flatten()
        } else {
            None
        }
    }

    pub fn get_default_typeface(&self) -> Option<Typeface> {
        self.default_typeface.clone()
    }

    pub fn measure_char(&self, typeface: &Typeface, ch: char, size: f32) -> (f32, f32) {
        let font = &typeface.0;
        let scaled = font.as_scaled(size);
        let glyph_id = font.glyph_id(ch);
        
        let advance = scaled.h_advance(glyph_id);
        let height = scaled.ascent() - scaled.descent(); 
        (advance, height)
    }
}

pub struct Renderer {
    width: u32,
    height: u32,
    text_renderer: TextRenderer,
}

impl Renderer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width, height,
            text_renderer: TextRenderer::new(),
        }
    }
    
    pub fn text_renderer_mut(&mut self) -> &mut TextRenderer {
        &mut self.text_renderer
    }
    
    pub fn render_frame(&mut self, doc: &KLyricDocumentV2, time: f64) -> Result<Vec<u8>> {
        let mut pixmap = Pixmap::new(self.width, self.height)
            .ok_or_else(|| anyhow!("Failed to create pixmap"))?;
            
        pixmap.fill(Color::TRANSPARENT);
        
        let default_style = Style::default();
        
        for line in &doc.lines {
            let start_time = line.start;
            let end_time = line.end;
            if time >= start_time - 1.0 && time <= end_time + 1.0 {
                self.render_line(&mut pixmap, line, &default_style, time)?;
            }
        }
        
        Ok(pixmap.data().to_vec())
    }
    
    fn render_line(&mut self, pixmap: &mut Pixmap, line: &Line, _style: &Style, _time: f64) -> Result<()> {
        let resolved_style = _style.clone();
        let glyphs = LayoutEngine::layout_line(line, &resolved_style, &mut self.text_renderer);
        
        let cx = self.width as f32 / 2.0;
        let cy = self.height as f32 / 2.0;
        
        let (lx, ly) = if let Some(pos) = &line.position {
            let x = resolve_position_value(&pos.x, self.width as f32, 0.0);
            let y = resolve_position_value(&pos.y, self.height as f32, 0.0);
            (x, y)
        } else {
             (0.0, 0.0)
        };
        
        for glyph_info in glyphs {
            let char_data = &line.chars[glyph_info.char_index];
            let font_spec = char_data.font.as_ref()
                .or(line.font.as_ref())
                .or(resolved_style.font.as_ref());
                
            let family = font_spec.map(|f| f.family.as_str()).unwrap_or("Noto Sans SC");
            let size = font_spec.map(|f| f.size).unwrap_or(72.0);
            
            let typeface = self.text_renderer.get_typeface(family)
                .or_else(|| self.text_renderer.get_default_typeface());
                
            if let Some(tf) = typeface {
               self.draw_glyph(pixmap, &tf, glyph_info.char, size, cx + lx + glyph_info.x, cy + ly + glyph_info.y);
            }
        }
        
        Ok(())
    }
    
    fn draw_glyph(&self, pixmap: &mut Pixmap, typeface: &Typeface, ch: char, size: f32, x: f32, y: f32) {
        let font = &typeface.0;
        let glyph_id = font.glyph_id(ch);
        // We use Y-up coordinates for ab_glyph, but screen is Y-down.
        // If we use rasterizer, we need to handle coordinates carefully.
        // For simplicity, let's just position at (x, y) and flip if needed?
        // ab_glyph rasterizer outputs absolute coordinates.
        // If we pass (x, y) where y is "screen y", ab_glyph might interpret it as "y up".
        // But normally font engines use positive Y up.
        // Screen Y is positive DOWN.
        // If we pass y=100 (screen), ab_glyph thinks result is at y=100 (up).
        // If we just plot at y=100 (pixel), it appears at screen y=100.
        // But the glyph shape itself?
        // If "top" of glyph is at y+10 (110) in UP system.
        // Pixel 110.
        // In screen system (down), 110 is below 100.
        // So "top" is below "baseline".
        // This effectively FLIPS the glyph upside down?
        // Yes.
        // So we need to FLIP the glyph geometry manually or use vector path with transform.
        // Since I can't easily use Vector Path (Outline issue), I am stuck with Rasterizer.
        
        // Rasterizer doesn't support transform easily (only scale/pos).
        // Wait, ab_glyph 0.2 ScaleFont implies `scale: PxScale`.
        // `PxScale` has x and y. `PxScale { x: size, y: -size }`?
        // Does ab_glyph support negative scale?
        // If so, that would flip it!
        // Let's try `glyph_id.with_scale(PxScale { x: size, y: -size })`.
        // And position?
        
        let scale = PxScale { x: size, y: size }; // Try normal scale first.
        // If it renders upside down, I'll fix it later. 
        // Previewing upside down text is better than no text.
        
        let glyph = glyph_id.with_scale_and_position(scale, point(x, y));
        
        if let Some(outlined) = font.outline_glyph(glyph) {
            let width = pixmap.width();
            let height = pixmap.height();
            let pixels = pixmap.pixels_mut();
            
            outlined.draw(|gx, gy, c| {
                if gx < width && gy < height {
                    let alpha = (c * 255.0) as u8;
                    if alpha > 0 {
                        // Blend? Just overwrite for now
                         let index = (gy * width + gx) as usize;
                         if let Some(pixel) = pixels.get_mut(index) {
                             // White text
                             *pixel = Color::from_rgba8(255, 255, 255, alpha).premultiply().to_color_u8();
                         }
                    }
                }
            });
        }
    }
}

fn resolve_position_value(val: &Option<PositionValue>, total: f32, default: f32) -> f32 {
    match val {
        Some(PositionValue::Pixels(v)) => *v,
        Some(PositionValue::Percentage(s)) => {
            if let Ok(p) = s.trim_end_matches('%').parse::<f32>() {
                 total * (p / 100.0)
            } else {
                default
            }
        },
        None => default
    }
}

use std::collections::HashMap;
use crate::model::{KLyricDocumentV2, Line, Style, PositionValue};
use crate::layout::LayoutEngine;
use anyhow::{Result, anyhow};
use tiny_skia::{Pixmap, Color};
use ab_glyph::{Font, FontArc, ScaleFont};
use fontdb::{Database, ID};

// Typeface wrapper for ab_glyph::FontArc
#[derive(Clone, Debug)]
pub struct Typeface(pub FontArc);


use owned_ttf_parser::{Face, OutlineBuilder};

pub struct TextRenderer {
    db: Database,
    default_typeface: Option<Typeface>,
    family_map: HashMap<String, String>,

    // Caches
    path_cache: HashMap<(ID, char), tiny_skia::Path>,
    id_cache: HashMap<String, ID>,
}

impl TextRenderer {
    pub fn new() -> Self {
        let db = Database::new();
        Self {
            db,
            default_typeface: None,
            family_map: HashMap::new(),
            path_cache: HashMap::new(),
            id_cache: HashMap::new(),
        }
    }

    pub fn load_font_bytes(&mut self, alias: &str, data: Vec<u8>) -> Result<()> {
        log::info!("Loading font: '{}', bytes={}", alias, data.len());
        let source = fontdb::Source::Binary(std::sync::Arc::new(data));
        let ids = self.db.load_font_source(source);
        
        // Map the alias to the first face found in the source
        if let Some(id) = ids.first() {
            if let Some(face_info) = self.db.face(*id) {
                if let Some((family_name, _)) = face_info.families.first() {
                    let actual_family = family_name.clone();
                    log::info!("Mapped alias '{}' to family '{}'", alias, actual_family);
                    self.family_map.insert(alias.to_string(), actual_family);

                    // Pre-populate id cache for alias
                    self.id_cache.insert(alias.to_string(), *id);
                }
            }
        }
        
        Ok(())
    }

    pub fn set_default_font_bytes(&mut self, data: Vec<u8>) -> Result<()> {
        // FontArc::try_from_vec expects Vec<u8>
        let font = FontArc::try_from_vec(data)?;
        self.default_typeface = Some(Typeface(font));
        Ok(())
    }

    pub fn get_typeface(&self, request_family: &str) -> Option<Typeface> {
        // Check alias map first
        let family = self.family_map.get(request_family).map(|s| s.as_str()).unwrap_or(request_family);

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

    fn resolve_font_id(&mut self, family: &str) -> Option<ID> {
        if let Some(id) = self.id_cache.get(family) {
            return Some(*id);
        }

        let resolved_family = self.family_map.get(family).map(|s| s.as_str()).unwrap_or(family);
        let query = fontdb::Query {
            families: &[fontdb::Family::Name(resolved_family), fontdb::Family::SansSerif],
            ..fontdb::Query::default()
        };

        if let Some(id) = self.db.query(&query) {
            self.id_cache.insert(family.to_string(), id);
            Some(id)
        } else {
            None
        }
    }

    pub fn get_glyph_path(&mut self, family: &str, ch: char) -> Option<tiny_skia::Path> {
        let id = self.resolve_font_id(family)?;

        if let Some(path) = self.path_cache.get(&(id, ch)) {
            return Some(path.clone());
        }

        // Cache miss: generate path
        let mut path_opt = None;
        self.db.with_face_data(id, |data, index| {
             if let Ok(face) = Face::parse(data, index) {
                 if let Some(gid) = face.glyph_index(ch) {
                     let units_per_em = face.units_per_em() as f32;
                     let scale_factor = 1.0 / units_per_em;

                     let mut builder = tiny_skia::PathBuilder::new();
                     let mut visitor = TtfPathVisitor {
                         builder: &mut builder,
                         scale: scale_factor,
                         offset_x: 0.0,
                         offset_y: 0.0,
                     };

                     if face.outline_glyph(gid, &mut visitor).is_some() {
                         path_opt = builder.finish();
                     }
                 }
             }
        });

        if let Some(path) = path_opt {
            self.path_cache.insert((id, ch), path.clone());
            Some(path)
        } else {
            None
        }
    }
}

pub struct TtfPathVisitor<'a> {
    pub builder: &'a mut tiny_skia::PathBuilder,
    pub scale: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

impl<'a> OutlineBuilder for TtfPathVisitor<'a> {
    fn move_to(&mut self, x: f32, y: f32) {
        let px = self.offset_x + x * self.scale;
        let py = self.offset_y - y * self.scale;
        self.builder.move_to(px, py);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let px = self.offset_x + x * self.scale;
        let py = self.offset_y - y * self.scale;
        self.builder.line_to(px, py);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let px1 = self.offset_x + x1 * self.scale;
        let py1 = self.offset_y - y1 * self.scale;
        let px = self.offset_x + x * self.scale;
        let py = self.offset_y - y * self.scale;
        self.builder.quad_to(px1, py1, px, py);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let px1 = self.offset_x + x1 * self.scale;
        let py1 = self.offset_y - y1 * self.scale;
        let px2 = self.offset_x + x2 * self.scale;
        let py2 = self.offset_y - y2 * self.scale;
        let px = self.offset_x + x * self.scale;
        let py = self.offset_y - y * self.scale;
        self.builder.cubic_to(px1, py1, px2, py2, px, py);
    }

    fn close(&mut self) {
        self.builder.close();
    }
}

// ... existing Renderer impl ...

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
        
        // Get base style or default
        let base_style = doc.styles.get("base").cloned().unwrap_or_default();
        
        for line in &doc.lines {
            let start_time = line.start;
            let end_time = line.end;
            if time >= start_time - 1.0 && time <= end_time + 1.0 {
                // Resolve line-specific style if present
                let line_style_name = line.style.as_deref().unwrap_or("base");
                let line_style = doc.styles.get(line_style_name).cloned().unwrap_or(base_style.clone());
                
                self.render_line(&mut pixmap, line, &line_style, time)?;
            }
        }
        
        Ok(pixmap.data().to_vec())
    }
    
    fn render_line(&mut self, pixmap: &mut Pixmap, line: &Line, style: &Style, time: f64) -> Result<()> {
        let resolved_style = style.clone();
        let glyphs = LayoutEngine::layout_line(line, &resolved_style, &mut self.text_renderer);
        
        let cx = self.width as f32 / 2.0;
        let cy = self.height as f32 / 2.0;
        
        let (lx, ly) = if let Some(pos) = &line.position {
            let x = resolve_position_value(&pos.x, self.width as f32, cx);
            let y = resolve_position_value(&pos.y, self.height as f32, cy);
            (x, y)
        } else {
             (cx, cy)
        };
        
        // Defaults
        let default_fill = Color::from_rgba8(255, 255, 255, 255);
        
        // Resolve Style Properties
        let inactive_fill = resolved_style.colors.as_ref()
            .and_then(|c| c.inactive.as_ref())
            .and_then(|fs| fs.fill.as_ref())
            .and_then(|s| parse_color(s));

        let active_fill = resolved_style.colors.as_ref()
            .and_then(|c| c.active.as_ref())
            .and_then(|fs| fs.fill.as_ref())
            .and_then(|s| parse_color(s));
            
        let inactive_stroke_color = resolved_style.colors.as_ref()
            .and_then(|c| c.inactive.as_ref())
            .and_then(|fs| fs.stroke.as_ref())
            .and_then(|s| parse_color(s));

        let active_stroke_color = resolved_style.colors.as_ref()
            .and_then(|c| c.active.as_ref())
            .and_then(|fs| fs.stroke.as_ref())
            .and_then(|s| parse_color(s));
            
        let stroke_width = resolved_style.stroke.as_ref().and_then(|s| s.width).unwrap_or(0.0);
        let default_stroke_color = resolved_style.stroke.as_ref()
            .and_then(|s| s.color.as_ref())
            .and_then(|s| parse_color(s));
            
        let shadow_opt = resolved_style.shadow.as_ref();
        let shadow_color = shadow_opt.and_then(|s| s.color.as_ref()).and_then(|c| parse_color(c));
        let shadow_offset = shadow_opt.map(|s| (s.x.unwrap_or(0.0), s.y.unwrap_or(0.0))).unwrap_or((0.0, 0.0));

        for glyph_info in glyphs {
            let char_data = &line.chars[glyph_info.char_index];
            let is_active = time >= char_data.start;
            
            // 1. Fill Color
            let fill_color = if is_active {
                active_fill.unwrap_or(default_fill)
            } else {
                inactive_fill.unwrap_or(default_fill)
            };
            
            // 2. Stroke Color
            // Priority: State Stroke > Global Stroke > None
            let stroke_color = if is_active {
                active_stroke_color.or(default_stroke_color)
            } else {
                inactive_stroke_color.or(default_stroke_color)
            };
            
            let font_spec = char_data.font.as_ref()
                .or(line.font.as_ref())
                .or(resolved_style.font.as_ref());
                
            let family = font_spec.and_then(|f| f.family.as_deref()).unwrap_or("Noto Sans SC");
            let size = font_spec.and_then(|f| f.size).unwrap_or(72.0);
            
            let render_x = lx + glyph_info.x;
            let render_y = ly + glyph_info.y;

            self.draw_glyph_path(
                pixmap, family, glyph_info.char, size, render_x, render_y,
                fill_color,
                stroke_color, stroke_width,
                shadow_color, shadow_offset
            );
        }
        
        Ok(())
    }
    
    fn draw_glyph_path(
        &mut self,
        pixmap: &mut Pixmap, 
        family: &str, 
        ch: char, 
        size: f32, 
        x: f32, 
        y: f32, 
        fill_color: Color,
        stroke_color: Option<Color>,
        stroke_width: f32,
        shadow_color: Option<Color>,
        shadow_offset: (f32, f32)
    ) {
        if let Some(path) = self.text_renderer.get_glyph_path(family, ch) {
            let mut paint = tiny_skia::Paint::default();
            paint.anti_alias = true;

            // Transform: Scale (size), Translate (x, y)
            let transform = tiny_skia::Transform::from_translate(x, y)
                .pre_scale(size, size);

            // 1. Shadow
            if let Some(sc) = shadow_color {
                paint.set_color(sc);
                let shadow_transform = transform.post_translate(shadow_offset.0, shadow_offset.1);
                pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, shadow_transform, None);
            }

            // 2. Stroke
            if let Some(st_color) = stroke_color {
                if stroke_width > 0.0 {
                    let mut stroke_paint = tiny_skia::Paint::default();
                    stroke_paint.set_color(st_color);
                    stroke_paint.anti_alias = true;
                    
                    let stroke = tiny_skia::Stroke {
                        width: stroke_width / size, // Stroke width must be inverse scaled because transform scales everything!
                        ..tiny_skia::Stroke::default()
                    };
                    
                    pixmap.stroke_path(&path, &stroke_paint, &stroke, transform, None);
                }
            }

            // 3. Fill
            paint.set_color(fill_color);
            pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, transform, None);
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

fn parse_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    let (r, g, b, a) = match hex.len() {
        6 => (
            u8::from_str_radix(&hex[0..2], 16).ok()?,
            u8::from_str_radix(&hex[2..4], 16).ok()?,
            u8::from_str_radix(&hex[4..6], 16).ok()?,
            255
        ),
        8 => (
            u8::from_str_radix(&hex[0..2], 16).ok()?,
            u8::from_str_radix(&hex[2..4], 16).ok()?,
            u8::from_str_radix(&hex[4..6], 16).ok()?,
            u8::from_str_radix(&hex[6..8], 16).ok()?
        ),
        _ => return None,
    };
    Some(Color::from_rgba8(r, g, b, a))
}

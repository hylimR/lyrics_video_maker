use anyhow::Result;
use tiny_skia::{Pixmap, Color, PixmapPaint, Transform as SkiaTransform, Stroke as SkiaStroke, Paint};
use std::collections::HashSet;

use crate::model::{KLyricDocumentV2, Line, PositionValue, EffectType, Transform, Easing};
use crate::style::StyleResolver;
use crate::layout::LayoutEngine;
use crate::text::TextRenderer;
use crate::effects::{EffectEngine, TriggerContext};
use crate::presets::CharBounds;

use super::particle_system::ParticleRenderSystem;
use super::utils::{parse_color, parse_percentage};

/// Default colors for karaoke states when not specified in style
const DEFAULT_INACTIVE_COLOR: &str = "#888888";  // Dimmed gray
const DEFAULT_ACTIVE_COLOR: &str = "#FFFF00";    // Bright yellow
const DEFAULT_COMPLETE_COLOR: &str = "#FFFFFF";  // White

pub struct LineRenderer<'a> {
    pub pixmap: &'a mut Pixmap,
    pub doc: &'a KLyricDocumentV2,
    pub time: f64,
    pub text_renderer: &'a mut TextRenderer,
    pub particle_system: &'a mut ParticleRenderSystem,
    pub active_keys: &'a mut HashSet<String>,
    pub width: u32,
    pub height: u32,
}

impl<'a> LineRenderer<'a> {
    pub fn render_line(&mut self, line: &Line, line_idx: usize) -> Result<()> {
        let resolver = StyleResolver::new(self.doc);
        let style_name = line.style.as_deref().unwrap_or("base");
        let style = resolver.resolve(style_name);
        
        // Layout Text
        let glyphs = LayoutEngine::layout_line(line, &style, self.text_renderer);

        // DEBUG: Only log for the first few frames/lines to avoid spam
        if self.time < 1.0 { // Arbitrary small time check to limit logs
             println!("🔍 Rust: render_line for '{:?}'. Time: {:.2}. Glyphs: {}", line.text, self.time, glyphs.len());
        }
        
        // Compute Line Position
        let (base_x, base_y) = self.compute_line_position(line);

        // Resolve font size for rasterization (Base)
        let style_family = style.font.as_ref().map(|f| f.family.as_str()).unwrap_or("Noto Sans SC");
        let style_size = style.font.as_ref().map(|f| f.size).unwrap_or(72.0);
        
        // Loop:
        for (idx, glyph) in glyphs.iter().enumerate() {
             let char_absolute_x = base_x + glyph.x;
             let char_absolute_y = base_y + glyph.y;
             
             // Resolve Font for THIS glyph (matches layout logic)
             let char_data = line.chars.get(glyph.char_index);
             let (family, size) = if let Some(c) = char_data.and_then(|c| c.font.as_ref()) {
                 (c.family.as_str(), c.size)
             } else if let Some(l) = &line.font {
                 (l.family.as_str(), l.size)
             } else {
                 (style_family, style_size)
             };

             let (w, h, alpha_pixels) = {
                 let font = self.text_renderer.get_font(family)
                     .or_else(|| self.text_renderer.get_default_font());
                 
                 if let Some(font) = font {
                     self.text_renderer.rasterize_char(&font, glyph.char, size)
                        .unwrap_or((0, 0, Vec::new()))
                 } else {
                     if self.time < 0.1 && idx == 0 {
                        println!("❌ Rust: Font not found for '{}' (fallback also failed)", family);
                     }
                     (0, 0, Vec::new())
                 }
             };

             if self.time < 0.1 && idx == 0 {
                 println!("🔍 Rust: Char '{}' size: {}x{}. Pos: ({:.1}, {:.1})", glyph.char, w, h, char_absolute_x, char_absolute_y);
             }

             if w > 0 && h > 0 {
                 if let Some(_) = Pixmap::new(w, h) {
                     // Resolve color
                     let is_active = char_data.map(|c| self.time >= c.start && self.time <= c.end).unwrap_or(false);
                     let is_past = char_data.map(|c| self.time > c.end).unwrap_or(false);
                     
                     let color_hex = if is_past {
                         style.colors.as_ref()
                             .and_then(|c| c.complete.as_ref())
                             .and_then(|fs| fs.fill.as_deref())
                             .unwrap_or(DEFAULT_COMPLETE_COLOR)
                     } else if is_active {
                         style.colors.as_ref()
                             .and_then(|c| c.active.as_ref())
                             .and_then(|fs| fs.fill.as_deref())
                             .unwrap_or(DEFAULT_ACTIVE_COLOR)
                     } else {
                         style.colors.as_ref()
                             .and_then(|c| c.inactive.as_ref())
                             .and_then(|fs| fs.fill.as_deref())
                             .unwrap_or(DEFAULT_INACTIVE_COLOR)
                     };
                     let text_color = parse_color(color_hex).unwrap_or(Color::WHITE);

                     // Helper to create colored pixmap from alpha
                     let create_colored_pixmap = |color: Color| -> Option<Pixmap> {
                         let mut pm = Pixmap::new(w, h)?;
                         let data = pm.data_mut();
                         let r = (color.red() * 255.0) as u8;
                         let g = (color.green() * 255.0) as u8;
                         let b = (color.blue() * 255.0) as u8;
                         let a_base = color.alpha();

                         for (i, &alpha) in alpha_pixels.iter().enumerate() {
                             let idx = i * 4;
                             if alpha > 0 {
                                 let final_alpha = (alpha as f32 / 255.0) * a_base;
                                 let fa = final_alpha;
                                 data[idx] = (r as f32 * fa) as u8;
                                 data[idx+1] = (g as f32 * fa) as u8;
                                 data[idx+2] = (b as f32 * fa) as u8;
                                 data[idx+3] = (final_alpha * 255.0) as u8;
                             }
                         }
                         Some(pm)
                     };

                     let draw_x = char_absolute_x as i32;
                     let draw_y = (char_absolute_y - h as f32 / 2.0) as i32;
                     let cx = w as f32 / 2.0;
                     let cy = h as f32 / 2.0;

                     // Compute Transform (Base + Effects)
                     let line_transform = line.transform.clone().unwrap_or_default();
                     let char_transform = char_data.and_then(|c| c.transform.clone()).unwrap_or_default();
                     
                     let base_transform = Transform {
                         x: line_transform.x + char_transform.x,
                         y: line_transform.y + char_transform.y,
                         rotation: line_transform.rotation + char_transform.rotation,
                         scale: line_transform.scale * char_transform.scale,
                         scale_x: line_transform.scale_x * char_transform.scale_x,
                         scale_y: line_transform.scale_y * char_transform.scale_y,
                         opacity: line_transform.opacity * char_transform.opacity,
                         anchor_x: char_transform.anchor_x,
                         anchor_y: char_transform.anchor_y,
                     };
                     
                     // Resolve effects
                     let mut active_effects = Vec::new();
                     for effect_name in &line.effects {
                        if let Some(effect) = self.doc.effects.get(effect_name) {
                            // Check if it is a preset wrapper
                            if let Some(preset_name) = &effect.preset {
                                if let Some(mut generated) = crate::presets::transitions::get_transition(preset_name) {
                                     // Override duration if specified in wrapper
                                     if let Some(d) = effect.duration {
                                         generated.duration = Some(d);
                                     }
                                     // Override easing if specified (only if not default Linear)
                                     if effect.easing != Easing::Linear {
                                         generated.easing = effect.easing.clone();
                                     }

                                     active_effects.push((effect_name, generated));
                                } else {
                                     // Fallback or particle preset
                                     active_effects.push((effect_name, effect.clone()));
                                }
                            } else {
                                active_effects.push((effect_name, effect.clone()));
                            }
                        } else if let Some(preset) = crate::presets::transitions::get_transition(effect_name) {
                            // Support preset transition names directly
                            active_effects.push((effect_name, preset));
                        }
                     }

                     let ctx = TriggerContext {
                         start_time: line.start,
                         end_time: line.end,
                         current_time: self.time,
                         active: true,
                     };
                     
                     let mut transform_effects = Vec::new();
                     let mut particle_effects = Vec::new();
                     let mut disintegrate_effects = Vec::new();

                     for (name, effect) in active_effects {
                         match effect.effect_type {
                             EffectType::Particle => particle_effects.push((name, effect)),
                             EffectType::Disintegrate => disintegrate_effects.push((name, effect)),
                             _ => transform_effects.push(effect),
                         }
                     }
                     
                     let final_transform = EffectEngine::compute_transform(
                         self.time,
                         &base_transform,
                         &transform_effects,
                         ctx.clone()
                     );

                     // Disintegrate Effect Progress
                     let mut disintegration_progress = 0.0;
                     if let Some((_name, effect)) = disintegrate_effects.first() {
                         if EffectEngine::should_trigger(effect, &ctx) {
                             disintegration_progress = EffectEngine::calculate_progress(self.time, effect, &ctx);
                             // Clamp progress
                             disintegration_progress = disintegration_progress.max(0.0).min(1.0);

                             // Spawn particles if needed (only at the very beginning)
                             // Since render_line is called every frame, we rely on ensure_disintegration_emitter
                             // to only create them once per key.
                             // However, we need the pixmap to be ready. It is ready here (text_pm).

                             // We do the spawning AFTER pixmap creation below, but we need to track if we should fade out the text here.
                         }
                     }

                     // Common Paint
                     let mut paint = PixmapPaint::default();
                     // Fade out text as disintegration progresses
                     paint.opacity = final_transform.opacity * (1.0 - disintegration_progress as f32);

                     // --- 1. SHADOW ---
                     let shadow_opts = if let Some(c) = char_data.and_then(|c| c.shadow.as_ref()) { Some(c) } 
                                       else if let Some(l) = &line.shadow { Some(l) } 
                                       else { style.shadow.as_ref() };

                     if let Some(shadow) = shadow_opts {
                         if let Some(color_hex) = &shadow.color {
                             if let Some(shadow_color) = parse_color(color_hex) {
                                 if let Some(shadow_pm) = create_colored_pixmap(shadow_color) {
                                     let sx = draw_x as f32 + final_transform.x + shadow.x;
                                     let sy = draw_y as f32 + final_transform.y + shadow.y;
                                     
                                     let mut st = SkiaTransform::identity();
                                     st = st.pre_translate(sx, sy);
                                     st = st.pre_translate(cx, cy);
                                     st = st.pre_rotate(final_transform.rotation);
                                     st = st.pre_scale(final_transform.scale * final_transform.scale_x, final_transform.scale * final_transform.scale_y);
                                     st = st.pre_translate(-cx, -cy);
                                     
                                     self.pixmap.draw_pixmap(0, 0, shadow_pm.as_ref(), &paint, st, None);
                                 }
                             }
                         }
                     }

                     // --- 2. STROKE (Real Path) ---
                     let stroke_opts = if let Some(c) = char_data.and_then(|c| c.stroke.as_ref()) { Some(c) } 
                                       else if let Some(l) = &line.stroke { Some(l) } 
                                       else { style.stroke.as_ref() };

                     if let Some(stroke) = stroke_opts {
                         if stroke.width > 0.0 {
                             if let Some(color_hex) = &stroke.color {
                                 if let Some(stroke_color) = parse_color(color_hex) {
                                     // Get glyph path for stroke
                                     let font = self.text_renderer.get_font(family)
                                         .or_else(|| self.text_renderer.get_default_font());

                                     if let Some(font) = font {
                                         if let Some(path) = self.text_renderer.get_glyph_path(&font, glyph.char, size) {
                                             // Calculate Stroke Paint
                                             let mut stroke_paint = Paint::default();
                                             // Scale alpha by paint opacity
                                             let alpha = stroke_color.alpha() * paint.opacity;
                                             let mut final_color = stroke_color;
                                             final_color.set_alpha(alpha);

                                             stroke_paint.set_color(final_color);
                                             stroke_paint.anti_alias = true;

                                             let mut skia_stroke = SkiaStroke::default();
                                             skia_stroke.width = stroke.width;

                                             // Transform for path
                                             // The path is relative to the baseline origin (0,0) of the glyph.
                                             // We need to position it at (char_absolute_x, char_absolute_y).
                                             // And apply line effects.

                                             // char_absolute_x/y seems to be the pen position (baseline).
                                             let screen_x = char_absolute_x + final_transform.x;
                                             let screen_y = char_absolute_y + final_transform.y;

                                             // TextRenderer::get_glyph_path returns path with Y flipped (growing down), origin at (0,0).
                                             // Usually (0,0) is baseline.
                                             
                                             // Import Font trait if not present, but better just use available methods.
                                             // font is FontRef.
                                             use ab_glyph::{Font, ScaleFont};
                                             let scaled_font = font.as_scaled(ab_glyph::PxScale::from(size));
                                             // Ensure we use the correct font methods
                                             let outlined = scaled_font.font.outline_glyph(scaled_font.scaled_glyph(glyph.char));
                                             
                                             if let Some(outlined) = outlined {
                                                  let bounds = outlined.px_bounds();

                                                  // Now we are in "Bitmap Top-Left" space.
                                                  // Shift path to be relative to this space.
                                                  let mut st = SkiaTransform::identity();
                                                  st = st.pre_translate(screen_x, screen_y);
                                                  st = st.pre_translate(cx, cy);
                                                  st = st.pre_rotate(final_transform.rotation);
                                                  st = st.pre_scale(
                                                     final_transform.scale * final_transform.scale_x,
                                                     final_transform.scale * final_transform.scale_y
                                                  );
                                                  st = st.pre_translate(-cx, -cy);
                                                  st = st.pre_translate(-bounds.min.x, -bounds.min.y);

                                                  self.pixmap.stroke_path(&path, &stroke_paint, &skia_stroke, st, None);
                                             }
                                         }
                                     }
                                 }
                             }
                         }
                     }

                     // --- 3. MAIN TEXT ---
                     // Fill glyph
                     if let Some(text_pm) = create_colored_pixmap(text_color) {
                         // Only draw text if not fully disintegrated
                         if paint.opacity > 0.01 {
                             let screen_x = draw_x as f32 + final_transform.x;
                             let screen_y = draw_y as f32 + final_transform.y;

                             let mut skia_transform = SkiaTransform::identity();
                             skia_transform = skia_transform.pre_translate(screen_x, screen_y);
                             skia_transform = skia_transform.pre_translate(cx, cy);
                             skia_transform = skia_transform.pre_rotate(final_transform.rotation);
                             skia_transform = skia_transform.pre_scale(
                                 final_transform.scale * final_transform.scale_x,
                                 final_transform.scale * final_transform.scale_y
                             );
                             skia_transform = skia_transform.pre_translate(-cx, -cy);

                             self.pixmap.draw_pixmap(
                                 0,
                                 0,
                                 text_pm.as_ref(),
                                 &paint,
                                 skia_transform,
                                 None
                             );
                         }

                         // --- DISINTEGRATION EFFECT ---
                         for (name, effect) in disintegrate_effects {
                         if !EffectEngine::should_trigger(&effect, &ctx) { continue; }

                         let progress = EffectEngine::calculate_progress(self.time, &effect, &ctx);
                             if progress < 0.0 || progress > 1.0 { continue; }

                             let key = format!("{}_{}_{}", line_idx, glyph.char_index, name);
                             self.active_keys.insert(key.clone());

                             // Calculate screen bounds for the emitter
                             let bounds = CharBounds {
                                 x: char_absolute_x + final_transform.x,
                                 y: char_absolute_y + final_transform.y - h as f32 / 2.0,
                                 width: w as f32 * final_transform.scale * final_transform.scale_x,
                                 height: h as f32 * final_transform.scale * final_transform.scale_y,
                             };

                             let seed = (line_idx * 1000 + glyph.char_index * 100) as u64;

                             // Trigger creation using the current text pixmap
                             self.particle_system.ensure_disintegration_emitter(
                                 key,
                                 &text_pm,
                                 bounds,
                                 seed,
                                 effect.particle_config.clone()
                             );
                         }
                     }
                     
                     // --- PARTICLE SPAWNING ---
                     for (name, effect) in particle_effects {
                         if !EffectEngine::should_trigger(&effect, &ctx) { continue; }
                         
                         let progress = EffectEngine::calculate_progress(self.time, &effect, &ctx);
                         if progress < 0.0 || progress > 1.0 { continue; }
                         
                         let key = format!("{}_{}_{}", line_idx, glyph.char_index, name);
                         self.active_keys.insert(key.clone());
                         
                         let bounds = CharBounds {
                             x: char_absolute_x + final_transform.x,  
                             y: char_absolute_y + final_transform.y - h as f32 / 2.0, 
                             width: w as f32 * final_transform.scale * final_transform.scale_x,
                             height: h as f32 * final_transform.scale * final_transform.scale_y,
                         };
                         
                         let seed = (line_idx * 1000 + glyph.char_index * 100) as u64;
                         
                         // Pass preset name string directly, let factory handle lookup
                         self.particle_system.ensure_emitter(
                             key, 
                             effect.preset.clone(), 
                             effect.particle_config.clone(), 
                             bounds, 
                             seed
                         );
                     }
                 }
             }
        }
        
        Ok(())
    }
    
    fn compute_line_position(&self, line: &Line) -> (f32, f32) {
        let mut x = self.width as f32 / 2.0;
        let mut y = self.height as f32 / 2.0;
        
        if let Some(pos) = &line.position {
            if let Some(px) = &pos.x {
                x = match px {
                    PositionValue::Pixels(v) => *v,
                    PositionValue::Percentage(s) => parse_percentage(s) * self.width as f32,
                };
            }
             if let Some(py) = &pos.y {
                y = match py {
                    PositionValue::Pixels(v) => *v,
                    PositionValue::Percentage(s) => parse_percentage(s) * self.height as f32,
                };
            }
        }
        
        (x, y)
    }
}

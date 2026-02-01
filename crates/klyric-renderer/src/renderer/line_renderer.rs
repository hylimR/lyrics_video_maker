use anyhow::Result;
use skia_safe::{Canvas, Color, Paint, BlendMode, PaintStyle, MaskFilter, BlurStyle, surfaces};
use std::collections::HashSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::model::{KLyricDocumentV2, Line, PositionValue, EffectType, Transform, Easing, RenderTransform, Style};
use crate::layout::LayoutEngine;
use crate::text::TextRenderer;
use crate::effects::{EffectEngine, TriggerContext};
use crate::presets::CharBounds;
use crate::expressions::{EvaluationContext, FastEvaluationContext};

use super::particle_system::ParticleRenderSystem;
use super::utils::parse_color;
use super::CategorizedLineEffects;

/// Default colors for karaoke states when not specified in style
const DEFAULT_INACTIVE_COLOR: &str = "#888888";  // Dimmed gray
const DEFAULT_ACTIVE_COLOR: &str = "#FFFF00";    // Bright yellow
const DEFAULT_COMPLETE_COLOR: &str = "#FFFFFF";  // White

pub struct LineRenderer<'a> {
    pub canvas: &'a Canvas,
    pub doc: &'a KLyricDocumentV2,
    pub time: f64,
    pub text_renderer: &'a mut TextRenderer,
    pub particle_system: &'a mut ParticleRenderSystem,
    pub active_keys: &'a mut HashSet<u64>,
    pub width: u32,
    pub height: u32,
}

/// Hash an emitter key from components to produce u64 for particle system
fn hash_emitter_key(line_idx: usize, char_idx: usize, name: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    line_idx.hash(&mut hasher);
    char_idx.hash(&mut hasher);
    name.hash(&mut hasher);
    hasher.finish()
}

impl<'a> LineRenderer<'a> {
    pub fn render_line(&mut self, line: &Line, line_idx: usize, style: &Style, effects: &'a CategorizedLineEffects) -> Result<()> {
        // Layout Text
        let glyphs = LayoutEngine::layout_line(line, style, self.text_renderer);

        // Compute Line Position
        let (base_x, base_y) = self.compute_line_position(line);

        // Resolve font size for rasterization (Base)
        let style_family = style.font.as_ref().and_then(|f| f.family.as_deref()).unwrap_or("Noto Sans SC");
        let style_size = style.font.as_ref().and_then(|f| f.size).unwrap_or(72.0);
        
        // Pre-resolve colors to avoid parsing per-glyph
        let inactive_hex = style.colors.as_ref()
            .and_then(|c| c.inactive.as_ref())
            .and_then(|fs| fs.fill.as_deref())
            .unwrap_or(DEFAULT_INACTIVE_COLOR);
        let inactive_color = parse_color(inactive_hex).unwrap_or(Color::WHITE);

        let active_hex = style.colors.as_ref()
            .and_then(|c| c.active.as_ref())
            .and_then(|fs| fs.fill.as_deref())
            .unwrap_or(DEFAULT_ACTIVE_COLOR);
        let active_color = parse_color(active_hex).unwrap_or(Color::WHITE);

        let complete_hex = style.colors.as_ref()
            .and_then(|c| c.complete.as_ref())
            .and_then(|fs| fs.fill.as_deref())
            .unwrap_or(DEFAULT_COMPLETE_COLOR);
        let complete_color = parse_color(complete_hex).unwrap_or(Color::WHITE);

        // Hoist Line Transform
        let line_transform = line.transform.clone().unwrap_or_default();

        // [Bolt Optimization] Hoist Paint and MaskFilter to avoid per-glyph allocation
        let mut paint = Paint::default();
        let mut shadow_paint = Paint::default();
        let mut stroke_paint = Paint::default();

        // Cache: (sigma, filter)
        let mut cached_blur_filter: Option<(f32, MaskFilter)> = None;

        // [Bolt Optimization] Hoist effect compilation and context creation
        // Calculate active effects and progress once per line.
        // Safety: Progress depends on (time, line.start, effect.delay).
        // Since effect.delay is scalar and we use line.start, progress is invariant for the line.
        // Per-character variations (staggering) are handled via expressions (evaluated per-char)
        // or specialized ops (like TypewriterLimit) which are preserved in compiled ops.
        let line_ctx = TriggerContext {
            start_time: line.start,
            end_time: line.end,
            current_time: self.time,
            active: true,
            char_index: None,
            char_count: Some(glyphs.len()),
        };

        let mut active_effects_progress: Vec<(&crate::effects::ResolvedEffect, f64)> = Vec::with_capacity(effects.transform_effects.len());
        for resolved in &effects.transform_effects {
            if EffectEngine::should_trigger(&resolved.effect, &line_ctx) {
                let p = EffectEngine::calculate_progress(self.time, &resolved.effect, &line_ctx);
                if (0.0..=1.0).contains(&p) {
                    active_effects_progress.push((resolved, EffectEngine::ease(p, &resolved.effect.easing)));
                }
            }
        }

        let compiled_ops = EffectEngine::compile_active_effects(&active_effects_progress, &line_ctx);

        // Reusable context for expression evaluation
        let eval_ctx = EvaluationContext {
            t: self.time,
            progress: 0.0, // Updated per effect/op
            width: self.width as f64,
            height: self.height as f64,
            index: None,
            count: Some(glyphs.len()),
            char_width: None,
            char_height: None,
        };
        let mut fast_ctx = FastEvaluationContext::new(&eval_ctx);

        // Pre-calculate base render transform for line (without char overrides)
        let line_render_transform = RenderTransform::new(&line_transform, &Transform::default());

        // [Bolt Optimization] Cache font resolution per-line to avoid redundant lookups
        let mut last_family: Option<&str> = None;
        let mut last_typeface: Option<skia_safe::Typeface> = None;

        // [Bolt Optimization] Pre-calculate Shadow/Stroke colors to avoid parsing inside loop
        let style_shadow_color = style.shadow.as_ref()
            .and_then(|s| s.color.as_deref())
            .and_then(parse_color);

        let line_shadow_color = line.shadow.as_ref()
            .and_then(|s| s.color.as_deref())
            .and_then(parse_color);

        let style_stroke_color = style.stroke.as_ref()
            .and_then(|s| s.color.as_deref())
            .and_then(parse_color);

        let line_stroke_color = line.stroke.as_ref()
            .and_then(|s| s.color.as_deref())
            .and_then(parse_color);

        // Loop:
        for glyph in glyphs.iter() {
             let char_absolute_x = base_x + glyph.x;
             let char_absolute_y = base_y + glyph.y;
             
             // Resolve Font for THIS glyph (matches layout logic)
             let char_data = line.chars.get(glyph.char_index);
             
             let family = char_data.and_then(|c| c.font.as_ref().and_then(|f| f.family.as_deref()))
                .or_else(|| line.font.as_ref().and_then(|f| f.family.as_deref()))
                .unwrap_or(style_family);

             let size = char_data.and_then(|c| c.font.as_ref().and_then(|f| f.size))
                .or_else(|| line.font.as_ref().and_then(|f| f.size))
                .unwrap_or(style_size);

             // Get typeface and path
             // Optimized font resolution: reuse typeface if family matches last iteration
             if last_family != Some(family) {
                 last_typeface = self.text_renderer.get_typeface(family)
                     .or_else(|| self.text_renderer.get_default_typeface());
                 last_family = Some(family);
             }
             
             if let Some(typeface) = &last_typeface {
                 // Get path
                 if let Some(path) = self.text_renderer.get_glyph_path(typeface, glyph.char, size) {
                     // Dimensions for effects
                     // Skia path bounds
                     let bounds = path.bounds();
                     let w = bounds.width();
                     let h = bounds.height();
                     // Helper midpoint
                     let cx = w / 2.0;
                     let cy = h / 2.0;

                     // Resolve color
                     let is_active = char_data.map(|c| self.time >= c.start && self.time <= c.end).unwrap_or(false);
                     let is_past = char_data.map(|c| self.time > c.end).unwrap_or(false);
                     
                     let text_color = if is_past {
                         complete_color
                     } else if is_active {
                         active_color
                     } else {
                         inactive_color
                     };

                     // Compute Transform (Base + Effects)
                     // [Bolt Optimization] Use RenderTransform and compiled ops
                     let mut final_transform = if let Some(c) = char_data {
                         let char_transform = c.transform.as_ref().cloned().unwrap_or_default();
                         RenderTransform::new(&line_transform, &char_transform)
                     } else {
                         line_render_transform.clone() // RenderTransform is Copy
                     };

                     // Update context
                     fast_ctx.set_index(glyph.char_index);

                     // Apply compiled effects
                     final_transform = EffectEngine::apply_compiled_ops(final_transform, &compiled_ops, &mut fast_ctx);
                     
                     // Need TriggerContext for layers/other effects
                     let ctx = TriggerContext {
                         start_time: line.start,
                         end_time: line.end,
                         current_time: self.time,
                         active: true,
                         char_index: Some(glyph.char_index),
                         char_count: Some(glyphs.len()),
                     };

                     // --- 4. MODIFIER LAYERS (New System) ---
                     if let Some(layers) = style.layers.as_ref() {
                         final_transform = EffectEngine::apply_layers_to_render(
                             self.time,
                             final_transform,
                             layers,
                             &ctx
                         );
                     }

                     // Disintegrate Effect Progress
                     let mut disintegration_progress = 0.0;
                     if let Some((_name, resolved_effect)) = effects.disintegrate_effects.first() {
                         let effect = &resolved_effect.effect;
                         if EffectEngine::should_trigger(effect, &ctx) {
                             disintegration_progress = EffectEngine::calculate_progress(self.time, effect, &ctx);
                             disintegration_progress = disintegration_progress.clamp(0.0, 1.0);
                         }
                     }

                     // Setup Paint
                     paint.reset(); // Reuse
                     paint.set_color(text_color);
                     paint.set_anti_alias(true);
                     
                     let final_opacity = final_transform.opacity * (1.0 - disintegration_progress as f32);
                     paint.set_alpha_f(final_opacity);

                     // Apply Blur with caching
                     let blur = final_transform.blur;
                     if blur > 0.0 {
                         let use_cached = if let Some((last_sigma, _)) = cached_blur_filter.as_ref() {
                             (last_sigma - blur).abs() < 0.001
                         } else {
                             false
                         };

                         if !use_cached {
                             // Create new filter
                             if let Some(filter) = MaskFilter::blur(BlurStyle::Normal, blur, false) {
                                 cached_blur_filter = Some((blur, filter));
                             } else {
                                 cached_blur_filter = None;
                             }
                         }

                         if let Some((_, ref filter)) = cached_blur_filter {
                             paint.set_mask_filter(Some(filter.clone()));
                         }
                     }

                     // --- DRAWING ---
                     // Calculate position context
                     let draw_x = char_absolute_x;
                     let draw_y = char_absolute_y;
                     
                     self.canvas.save();
                     
                     // Check for StrokeReveal
                     let mut stroke_reveal_progress = None;
                     for effect in &effects.stroke_reveal_effects {
                         if EffectEngine::should_trigger(effect, &ctx) {
                             let p = EffectEngine::calculate_progress(self.time, effect, &ctx);
                             stroke_reveal_progress = Some(p.clamp(0.0, 1.0));
                             break; // Only one stroke reveal supported
                         }
                     }

                     // Apply Transforms
                     self.canvas.translate((draw_x + final_transform.x, draw_y + final_transform.y));
                     
                     let path_center_x = bounds.center_x();
                     let path_center_y = bounds.center_y();
                     
                     self.canvas.translate((path_center_x, path_center_y));
                     self.canvas.rotate(final_transform.rotation, None);
                     self.canvas.scale((final_transform.scale * final_transform.scale_x, final_transform.scale * final_transform.scale_y));
                     self.canvas.translate((-path_center_x, -path_center_y));
                     
                     // Modify path if StrokeReveal is active
                     let draw_path = if let Some(progress) = stroke_reveal_progress {
                         let mut measure = skia_safe::PathMeasure::new(&path, false, None);
                         let length = measure.length();
                         if let Some(partial_path) = measure.segment(0.0, length * progress as f32, true) {
                             partial_path
                         } else {
                             path.clone()
                         }
                     } else {
                         path.clone() // Clone for drawing to avoid borrow issues? path is local
                     };
                     let path = &draw_path; // Re-bind path to modified version if needed

                     
                     // --- 1. SHADOW ---
                     let (active_shadow, active_shadow_color) = if let Some(c_shadow) = char_data.and_then(|c| c.shadow.as_ref()) {
                         (Some(c_shadow), c_shadow.color.as_deref().and_then(parse_color))
                     } else if let Some(l_shadow) = line.shadow.as_ref() {
                         (Some(l_shadow), line_shadow_color)
                     } else {
                         (style.shadow.as_ref(), style_shadow_color)
                     };

                     if let (Some(shadow), Some(shadow_color)) = (active_shadow, active_shadow_color) {
                         shadow_paint.reset();
                         shadow_paint.set_color(shadow_color);
                         shadow_paint.set_alpha_f(final_opacity);
                         shadow_paint.set_anti_alias(true);

                         // Apply blur to shadow if needed
                         if final_transform.blur > 0.0 {
                             if let Some((_, ref filter)) = cached_blur_filter {
                                 shadow_paint.set_mask_filter(Some(filter.clone()));
                             }
                         }

                         self.canvas.save();
                         self.canvas.translate((shadow.x_or_default(), shadow.y_or_default()));
                         self.canvas.draw_path(path, &shadow_paint);
                         self.canvas.restore();
                     }

                     // --- 2. STROKE ---
                     let (active_stroke, active_stroke_color) = if let Some(c_stroke) = char_data.and_then(|c| c.stroke.as_ref()) {
                         (Some(c_stroke), c_stroke.color.as_deref().and_then(parse_color))
                     } else if let Some(l_stroke) = line.stroke.as_ref() {
                         (Some(l_stroke), line_stroke_color)
                     } else {
                         (style.stroke.as_ref(), style_stroke_color)
                     };

                     if let (Some(stroke), Some(stroke_color)) = (active_stroke, active_stroke_color) {
                         if stroke.width_or_default() > 0.0 {
                             stroke_paint.reset();
                             stroke_paint.set_style(PaintStyle::Stroke);
                             stroke_paint.set_stroke_width(stroke.width_or_default());
                             stroke_paint.set_color(stroke_color);
                             stroke_paint.set_alpha_f(final_opacity);
                             stroke_paint.set_anti_alias(true);

                             if final_transform.blur > 0.0 {
                                 if let Some((_, ref filter)) = cached_blur_filter {
                                     stroke_paint.set_mask_filter(Some(filter.clone()));
                                 }
                             }

                             self.canvas.draw_path(path, &stroke_paint);
                         }
                     }

                     // --- 3. MAIN TEXT (With Glitch Logic) ---
                     if paint.alpha_f() > 0.001 {
                         if final_transform.glitch_offset.abs() > 0.01 {
                             // Glitch Effect: Draw channels separately
                             let offset = final_transform.glitch_offset;

                             // Red Channel
                             let mut r_paint = paint.clone();
                             r_paint.set_color(Color::from_argb((final_opacity * 255.0) as u8, 255, 0, 0));
                             r_paint.set_blend_mode(BlendMode::Plus); // Additive blending for RGB separation

                             // Green Channel
                             let mut g_paint = paint.clone();
                             g_paint.set_color(Color::from_argb((final_opacity * 255.0) as u8, 0, 255, 0));
                             g_paint.set_blend_mode(BlendMode::Plus);

                             // Blue Channel
                             let mut b_paint = paint.clone();
                             b_paint.set_color(Color::from_argb((final_opacity * 255.0) as u8, 0, 0, 255));
                             b_paint.set_blend_mode(BlendMode::Plus);

                             self.canvas.save();
                             self.canvas.translate((-offset, -offset));
                             self.canvas.draw_path(path, &r_paint);
                             self.canvas.restore();

                             self.canvas.save();
                             self.canvas.translate((offset, -offset)); // Different offset for G? Or just offset?
                             // Standard Chromatic Aberration: R: -off, B: +off, G: 0
                             self.canvas.draw_path(path, &g_paint); // Maybe G is at 0?
                             // Actually let's do: R at -offset, B at +offset, G at 0.
                             // But let's try to match glitch logic: jittery offsets.
                             self.canvas.restore();

                             self.canvas.save();
                             self.canvas.translate((offset, offset));
                             self.canvas.draw_path(path, &b_paint);
                             self.canvas.restore();

                             // Re-draw original white core? No, RGB additive makes white.
                             // But we need to handle non-white colors properly.
                             // If text is NOT white, splitting RGB is complex.
                             // For now assuming white text for glitch effect or simple displacement.

                         } else {
                             // Normal Draw
                             self.canvas.draw_path(path, &paint);
                         }
                     }
                     
                     self.canvas.restore(); // Restore transform for next glyph/effects

                     // --- DISINTEGRATION EFFECT ---
                     for (name, resolved_effect) in &effects.disintegrate_effects {
                         let effect = &resolved_effect.effect;
                         if !EffectEngine::should_trigger(effect, &ctx) { continue; }

                         let progress = EffectEngine::calculate_progress(self.time, effect, &ctx);
                         if !(0.0..=1.0).contains(&progress) { continue; }

                         let key = hash_emitter_key(line_idx, glyph.char_index, name);
                         self.active_keys.insert(key);

                         // We need to capture the glyph as an image for the emitter
                         // Create small surface
                         // Bounds might be slightly larger due to stroke/shadow, but let's stick to path bounds for particles
                         let capture_w = w.ceil() as i32 + 20; // Padding
                         let capture_h = h.ceil() as i32 + 20;
                         if capture_w <= 0 || capture_h <= 0 { continue; }
                         
                         // Create offscreen surface
                         // Note: creating surfaces every frame is expensive. 
                         // But disintegration usually only triggers ONCE per char.
                         // Optimization: Check if emitter exists already? 
                         // ParticleSystem does checks, but we shouldn't create surface if not needed.
                         // But we can't easily check particle system state from here without mutable borrow conflict?
                         // Actually active_keys insertion handles liveness.
                         // We should only generate if self.time is close to start?
                         // EffectEngine handles trigger/progress.
                         
                         // "ensure_disintegration_emitter" checks existence.
                         // But ideally we don't construct the Image if it exists.
                         // Let's rely on loose check or just pay the cost (it's fine for export).
                         
                         if self.particle_system.has_emitter(key) {
                             // Just update active
                             // But we can't access it here easily because self.particle_system is borrowed?
                             // No, we have &mut self in render_line.
                             // Actually we have separate borrows in Mod.rs.
                             // LineRenderer struct holds &mut separate fields.
                             // So yes we can check.
                             // Emitter exists - ensure_disintegration_emitter handles the active state internally
                              continue;
                         }

                         if let Some(mut surface) = surfaces::raster_n32_premul((capture_w, capture_h)) {
                             let c = surface.canvas();
                             // Center the path in the capture
                             let _tx = (capture_w as f32 / 2.0) - cx;
                             let _ty = (capture_h as f32 / 2.0) - cy; // - bounds.top?
                             // path bounds .top might be negative.
                             // bounds.y is usually negative (ascender).
                             // If bounds y is -50, height 70.
                             // We want to translate such that top-left of bounds is at (0,0)?
                             // Or center.
                             
                             let bounds_left = bounds.left;
                             let bounds_top = bounds.top;
                             
                             c.translate((-bounds_left + 10.0, -bounds_top + 10.0));
                             
                             // Draw path filled white
                             let mut cap_paint = Paint::default();
                             cap_paint.set_color(Color::WHITE);
                             cap_paint.set_anti_alias(true);
                             c.draw_path(path, &cap_paint);
                             
                             let image = surface.image_snapshot();

                             // Calculate screen bounds for the emitter
                             let bounds_rect = CharBounds {
                                 x: draw_x + final_transform.x + bounds_left - 10.0, // Adjust back
                                 y: draw_y + final_transform.y + bounds_top - 10.0,
                                 width: capture_w as f32 * final_transform.scale,
                                 height: capture_h as f32 * final_transform.scale,
                             };

                             let seed = (line_idx * 1000 + glyph.char_index * 100) as u64;

                             self.particle_system.ensure_disintegration_emitter(
                                 key,
                                 &image,
                                 bounds_rect,
                                 seed,
                                 effect.particle_config.clone()
                             );
                         }
                     }
                     
                     // --- PARTICLE SPAWNING ---
                     // Process standard particle effects
                     for (name, resolved_effect) in &effects.particle_effects {
                         let effect = &resolved_effect.effect;
                         if !EffectEngine::should_trigger(effect, &ctx) { continue; }
                         
                         let progress = EffectEngine::calculate_progress(self.time, effect, &ctx);
                         if !(0.0..=1.0).contains(&progress) { continue; }

                         let key = hash_emitter_key(line_idx, glyph.char_index, name);
                         self.active_keys.insert(key);
                         
                         let bounds_rect = CharBounds {
                             x: draw_x + final_transform.x + bounds.left,
                             y: draw_y + final_transform.y + bounds.top,
                             width: w * final_transform.scale,
                             height: h * final_transform.scale,
                         };

                         // [Bolt Optimization] Skip expensive config calculation if emitter exists.
                         // Note: `ensure_emitter` currently ignores config updates for existing emitters,
                         // so re-evaluating the config here is redundant work.
                         if self.particle_system.has_emitter(key) {
                             self.particle_system.update_emitter_bounds(key, bounds_rect);
                             continue;
                         }

                         // Evaluation Context for Particles (only needed for new emitters)
                         let eval_ctx = crate::expressions::EvaluationContext {
                             t: self.time,
                             progress,
                             index: Some(glyph.char_index),
                             count: Some(glyphs.len()),
                             ..Default::default()
                         };
                         
                         let seed = (line_idx * 1000 + glyph.char_index * 100) as u64;
                         
                         // Clone config and apply overrides
                         let mut p_config = effect.particle_config.clone();
                         if let (Some(config), Some(overrides)) = (&mut p_config, &effect.particle_override) {
                             let fast_ctx = crate::expressions::FastEvaluationContext::new(&eval_ctx);
                             // [Bolt Optimization] Use pre-compiled expressions
                             crate::particle::config::apply_particle_overrides(
                                 config,
                                 overrides,
                                 Some(&resolved_effect.compiled_expressions),
                                 &fast_ctx
                             );
                         }

                         self.particle_system.ensure_emitter(
                             key, 
                             effect.preset.clone(), 
                             p_config, 
                             bounds_rect, 
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
                    PositionValue::Percentage(p) => p * self.width as f32,
                };
            }
             if let Some(py) = &pos.y {
                y = match py {
                    PositionValue::Pixels(v) => *v,
                    PositionValue::Percentage(p) => p * self.height as f32,
                };
            }
        }
        
        (x, y)
    }
}

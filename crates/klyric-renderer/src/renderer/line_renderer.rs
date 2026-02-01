use anyhow::Result;
use skia_safe::{Canvas, Color, Paint, BlendMode, PaintStyle, MaskFilter, BlurStyle, surfaces};
use std::collections::HashSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::model::{KLyricDocumentV2, Line, PositionValue, EffectType, Transform, Easing, RenderTransform, Style};
use crate::layout::{LayoutEngine, GlyphInfo};
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
    pub fn render_line(&mut self, line: &Line, glyphs: &[GlyphInfo], line_idx: usize, style: &Style, effects: &'a CategorizedLineEffects) -> Result<()> {
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

        // [Bolt Optimization] Hoist Glitch Effect Paints to avoid 3x clones per char
        let mut r_paint = Paint::default();
        r_paint.set_color(Color::RED);
        r_paint.set_blend_mode(BlendMode::Plus);
        r_paint.set_anti_alias(true);
        r_paint.set_style(PaintStyle::Fill);

        let mut g_paint = Paint::default();
        g_paint.set_color(Color::GREEN);
        g_paint.set_blend_mode(BlendMode::Plus);
        g_paint.set_anti_alias(true);
        g_paint.set_style(PaintStyle::Fill);

        let mut b_paint = Paint::default();
        b_paint.set_color(Color::BLUE);
        b_paint.set_blend_mode(BlendMode::Plus);
        b_paint.set_anti_alias(true);
        b_paint.set_style(PaintStyle::Fill);

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

        // [Bolt Optimization] Hoist Disintegration Effect Resolution
        // Safety: We use `line_ctx` (no char_index) because `calculate_progress` strictly depends on
        // line start/end times and scalar delay, making it invariant per line.
        // Per-character variation (staggering) must use Expressions or different Effect types.
        let mut active_disintegrate_effects: Vec<(&String, &crate::effects::ResolvedEffect, f64)> = Vec::with_capacity(effects.disintegrate_effects.len());
        for (name, resolved_effect) in &effects.disintegrate_effects {
             let effect = &resolved_effect.effect;
             if EffectEngine::should_trigger(effect, &line_ctx) {
                 let progress = EffectEngine::calculate_progress(self.time, effect, &line_ctx);
                 if (0.0..=1.0).contains(&progress) {
                     active_disintegrate_effects.push((name, resolved_effect, progress));
                 }
             }
        }

        // [Bolt Optimization] Hoist Particle Effect Resolution
        let mut active_particle_effects: Vec<(&String, &crate::effects::ResolvedEffect, f64)> = Vec::with_capacity(effects.particle_effects.len());
        for (name, resolved_effect) in &effects.particle_effects {
             let effect = &resolved_effect.effect;
             if EffectEngine::should_trigger(effect, &line_ctx) {
                 let progress = EffectEngine::calculate_progress(self.time, effect, &line_ctx);
                 if (0.0..=1.0).contains(&progress) {
                     active_particle_effects.push((name, resolved_effect, progress));
                 }
             }
        }

        // [Bolt Optimization] Hoist Stroke Reveal Resolution
        let mut active_stroke_reveal_progress: Option<f64> = None;
        for effect in &effects.stroke_reveal_effects {
             if EffectEngine::should_trigger(effect, &line_ctx) {
                 let p = EffectEngine::calculate_progress(self.time, effect, &line_ctx);
                 active_stroke_reveal_progress = Some(p.clamp(0.0, 1.0));
                 break; // Only one stroke reveal supported
             }
        }

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

             // [Bolt Optimization] Use pre-resolved font from layout
             let size = glyph.font_size;
             
             if let Some(typeface) = &glyph.typeface {
                 // Get path
                 // [Bolt Optimization] Use cached path by ID instead of resolving by char/Font object
                 if let Some(path) = self.text_renderer.get_path_cached(typeface, size, glyph.glyph_id) {
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
                     // Optimization: Eliminates 1 allocation and 1 deep copy (96 bytes) per character per frame
                     // by using reference to char transform or falling back to pre-calculated line transform.
                     let mut final_transform = if let Some(c) = char_data {
                         if let Some(t) = &c.transform {
                             RenderTransform::new(&line_transform, t)
                         } else {
                             line_render_transform
                         }
                     } else {
                         line_render_transform // RenderTransform is Copy
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
                     // [Bolt Optimization] Use pre-calculated progress
                     let mut disintegration_progress = 0.0;
                     if let Some((_name, _resolved, progress)) = active_disintegrate_effects.first() {
                         disintegration_progress = progress.clamp(0.0, 1.0);
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
                     // [Bolt Optimization] Use pre-calculated progress
                     let stroke_reveal_progress = active_stroke_reveal_progress;

                     // Apply Transforms
                     self.canvas.translate((draw_x + final_transform.x, draw_y + final_transform.y));
                     
                     let path_center_x = bounds.center_x();
                     let path_center_y = bounds.center_y();
                     
                     self.canvas.translate((path_center_x, path_center_y));
                     self.canvas.rotate(final_transform.rotation, None);
                     self.canvas.scale((final_transform.scale * final_transform.scale_x, final_transform.scale * final_transform.scale_y));
                     self.canvas.translate((-path_center_x, -path_center_y));
                     
                     // Modify path if StrokeReveal is active
                     // [Bolt Optimization] COW path: avoid clone unless modified
                     let mut modified_path_storage: Option<skia_safe::Path> = None;

                     let path_to_draw: &skia_safe::Path = if let Some(progress) = stroke_reveal_progress {
                         let mut measure = skia_safe::PathMeasure::new(path, false, None);
                         let length = measure.length();
                         if let Some(partial_path) = measure.segment(0.0, length * progress as f32, true) {
                             modified_path_storage = Some(partial_path);
                             modified_path_storage.as_ref().unwrap()
                         } else {
                             path
                         }
                     } else {
                         path
                     };
                     let path = path_to_draw; // Shadow original path with the one to draw

                     
                     // --- 1. SHADOW ---
                     let (active_shadow, active_shadow_color) = if let Some(c_shadow) = char_data.and_then(|c| c.shadow.as_ref()) {
                         // [Bolt Optimization] Use pre-parsed color from glyph info
                         (Some(c_shadow), glyph.override_shadow_color)
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
                         // [Bolt Optimization] Use pre-parsed color from glyph info
                         (Some(c_stroke), glyph.override_stroke_color)
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

                             // Update shared properties for all channels
                             // [Bolt Optimization] Update hoisted paints instead of cloning.
                             // Note: We strictly use Fill style here as this block is for the main text fill.
                             // Strokes are handled in a separate block below.

                             // Red Channel Update
                             r_paint.set_alpha_f(final_opacity);
                             if final_transform.blur > 0.0 {
                                 if let Some((_, ref filter)) = cached_blur_filter {
                                     r_paint.set_mask_filter(Some(filter.clone()));
                                 }
                             } else {
                                 r_paint.set_mask_filter(None);
                             }

                             // Green Channel Update
                             g_paint.set_alpha_f(final_opacity);
                             if final_transform.blur > 0.0 {
                                 if let Some((_, ref filter)) = cached_blur_filter {
                                     g_paint.set_mask_filter(Some(filter.clone()));
                                 }
                             } else {
                                 g_paint.set_mask_filter(None);
                             }

                             // Blue Channel Update
                             b_paint.set_alpha_f(final_opacity);
                             if final_transform.blur > 0.0 {
                                 if let Some((_, ref filter)) = cached_blur_filter {
                                     b_paint.set_mask_filter(Some(filter.clone()));
                                 }
                             } else {
                                 b_paint.set_mask_filter(None);
                             }

                             self.canvas.save();
                             self.canvas.translate((-offset, -offset));
                             self.canvas.draw_path(path, &r_paint);
                             self.canvas.restore();

                             self.canvas.save();
                             self.canvas.translate((offset, -offset));
                             self.canvas.draw_path(path, &g_paint);
                             self.canvas.restore();

                             self.canvas.save();
                             self.canvas.translate((offset, offset));
                             self.canvas.draw_path(path, &b_paint);
                             self.canvas.restore();

                         } else {
                             // Normal Draw
                             self.canvas.draw_path(path, &paint);
                         }
                     }
                     
                     self.canvas.restore(); // Restore transform for next glyph/effects

                     // --- DISINTEGRATION EFFECT ---
                     // [Bolt Optimization] Iterate active effects only
                     for (name, resolved_effect, _progress) in &active_disintegrate_effects {
                         let effect = &resolved_effect.effect;
                         // progress is already checked to be in range

                         let key = hash_emitter_key(line_idx, glyph.char_index, name);
                         self.active_keys.insert(key);

                         // We need to capture the glyph as an image for the emitter
                         // Create small surface
                         // Bounds might be slightly larger due to stroke/shadow, but let's stick to path bounds for particles
                         let capture_w = w.ceil() as i32 + 20; // Padding
                         let capture_h = h.ceil() as i32 + 20;
                         if capture_w <= 0 || capture_h <= 0 { continue; }
                         
                         // Create offscreen surface for disintegration effect.
                         // TODO: Optimization - Avoid creating surface if emitter already exists.
                         // Current implementation relies on `ensure_disintegration_emitter` to handle existence checks.
                         
                         if self.particle_system.has_emitter(key) {
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
                     // [Bolt Optimization] Iterate active effects only
                     for (name, resolved_effect, progress) in &active_particle_effects {
                         let effect = &resolved_effect.effect;
                         let progress = *progress; // Copy f64

                         let key = hash_emitter_key(line_idx, glyph.char_index, name);
                         self.active_keys.insert(key);
                         
                         let bounds_rect = CharBounds {
                             x: draw_x + final_transform.x + bounds.left,
                             y: draw_y + final_transform.y + bounds.top,
                             width: w * final_transform.scale,
                             height: h * final_transform.scale,
                         };

                         // [Bolt Optimization] Skip expensive config calculation if emitter exists.
                         // But if we have overrides, we MUST recalculate config to apply dynamic expressions.
                         if self.particle_system.has_emitter(key) && effect.particle_override.is_none() {
                             self.particle_system.update_emitter_bounds(key, bounds_rect);
                             continue;
                         }

                         // Evaluation Context for Particles (needed for new emitters OR dynamic updates)
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

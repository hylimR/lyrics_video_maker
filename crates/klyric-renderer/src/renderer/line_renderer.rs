use anyhow::Result;
use skia_safe::{Canvas, Color, Paint, BlendMode, PaintStyle, MaskFilter, BlurStyle, surfaces};
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use crate::model::{KLyricDocumentV2, Line, PositionValue, EffectType, Transform, Easing, RenderTransform, Style};
use crate::layout::{LayoutEngine, GlyphInfo};
use crate::text::TextRenderer;
use crate::effects::{EffectEngine, TriggerContext, CompiledRenderOp};
use crate::presets::CharBounds;
use crate::expressions::{EvaluationContext, FastEvaluationContext};

use super::particle_system::ParticleRenderSystem;
use super::utils::parse_color;
use super::CategorizedLineEffects;
use super::ResolvedStyleColors;

pub struct RenderPaints {
    pub main_paint: Paint,
    pub shadow_paint: Paint,
    pub stroke_paint: Paint,
    pub r_paint: Paint,
    pub g_paint: Paint,
    pub b_paint: Paint,
    pub cached_blur_filter: Option<(f32, MaskFilter)>,
    pub current_paint_blur: f32,
    pub current_shadow_blur: f32,
    pub current_stroke_blur: f32,
    pub current_r_blur: f32,
    pub current_g_blur: f32,
    pub current_b_blur: f32,
}

impl RenderPaints {
    pub fn new() -> Self {
        let mut main_paint = Paint::default();
        main_paint.set_anti_alias(true);

        let mut shadow_paint = Paint::default();
        shadow_paint.set_anti_alias(true);

        let mut stroke_paint = Paint::default();
        stroke_paint.set_anti_alias(true);
        stroke_paint.set_style(PaintStyle::Stroke);

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

        Self {
            main_paint,
            shadow_paint,
            stroke_paint,
            r_paint,
            g_paint,
            b_paint,
            cached_blur_filter: None,
            current_paint_blur: 0.0,
            current_shadow_blur: 0.0,
            current_stroke_blur: 0.0,
            current_r_blur: 0.0,
            current_g_blur: 0.0,
            current_b_blur: 0.0,
        }
    }
}

pub struct LineRenderScratch {
    pub active_transform_indices: Vec<(usize, f64)>,
    pub compiled_ops: Vec<CompiledRenderOp>,
    pub active_disintegrate_indices: Vec<(usize, f64, u64)>,
    pub active_particle_indices: Vec<(usize, f64, u64)>,
    pub local_layer_indices: Vec<usize>,
    /// [Bolt Optimization] Cache for PathMeasure to avoid re-scanning paths every frame.
    /// Key: (typeface_id, font_size_bits, glyph_id)
    /// Value: (Path, PathMeasure). We must store Path because PathMeasure refers to it.
    pub path_measure_cache: HashMap<(u32, u64, u16), (skia_safe::Path, skia_safe::PathMeasure)>,
}

impl LineRenderScratch {
    pub fn new() -> Self {
        Self {
            active_transform_indices: Vec::with_capacity(16),
            compiled_ops: Vec::with_capacity(32),
            active_disintegrate_indices: Vec::with_capacity(4),
            active_particle_indices: Vec::with_capacity(8),
            local_layer_indices: Vec::with_capacity(4),
            path_measure_cache: HashMap::new(),
        }
    }
}

// [Bolt Optimization] Fast Hashing for Cache Keys
// Replaces DefaultHasher (SipHasher) with a simple bitwise mix (FxHash-like).
// This eliminates allocating and cloning Hasher state in the hot loop.
const HASH_SEED: u64 = 0x517cc1b727220a95;

#[inline(always)]
fn fast_hash_combine(acc: u64, val: u64) -> u64 {
    (acc.rotate_left(5) ^ val).wrapping_mul(HASH_SEED)
}

#[inline(always)]
fn fast_hash_prefix(line_idx: usize, name_hash: u64) -> u64 {
    let mut h = HASH_SEED;
    h = fast_hash_combine(h, line_idx as u64);
    h = fast_hash_combine(h, name_hash);
    h
}

#[inline(always)]
fn fast_hash_finish(prefix: u64, char_idx: usize) -> u64 {
    fast_hash_combine(prefix, char_idx as u64)
}

impl Default for LineRenderScratch {
    fn default() -> Self {
        Self::new()
    }
}

pub struct LineRenderer<'a> {
    pub canvas: &'a Canvas,
    pub doc: &'a KLyricDocumentV2,
    pub time: f64,
    pub text_renderer: &'a mut TextRenderer,
    pub particle_system: &'a mut ParticleRenderSystem,
    pub width: u32,
    pub height: u32,
    pub paints: &'a mut RenderPaints,
}

impl<'a> LineRenderer<'a> {
    pub fn render_line(&mut self, line: &Line, glyphs: &[GlyphInfo], line_idx: usize, style: &Style, colors: &ResolvedStyleColors, effects: &'a CategorizedLineEffects, scratch: &mut LineRenderScratch) -> Result<()> {
        // Compute Line Position
        let (base_x, base_y) = self.compute_line_position(line);

        // Resolve font size for rasterization (Base)
        let style_family = style.font.as_ref().and_then(|f| f.family.as_deref()).unwrap_or("Noto Sans SC");
        let style_size = style.font.as_ref().and_then(|f| f.size).unwrap_or(72.0);

        // Pre-resolve colors to avoid parsing per-glyph
        // [Bolt Optimization] Use cached colors passed from Renderer
        let inactive_color = colors.inactive;
        let active_color = colors.active;
        let complete_color = colors.complete;

        // Hoist Line Transform
        let line_transform = line.transform.clone().unwrap_or_default();

        // [Bolt Optimization] Paint objects are now reused from `self.paints` (see RenderPaints)

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

        // [Bolt Optimization] Use scratch buffers for active effects to avoid per-frame allocation
        scratch.active_transform_indices.clear();
        for (i, resolved) in effects.transform_effects.iter().enumerate() {
            if EffectEngine::should_trigger(&resolved.effect, &line_ctx) {
                let p = EffectEngine::calculate_progress(self.time, &resolved.effect, &line_ctx);
                if (0.0..=1.0).contains(&p) {
                    scratch.active_transform_indices.push((i, EffectEngine::ease(p, &resolved.effect.easing)));
                }
            }
        }

        // Compile ops into scratch buffer
        EffectEngine::compile_active_effects(&effects.transform_effects, &scratch.active_transform_indices, &line_ctx, &mut scratch.compiled_ops);

        // [Bolt Optimization] Hoist Disintegration Effect Resolution
        // Safety: We use `line_ctx` (no char_index) because `calculate_progress` strictly depends on
        // line start/end times and scalar delay, making it invariant per line.
        // Per-character variation (staggering) must use Expressions or different Effect types.
        scratch.active_disintegrate_indices.clear();
        for (i, (_name, resolved_effect)) in effects.disintegrate_effects.iter().enumerate() {
             let effect = &resolved_effect.effect;
             if EffectEngine::should_trigger(effect, &line_ctx) {
                 let progress = EffectEngine::calculate_progress(self.time, effect, &line_ctx);
                 if (0.0..=1.0).contains(&progress) {
                     // [Bolt Optimization] Pre-calculate partial hash prefix
                     let prefix = fast_hash_prefix(line_idx, resolved_effect.name_hash);
                     scratch.active_disintegrate_indices.push((i, progress, prefix));
                 }
             }
        }

        // [Bolt Optimization] Hoist Particle Effect Resolution
        scratch.active_particle_indices.clear();
        for (i, (_name, resolved_effect)) in effects.particle_effects.iter().enumerate() {
             let effect = &resolved_effect.effect;
             if EffectEngine::should_trigger(effect, &line_ctx) {
                 let progress = EffectEngine::calculate_progress(self.time, effect, &line_ctx);
                 if (0.0..=1.0).contains(&progress) {
                     // [Bolt Optimization] Pre-calculate partial hash prefix
                     let prefix = fast_hash_prefix(line_idx, resolved_effect.name_hash);
                     scratch.active_particle_indices.push((i, progress, prefix));
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
        let style_shadow_color = colors.shadow;

        let line_shadow_color = line.shadow.as_ref()
            .and_then(|s| s.color.as_deref())
            .and_then(parse_color);

        let style_stroke_color = colors.stroke;

        let line_stroke_color = line.stroke.as_ref()
            .and_then(|s| s.color.as_deref())
            .and_then(parse_color);

        // [Bolt Optimization] Pre-calculate fallback Shadow/Stroke (Line > Style)
        // This avoids checking line vs style for every character.
        let (fallback_shadow, fallback_shadow_color) = if let Some(l_shadow) = line.shadow.as_ref() {
            (Some(l_shadow), line_shadow_color)
        } else {
            (style.shadow.as_ref(), style_shadow_color)
        };

        let (fallback_stroke, fallback_stroke_color) = if let Some(l_stroke) = line.stroke.as_ref() {
            (Some(l_stroke), line_stroke_color)
        } else {
            (style.stroke.as_ref(), style_stroke_color)
        };

        // [Bolt Optimization] Hoist Global Layer Transforms
        // We separate layers into "Global" (same for every char) and "Local" (depends on index/char).
        // Global layers are computed once per line. Local layers are computed per char.
        scratch.local_layer_indices.clear();
        let global_layer_transform = if let Some(layers) = style.layers.as_ref() {
            let global_ctx = TriggerContext {
                start_time: line.start,
                end_time: line.end,
                current_time: self.time,
                active: true,
                char_index: Some(0), // Dummy index to satisfy Scope(Char)
                char_count: Some(glyphs.len()),
            };
            EffectEngine::compute_global_layer_transform(
                self.time,
                layers,
                &global_ctx,
                &mut scratch.local_layer_indices,
            )
        } else {
            RenderTransform::default()
        };

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
                 // [Bolt Optimization] Use cached path from GlyphInfo (populated during layout)
                 // This avoids FFI calls and HashMap lookups per frame.
                 if let Some(path) = glyph.path.as_ref() {
                     // Dimensions for effects
                     // [Bolt Optimization] Use cached bounds from GlyphInfo
                     let bounds = glyph.bounds.unwrap_or_else(|| path.bounds());
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
                     final_transform = EffectEngine::apply_compiled_ops(final_transform, &scratch.compiled_ops, &mut fast_ctx);
                     
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
                     // [Bolt Optimization] Apply hoisted global transform + local layers
                     final_transform.combine(&global_layer_transform);

                     if let Some(layers) = style.layers.as_ref() {
                         if !scratch.local_layer_indices.is_empty() {
                             final_transform = EffectEngine::apply_specific_layers_to_render(
                                 self.time,
                                 final_transform,
                                 layers,
                                 &scratch.local_layer_indices,
                                 &ctx,
                             );
                         }
                     }

                     // Disintegrate Effect Progress
                     // [Bolt Optimization] Use pre-calculated progress
                     let mut disintegration_progress = 0.0;
                     if let Some((_idx, progress, _)) = scratch.active_disintegrate_indices.first() {
                         disintegration_progress = progress.clamp(0.0, 1.0);
                     }

                     // Setup Paint
                     // [Bolt Optimization] Manual reuse without reset()
                     self.paints.main_paint.set_color(text_color);
                     
                     let final_opacity = final_transform.opacity * (1.0 - disintegration_progress as f32);
                     self.paints.main_paint.set_alpha_f(final_opacity);

                     // Apply Blur with caching
                     let blur = final_transform.blur;
                     if blur > 0.0 {
                         let use_cached = if let Some((last_sigma, _)) = self.paints.cached_blur_filter.as_ref() {
                             (last_sigma - blur).abs() < 0.001
                         } else {
                             false
                         };

                         if !use_cached {
                             // Create new filter
                             if let Some(filter) = MaskFilter::blur(BlurStyle::Normal, blur, false) {
                                 self.paints.cached_blur_filter = Some((blur, filter));
                             } else {
                                 self.paints.cached_blur_filter = None;
                             }
                         }
                     }

                     // [Bolt Optimization] Only update paint mask filter if blur changed
                     apply_paint_blur(&mut self.paints.main_paint, &mut self.paints.current_paint_blur, blur, &self.paints.cached_blur_filter);

                     // --- DRAWING ---
                     // Calculate position context
                     let draw_x = char_absolute_x;
                     let draw_y = char_absolute_y;
                     
                     // Check for StrokeReveal
                     // [Bolt Optimization] Use pre-calculated progress
                     let stroke_reveal_progress = active_stroke_reveal_progress;

                     // [Bolt Optimization] Fast path for translation-only transforms
                     // Skip expensive save/restore if we only have translation (no rotate/scale)
                     // WARNING: This optimization assumes that NO other canvas state (clip, matrix, etc.)
                     // is permanently modified within this block. If future changes introduce clipping
                     // or complex matrix ops that need restoration, this optimization must be disabled.
                     let is_simple_transform = final_transform.is_simple_translation();
                     let tx = draw_x + final_transform.x;
                     let ty = draw_y + final_transform.y;

                     if is_simple_transform {
                         self.canvas.translate((tx, ty));
                     } else {
                         self.canvas.save();
                         self.canvas.translate((tx, ty));

                         let path_center_x = bounds.center_x();
                         let path_center_y = bounds.center_y();

                         self.canvas.translate((path_center_x, path_center_y));
                         self.canvas.rotate(final_transform.rotation, None);
                         self.canvas.scale((final_transform.scale * final_transform.scale_x, final_transform.scale * final_transform.scale_y));
                         self.canvas.translate((-path_center_x, -path_center_y));
                     }
                     
                     // Modify path if StrokeReveal is active
                     // [Bolt Optimization] COW path: avoid clone unless modified
                     let mut modified_path_storage: Option<skia_safe::Path> = None;

                     let path_to_draw: &skia_safe::Path = if let Some(progress) = stroke_reveal_progress {
                         // [Bolt Optimization] Short-circuit if effectively complete to avoid PathMeasure
                         if progress >= 0.999 {
                             path
                         } else if progress <= 0.001 {
                             // [Bolt Optimization] Avoid PathMeasure if progress is effectively zero (e.g. delay).
                             // Returns an empty path to skip drawing.
                             modified_path_storage = Some(skia_safe::Path::new());
                             modified_path_storage.as_ref().unwrap()
                         } else {
                             // [Bolt Optimization] Use cached PathMeasure
                             let tf_id: u32 = if let Some(tf) = &glyph.typeface {
                                 tf.unique_id().into()
                             } else {
                                 0
                             };
                             let key = (tf_id, glyph.font_size.to_bits(), glyph.glyph_id);

                             // Safety: PathMeasure refers to SkPath object. We must keep the SkPath object alive.
                             // We store (Path, PathMeasure) in the cache.
                             let (_, measure) = scratch.path_measure_cache.entry(key).or_insert_with(|| {
                                 let p = path.clone();
                                 let m = skia_safe::PathMeasure::new(&p, false, None);
                                 (p, m)
                             });

                             let length = measure.length();
                             if let Some(partial_path) = measure.segment(0.0, length * progress as f32, true) {
                                 modified_path_storage = Some(partial_path);
                                 modified_path_storage.as_ref().unwrap()
                             } else {
                                 path
                             }
                         }
                     } else {
                         path
                     };
                     let path = path_to_draw; // Shadow original path with the one to draw

                     
                     // --- 1. SHADOW ---
                     let (active_shadow, active_shadow_color) = if let Some(c_shadow) = char_data.and_then(|c| c.shadow.as_ref()) {
                         // [Bolt Optimization] Use pre-parsed color from glyph info
                         (Some(c_shadow), glyph.override_shadow_color)
                     } else {
                         (fallback_shadow, fallback_shadow_color)
                     };

                     if let (Some(shadow), Some(shadow_color)) = (active_shadow, active_shadow_color) {
                         // [Bolt Optimization] Reuse shadow paint
                         self.paints.shadow_paint.set_color(shadow_color);
                         self.paints.shadow_paint.set_alpha_f(final_opacity);

                         // [Bolt Optimization] Apply blur to shadow with state tracking
                         apply_paint_blur(&mut self.paints.shadow_paint, &mut self.paints.current_shadow_blur, final_transform.blur, &self.paints.cached_blur_filter);

                         self.canvas.save();
                         self.canvas.translate((shadow.x_or_default(), shadow.y_or_default()));
                         self.canvas.draw_path(path, &self.paints.shadow_paint);
                         self.canvas.restore();
                     }

                     // --- 2. STROKE ---
                     let (active_stroke, active_stroke_color) = if let Some(c_stroke) = char_data.and_then(|c| c.stroke.as_ref()) {
                         // [Bolt Optimization] Use pre-parsed color from glyph info
                         (Some(c_stroke), glyph.override_stroke_color)
                     } else {
                         (fallback_stroke, fallback_stroke_color)
                     };

                     if let (Some(stroke), Some(stroke_color)) = (active_stroke, active_stroke_color) {
                         if stroke.width_or_default() > 0.0 {
                             // [Bolt Optimization] Reuse stroke paint
                             self.paints.stroke_paint.set_stroke_width(stroke.width_or_default());
                             self.paints.stroke_paint.set_color(stroke_color);
                             self.paints.stroke_paint.set_alpha_f(final_opacity);

                             // [Bolt Optimization] Apply blur to stroke with state tracking
                             apply_paint_blur(&mut self.paints.stroke_paint, &mut self.paints.current_stroke_blur, final_transform.blur, &self.paints.cached_blur_filter);

                             self.canvas.draw_path(path, &self.paints.stroke_paint);
                         }
                     }

                     // --- 3. MAIN TEXT (With Glitch Logic) ---
                     if self.paints.main_paint.alpha_f() > 0.001 {
                         if final_transform.glitch_offset.abs() > 0.01 {
                             // Glitch Effect: Draw channels separately
                             let offset = final_transform.glitch_offset;

                             // Update shared properties for all channels
                             // [Bolt Optimization] Update hoisted paints instead of cloning.
                             // Note: We strictly use Fill style here as this block is for the main text fill.
                             // Strokes are handled in a separate block below.

                             // Red Channel Update
                             self.paints.r_paint.set_alpha_f(final_opacity);
                             apply_paint_blur(&mut self.paints.r_paint, &mut self.paints.current_r_blur, final_transform.blur, &self.paints.cached_blur_filter);

                             // Green Channel Update
                             self.paints.g_paint.set_alpha_f(final_opacity);
                             apply_paint_blur(&mut self.paints.g_paint, &mut self.paints.current_g_blur, final_transform.blur, &self.paints.cached_blur_filter);

                             // Blue Channel Update
                             self.paints.b_paint.set_alpha_f(final_opacity);
                             apply_paint_blur(&mut self.paints.b_paint, &mut self.paints.current_b_blur, final_transform.blur, &self.paints.cached_blur_filter);

                             self.canvas.save();
                             self.canvas.translate((-offset, -offset));
                             self.canvas.draw_path(path, &self.paints.r_paint);
                             self.canvas.restore();

                             self.canvas.save();
                             self.canvas.translate((offset, -offset));
                             self.canvas.draw_path(path, &self.paints.g_paint);
                             self.canvas.restore();

                             self.canvas.save();
                             self.canvas.translate((offset, offset));
                             self.canvas.draw_path(path, &self.paints.b_paint);
                             self.canvas.restore();

                         } else {
                             // Normal Draw
                             self.canvas.draw_path(path, &self.paints.main_paint);
                         }
                     }
                     
                     if is_simple_transform {
                         self.canvas.translate((-tx, -ty));
                     } else {
                         self.canvas.restore(); // Restore transform for next glyph/effects
                     }

                     // --- DISINTEGRATION EFFECT ---
                     // [Bolt Optimization] Iterate active effects only
                     for (idx, _progress, base_prefix) in &scratch.active_disintegrate_indices {
                         let (_name, resolved_effect) = &effects.disintegrate_effects[*idx];
                         let effect = &resolved_effect.effect;
                         // progress is already checked to be in range

                         // [Bolt Optimization] Finish hashing using pre-calculated prefix (Zero Alloc)
                         let key = fast_hash_finish(*base_prefix, glyph.char_index);

                         // We need to capture the glyph as an image for the emitter
                         // Create small surface
                         // Bounds might be slightly larger due to stroke/shadow, but let's stick to path bounds for particles
                         let capture_w = w.ceil() as i32 + 20; // Padding
                         let capture_h = h.ceil() as i32 + 20;
                         if capture_w <= 0 || capture_h <= 0 { continue; }
                         
                         // Create offscreen surface for disintegration effect.
                         // Optimization: Skip surface creation if emitter already exists.
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
                     for (idx, progress, base_prefix) in &scratch.active_particle_indices {
                         let (_name, resolved_effect) = &effects.particle_effects[*idx];
                         let effect = &resolved_effect.effect;
                         let progress = *progress; // Copy f64

                         // [Bolt Optimization] Finish hashing using pre-calculated prefix (Zero Alloc)
                         let key = fast_hash_finish(*base_prefix, glyph.char_index);

                         let bounds_rect = CharBounds {
                             x: draw_x + final_transform.x + bounds.left,
                             y: draw_y + final_transform.y + bounds.top,
                             width: w * final_transform.scale,
                             height: h * final_transform.scale,
                         };

                         // [Bolt Optimization] Update existing emitter (Single Lookup)
                         // We set context progress just in case overrides need it
                         fast_ctx.set_progress(progress);

                         if self.particle_system.update_existing_emitter(
                             key,
                             bounds_rect.clone(),
                             effect.particle_config.as_ref(),
                             effect.particle_override.as_ref(),
                             Some(&resolved_effect.compiled_expressions),
                             &fast_ctx
                         ) {
                             continue;
                         }

                         // New Emitter Path (Allocation Path)
                         let seed = (line_idx * 1000 + glyph.char_index * 100) as u64;
                         
                         // Clone config and apply overrides
                         let mut p_config = effect.particle_config.clone();
                         if let (Some(config), Some(overrides)) = (&mut p_config, &effect.particle_override) {
                             // Reuse context
                             fast_ctx.set_progress(progress);

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

/// Helper to apply blur mask filter to a paint object with state tracking.
/// This prevents redundant ref-counting updates when blur sigma hasn't changed.
fn apply_paint_blur(
    paint: &mut Paint,
    current_blur: &mut f32,
    target_blur: f32,
    cached_filter: &Option<(f32, MaskFilter)>
) {
    if (target_blur - *current_blur).abs() > 0.001 {
        if target_blur > 0.0 {
            if let Some((_, ref filter)) = cached_filter {
                paint.set_mask_filter(Some(filter.clone()));
                *current_blur = target_blur;
            } else {
                // Fallback if filter creation failed
                paint.set_mask_filter(None);
                *current_blur = 0.0;
            }
        } else {
            paint.set_mask_filter(None);
            *current_blur = 0.0;
        }
    }
}

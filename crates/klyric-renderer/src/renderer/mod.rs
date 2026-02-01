pub mod line_renderer;
pub mod particle_system;
pub mod utils;

use anyhow::Result;
use skia_safe::{surfaces, AlphaType, Canvas, Color, ColorType, ImageInfo, Surface};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::effects::ResolvedEffect;
use crate::expressions::ExpressionEvaluator;
use crate::layout::{GlyphInfo, LayoutEngine};
use crate::model::{AnimatedValue, Easing, Effect, EffectType, KLyricDocumentV2, Line, Style};
use crate::presets::{CharBounds, EffectPreset};
use crate::style::StyleResolver;
use crate::text::TextRenderer;

use self::line_renderer::LineRenderer;
use self::particle_system::ParticleRenderSystem;
use self::utils::parse_color;

/// Default colors for karaoke states when not specified in style
const DEFAULT_INACTIVE_COLOR: &str = "#888888";  // Dimmed gray
const DEFAULT_ACTIVE_COLOR: &str = "#FFFF00";    // Bright yellow
const DEFAULT_COMPLETE_COLOR: &str = "#FFFFFF";  // White

#[derive(Clone, Debug)]
pub struct ResolvedStyleColors {
    pub inactive: Color,
    pub active: Color,
    pub complete: Color,
    pub shadow: Option<Color>,
    pub stroke: Option<Color>,
}

fn resolve_style_colors(style: &Style) -> ResolvedStyleColors {
    let inactive_hex = style.colors.as_ref()
        .and_then(|c| c.inactive.as_ref())
        .and_then(|fs| fs.fill.as_deref())
        .unwrap_or(DEFAULT_INACTIVE_COLOR);
    let inactive = parse_color(inactive_hex).unwrap_or(Color::WHITE);

    let active_hex = style.colors.as_ref()
        .and_then(|c| c.active.as_ref())
        .and_then(|fs| fs.fill.as_deref())
        .unwrap_or(DEFAULT_ACTIVE_COLOR);
    let active = parse_color(active_hex).unwrap_or(Color::WHITE);

    let complete_hex = style.colors.as_ref()
        .and_then(|c| c.complete.as_ref())
        .and_then(|fs| fs.fill.as_deref())
        .unwrap_or(DEFAULT_COMPLETE_COLOR);
    let complete = parse_color(complete_hex).unwrap_or(Color::WHITE);

    let shadow = style.shadow.as_ref()
        .and_then(|s| s.color.as_deref())
        .and_then(parse_color);

    let stroke = style.stroke.as_ref()
        .and_then(|s| s.color.as_deref())
        .and_then(parse_color);

    ResolvedStyleColors {
        inactive,
        active,
        complete,
        shadow,
        stroke,
    }
}

#[derive(Clone)]
pub struct CategorizedLineEffects {
    pub transform_effects: Vec<ResolvedEffect>,
    pub particle_effects: Vec<(String, ResolvedEffect)>,
    pub disintegrate_effects: Vec<(String, ResolvedEffect)>,
    pub stroke_reveal_effects: Vec<Effect>,
}

pub struct Renderer {
    width: u32,
    height: u32,
    text_renderer: TextRenderer,
    particle_system: ParticleRenderSystem,
    /// Last rendered time for delta calculation
    last_time: f64,
    /// Cached surface for rendering
    surface: Option<Surface>,
    /// Cache for resolved styles to avoid re-resolution every frame
    style_cache: HashMap<String, Style>,
    /// Pointer to the last document used (to invalidate cache)
    last_doc_ptr: usize,
    /// Cache for text layouts: content_hash -> Vec<GlyphInfo>
    layout_cache: HashMap<u64, Vec<GlyphInfo>>,
    /// Cache for layout hashes per line: (line_ptr, style_ptr) -> layout_hash
    /// Optimization: Avoids O(N) hashing per frame.
    line_hash_cache: HashMap<(usize, usize), u64>,
    /// Cache for pre-categorized effects per line: line_ptr -> CategorizedLineEffects
    line_effect_cache: HashMap<usize, CategorizedLineEffects>,
    /// Cache for resolved style colors: style_name -> ResolvedStyleColors
    style_color_cache: HashMap<String, ResolvedStyleColors>,
    /// Cached paint objects to avoid allocation per frame
    render_paints: line_renderer::RenderPaints,
}

impl Renderer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            text_renderer: TextRenderer::new(),
            particle_system: ParticleRenderSystem::new(),
            last_time: 0.0,
            surface: None,
            style_cache: HashMap::new(),
            last_doc_ptr: 0,
            layout_cache: HashMap::new(),
            line_hash_cache: HashMap::new(),
            line_effect_cache: HashMap::new(),
            style_color_cache: HashMap::new(),
            render_paints: line_renderer::RenderPaints::new(),
        }
    }

    pub fn text_renderer_mut(&mut self) -> &mut TextRenderer {
        &mut self.text_renderer
    }

    /// Render directly to an existing Canvas
    pub fn render_to_canvas(
        &mut self,
        canvas: &Canvas,
        doc: &KLyricDocumentV2,
        time: f64,
    ) -> Result<()> {
        // Clear transient font cache to prevent memory leak from animated sizes
        self.text_renderer.clear_font_cache();

        // Calculate delta time
        let dt = if self.last_time > 0.0 {
            (time - self.last_time).max(0.0)
        } else {
            0.0
        };
        self.last_time = time;

        // Check if document changed (pointer check)
        let current_doc_ptr = doc as *const _ as usize;
        if self.last_doc_ptr != current_doc_ptr {
            self.style_cache.clear();
            // We also clear layout cache on doc change to avoid unbounded growth
            // even though hashing protects against stale content.
            self.layout_cache.clear();
            self.line_hash_cache.clear();
            self.line_effect_cache.clear();
            self.style_color_cache.clear();
            self.last_doc_ptr = current_doc_ptr;
        }

        // 1. Draw Background
        self.draw_background(canvas, doc);

        // Track which emitters are active this frame
        self.particle_system.reset_active_flags();

        // 2. Find Active Lines and render
        if let Some(line) = doc.get_active_line(time) {
            // We need the line index to create unique keys
            if let Some(line_idx) = doc.lines.iter().position(|l| std::ptr::eq(l, line)) {
                // Resolve style (cached)
                let style_name = line.style.as_deref().unwrap_or("base");
                let style = if let Some(s) = self.style_cache.get(style_name) {
                    s
                } else {
                    let s = StyleResolver::new(doc).resolve(style_name);
                    self.style_cache.insert(style_name.to_string(), s);
                    self.style_cache.get(style_name).unwrap()
                };

                // Layout (Cached via Content Hash)
                // Optimization: Cache the hash calculation to avoid O(N) work every frame.
                // Key by (line_ptr, style_ptr) to ensure correctness if line changes style association.
                let line_ptr = line as *const _ as usize;
                let style_ptr = style as *const _ as usize;
                let hash_key = (line_ptr, style_ptr);

                let layout_hash = if let Some(&hash) = self.line_hash_cache.get(&hash_key) {
                    hash
                } else {
                    let hash = compute_layout_hash(line, style);
                    self.line_hash_cache.insert(hash_key, hash);
                    hash
                };

                if !self.layout_cache.contains_key(&layout_hash) {
                    let g = LayoutEngine::layout_line(line, style, &mut self.text_renderer);
                    self.layout_cache.insert(layout_hash, g);
                }
                let glyphs = self.layout_cache.get(&layout_hash).unwrap();

                // Effects (Cached via Line Ptr)
                if !self.line_effect_cache.contains_key(&line_ptr) {
                    let effects = Self::resolve_line_effects(doc, line, style);
                    self.line_effect_cache.insert(line_ptr, effects);
                }
                let effects = self.line_effect_cache.get(&line_ptr).unwrap();

                // [Bolt Optimization] Resolve colors (cached)
                if !self.style_color_cache.contains_key(style_name) {
                    let colors = resolve_style_colors(style);
                    self.style_color_cache
                        .insert(style_name.to_string(), colors);
                }
                let style_colors = self.style_color_cache.get(style_name).unwrap();

                let mut line_renderer = LineRenderer {
                    canvas,
                    doc,
                    time,
                    text_renderer: &mut self.text_renderer,
                    particle_system: &mut self.particle_system,
                    width: self.width,
                    height: self.height,
                    paints: &mut self.render_paints,
                };

                line_renderer.render_line(line, glyphs, line_idx, style, style_colors, effects)?;
            }
        }

        // 3. Update and render particles
        self.particle_system
            .update(dt as f32);
        self.particle_system.render(canvas);

        Ok(())
    }

    /// Resolve and categorize effects for a line.
    /// This resolves presets and creates owned Effect copies for caching.
    fn resolve_line_effects(
        doc: &KLyricDocumentV2,
        line: &Line,
        style: &Style,
    ) -> CategorizedLineEffects {
        let empty_vec = Vec::new();
        let style_effects = style.effects.as_ref().unwrap_or(&empty_vec);
        let line_effects = &line.effects;
        let total_effects = style_effects.len() + line_effects.len();

        let mut transform_effects: Vec<ResolvedEffect> = Vec::with_capacity(total_effects);
        let mut particle_effects: Vec<(String, ResolvedEffect)> =
            Vec::with_capacity(total_effects / 2);
        let mut disintegrate_effects: Vec<(String, ResolvedEffect)> = Vec::with_capacity(1);
        let mut stroke_reveal_effects: Vec<Effect> = Vec::with_capacity(1);

        // Collect all effect names: Style effects first (base), then Line effects (override/stack)
        let all_effects_names = style_effects.iter().chain(line_effects.iter());

        for effect_name in all_effects_names {
            // Resolve effect (handling presets and references)
            let effect_resolved: Option<Effect> = if let Some(effect) = doc.effects.get(effect_name)
            {
                if let Some(preset_name) = &effect.preset {
                    if let Some(mut generated) =
                        crate::presets::transitions::get_transition(preset_name)
                    {
                        if let Some(d) = effect.duration {
                            generated.duration = Some(d);
                        }
                        if effect.easing != Easing::Linear {
                            generated.easing = effect.easing.clone();
                        }
                        Some(generated)
                    } else {
                        Some(effect.clone())
                    }
                } else {
                    Some(effect.clone())
                }
            } else if let Some(preset) = crate::presets::transitions::get_transition(effect_name) {
                Some(preset)
            } else {
                None
            };

            if let Some(effect) = effect_resolved {
                match effect.effect_type {
                    EffectType::Particle => {
                        let resolved = Self::resolve_expressions(effect, effect_name);
                        particle_effects.push((effect_name.clone(), resolved));
                    }
                    EffectType::Disintegrate => {
                        let resolved = Self::resolve_expressions(effect, effect_name);
                        disintegrate_effects.push((effect_name.clone(), resolved));
                    }
                    EffectType::StrokeReveal => {
                        stroke_reveal_effects.push(effect);
                    }
                    _ => {
                        let resolved = Self::resolve_expressions(effect, effect_name);
                        transform_effects.push(resolved);
                    }
                }
            }
        }

        CategorizedLineEffects {
            transform_effects,
            particle_effects,
            disintegrate_effects,
            stroke_reveal_effects,
        }
    }

    /// Helper to compile expressions in an Effect
    fn resolve_expressions(effect: Effect, name: &str) -> ResolvedEffect {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut map = HashMap::new();
        // Check standard properties
        for value in effect.properties.values() {
            if let AnimatedValue::Expression(expr) = value {
                if let Ok(node) = ExpressionEvaluator::compile(expr) {
                    map.insert(expr.clone(), Arc::new(node));
                }
            }
        }
        // Check particle overrides
        if let Some(overrides) = &effect.particle_override {
            for expr in overrides.values() {
                if let Ok(node) = ExpressionEvaluator::compile(expr) {
                    map.insert(expr.clone(), Arc::new(node));
                }
            }
        }

        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        let name_hash = hasher.finish();

        ResolvedEffect {
            effect,
            compiled_expressions: map,
            name_hash,
        }
    }

    /// Render a frame and return raw RGBA pixels
    pub fn render_frame(&mut self, doc: &KLyricDocumentV2, time: f64) -> Result<Vec<u8>> {
        // Check if we need to recreate the surface
        let needs_recreate = if let Some(s) = &self.surface {
            s.width() != self.width as i32 || s.height() != self.height as i32
        } else {
            true
        };

        if needs_recreate {
            log::info!(
                " creating/recreating renderer surface: {}x{}",
                self.width,
                self.height
            );
            let surface = surfaces::raster_n32_premul((self.width as i32, self.height as i32))
                .ok_or_else(|| anyhow::anyhow!("Failed to create skia surface"))?;
            self.surface = Some(surface);
        }

        // Take surface ownerhip temporarily to avoid double borrow
        let mut surface = self.surface.take().expect("Surface should exist");

        // Render
        let render_result = self.render_to_canvas(surface.canvas(), doc, time);

        if let Err(e) = render_result {
            // Put surface back before returning error
            self.surface = Some(surface);
            return Err(e);
        }

        // Return pixels (RGBA or BGRA? Surface N32 usually implies native. We might need specific ColorType::RGBA8888)
        // Ensure we get RGBA for ffmpeg
        let mut pixels = vec![0u8; (self.width * self.height * 4) as usize];
        let info = ImageInfo::new(
            (self.width as i32, self.height as i32),
            ColorType::RGBA8888,
            AlphaType::Premul,
            None,
        );

        let read_success =
            surface.read_pixels(&info, &mut pixels, (self.width * 4) as usize, (0, 0));

        // Put surface back
        self.surface = Some(surface);

        if read_success {
            Ok(pixels)
        } else {
            Err(anyhow::anyhow!("Failed to read pixels from surface"))
        }
    }

    /// Add a manual particle effect (e.g. for testing)
    pub fn add_particle_effect(&mut self, preset: EffectPreset, bounds: CharBounds, seed: u64) {
        self.particle_system.add_manual_effect(preset, bounds, seed);
    }

    /// Trigger a burst effect at given position
    pub fn burst_effect(
        &mut self,
        preset: EffectPreset,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        seed: u64,
    ) {
        let bounds = CharBounds {
            x,
            y,
            width,
            height,
        };
        self.particle_system.burst_effect(preset, bounds, seed);
    }

    /// Clear all particle emitters
    pub fn clear_particles(&mut self) {
        self.particle_system.clear();
    }

    fn draw_background(&self, canvas: &Canvas, doc: &KLyricDocumentV2) {
        if let Some(theme) = &doc.theme {
            if let Some(bg) = &theme.background {
                if let Some(hex) = &bg.color {
                    if let Some(color) = parse_color(hex) {
                        let sc = Color::from_argb(255, color.r(), color.g(), color.b());
                        canvas.clear(sc);
                        return;
                    }
                }
            }
        }
        // Default black
        canvas.clear(Color::BLACK);
    }
}

fn compute_layout_hash(line: &Line, style: &Style) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();

    // Line Layout
    if let Some(l) = &line.layout {
        l.mode.hash(&mut hasher);
        l.align.hash(&mut hasher);
        l.justify.hash(&mut hasher);
        l.gap.to_bits().hash(&mut hasher);
        l.wrap.hash(&mut hasher);
        if let Some(mw) = l.max_width {
            mw.to_bits().hash(&mut hasher);
        }
    }

    // Line Font Override
    if let Some(f) = &line.font {
        f.family.hash(&mut hasher);
        if let Some(s) = f.size {
            s.to_bits().hash(&mut hasher);
        }
    }

    // Style Font (Base)
    if let Some(f) = &style.font {
        f.family.hash(&mut hasher);
        if let Some(s) = f.size {
            s.to_bits().hash(&mut hasher);
        }
    }

    // Chars
    for c in &line.chars {
        c.char.hash(&mut hasher);
        if let Some(f) = &c.font {
            f.family.hash(&mut hasher);
            if let Some(s) = f.size {
                s.to_bits().hash(&mut hasher);
            }
        }
    }

    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Background, Project, Resolution, Theme};
    use std::collections::HashMap;

    /// Create a minimal empty document for testing
    fn minimal_doc() -> KLyricDocumentV2 {
        KLyricDocumentV2 {
            schema: None,
            version: "2.0".to_string(),
            project: Project {
                title: "Test".to_string(),
                artist: None,
                album: None,
                duration: 10.0,
                resolution: Resolution {
                    width: 1920,
                    height: 1080,
                },
                fps: 30,
                audio: None,
                created: None,
                modified: None,
            },
            theme: None,
            styles: HashMap::new(),
            effects: HashMap::new(),
            lines: Vec::new(),
        }
    }

    /// Create a document with custom background color
    fn doc_with_background(color: &str) -> KLyricDocumentV2 {
        let mut doc = minimal_doc();
        doc.theme = Some(Theme {
            background: Some(Background {
                bg_type: crate::model::BackgroundType::Solid,
                color: Some(color.to_string()),
                gradient: None,
                image: None,
                video: None,
                opacity: 1.0,
            }),
            default_style: None,
        });
        doc
    }

    // --- Renderer Creation Tests ---

    #[test]
    fn test_new_dimensions() {
        let renderer = Renderer::new(800, 600);

        assert_eq!(renderer.width, 800, "Width should be stored correctly");
        assert_eq!(renderer.height, 600, "Height should be stored correctly");
    }

    // --- Render Frame Tests ---

    #[test]
    fn test_render_empty_doc() {
        let mut renderer = Renderer::new(100, 100);
        let doc = minimal_doc();

        let pixels = renderer
            .render_frame(&doc, 0.0)
            .expect("Render should succeed");

        // Correct pixel count: width * height * 4 bytes (RGBA)
        let expected_size = 100 * 100 * 4;
        assert_eq!(
            pixels.len(),
            expected_size,
            "Pixel buffer should have correct size: {} vs {}",
            pixels.len(),
            expected_size
        );
    }

    #[test]
    fn test_render_black_background() {
        let mut renderer = Renderer::new(10, 10);
        let doc = minimal_doc();

        let pixels = renderer
            .render_frame(&doc, 0.0)
            .expect("Render should succeed");

        // Check that pixels are black (R=0, G=0, B=0)
        // Due to premultiplied alpha, fully opaque black is (0, 0, 0, 255)
        let mut non_black_count = 0;
        for chunk in pixels.chunks_exact(4) {
            // Allow small tolerance for GPU precision
            if chunk[0] > 5 || chunk[1] > 5 || chunk[2] > 5 {
                non_black_count += 1;
            }
        }

        assert_eq!(
            non_black_count, 0,
            "Default background should be black, found {} non-black pixels",
            non_black_count
        );
    }

    #[test]
    fn test_render_custom_background() {
        let mut renderer = Renderer::new(10, 10);
        let doc = doc_with_background("#FF0000"); // Red

        let pixels = renderer
            .render_frame(&doc, 0.0)
            .expect("Render should succeed");

        // Check that we have red pixels
        // At least one pixel should be red
        let mut red_count = 0;
        for chunk in pixels.chunks_exact(4) {
            if chunk[0] > 200 && chunk[1] < 50 && chunk[2] < 50 {
                red_count += 1;
            }
        }

        assert!(
            red_count > 0,
            "Custom red background should produce red pixels"
        );
    }

    // --- Particle Effect Tests ---

    #[test]
    fn test_particle_effect_add() {
        let mut renderer = Renderer::new(100, 100);
        let bounds = CharBounds {
            x: 50.0,
            y: 50.0,
            width: 20.0,
            height: 20.0,
        };

        // Add a particle effect
        renderer.add_particle_effect(EffectPreset::Fire, bounds, 42);

        // Verify emitter was added
        assert!(
            !renderer.particle_system.particle_emitters.is_empty(),
            "Particle emitter should be added"
        );
    }

    #[test]
    fn test_burst_effect() {
        let mut renderer = Renderer::new(100, 100);

        // Trigger a burst effect
        renderer.burst_effect(EffectPreset::Sparkle, 50.0, 50.0, 20.0, 20.0, 123);

        // Verify emitter was created
        assert!(
            !renderer.particle_system.particle_emitters.is_empty(),
            "Burst emitter should be created"
        );
    }

    #[test]
    fn test_clear_particles() {
        let mut renderer = Renderer::new(100, 100);

        // Add some effects
        renderer.burst_effect(EffectPreset::Fire, 50.0, 50.0, 20.0, 20.0, 1);
        renderer.burst_effect(EffectPreset::Sparkle, 50.0, 50.0, 20.0, 20.0, 2);

        assert!(
            !renderer.particle_system.particle_emitters.is_empty(),
            "Should have emitters before clear"
        );

        // Clear all
        renderer.clear_particles();

        assert!(
            renderer.particle_system.particle_emitters.is_empty(),
            "All emitters should be cleared"
        );
    }

    // --- Text Renderer Access Tests ---

    #[test]
    fn test_text_renderer_mut() {
        let mut renderer = Renderer::new(100, 100);

        // Get mutable reference
        let text_renderer = renderer.text_renderer_mut();

        // Should be able to use it (verify it's a valid reference)
        assert!(
            text_renderer.get_default_typeface().is_none(),
            "Text renderer should be accessible"
        );
    }

    #[test]
    fn test_render_stroke_reveal() {
        let mut renderer = Renderer::new(100, 100);
        let mut doc = minimal_doc();

        // Add a line with text and stroke reveal effect
        let mut line = crate::model::Line::default();
        line.text = Some("Test".to_string());
        line.start = 0.0;
        line.end = 5.0;
        line.effects = vec!["reveal".to_string()];

        // Populate chars
        let chars = vec!["T", "e", "s", "t"];
        line.chars = chars.iter().enumerate().map(|(i, c)| crate::model::Char {
            char: c.to_string(),
            start: i as f64,
            end: (i + 1) as f64,
            style: None,
            font: None,
            stroke: None,
            shadow: None,
            effects: Vec::new(),
            transform: None,
        }).collect();

        doc.lines.push(line);

        // Add effect definition
        let mut effect = crate::model::Effect::default();
        effect.effect_type = crate::model::EffectType::StrokeReveal;
        effect.duration = Some(1.0);
        doc.effects.insert("reveal".to_string(), effect);

        // Render at 0.5s (partial reveal)
        let result = renderer.render_frame(&doc, 0.5);
        assert!(result.is_ok(), "Rendering stroke reveal should succeed");
    }
}

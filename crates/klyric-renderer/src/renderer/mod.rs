pub mod utils;
pub mod particle_system;
pub mod line_renderer;

use anyhow::Result;
use tiny_skia::{Pixmap, Color};
use std::collections::HashSet;

use crate::model::KLyricDocumentV2;
use crate::text::TextRenderer;
use crate::presets::{CharBounds, EffectPreset};

use self::particle_system::ParticleRenderSystem;
use self::line_renderer::LineRenderer;
use self::utils::parse_color;

pub struct Renderer {
    width: u32,
    height: u32,
    text_renderer: TextRenderer,
    particle_system: ParticleRenderSystem,
    /// Last rendered time for delta calculation
    last_time: f64,
}

impl Renderer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            text_renderer: TextRenderer::new(),
            particle_system: ParticleRenderSystem::new(),
            last_time: 0.0,
        }
    }
    
    pub fn text_renderer_mut(&mut self) -> &mut TextRenderer {
        &mut self.text_renderer
    }

    pub fn render_frame(&mut self, doc: &KLyricDocumentV2, time: f64) -> Result<Pixmap> {
        let mut pixmap = Pixmap::new(self.width, self.height)
            .ok_or_else(|| anyhow::anyhow!("Failed to create pixmap"))?;
            
        // Calculate delta time
        let dt = if self.last_time > 0.0 { (time - self.last_time).max(0.0) } else { 0.0 };
        self.last_time = time;
            
        // 1. Draw Background
        self.draw_background(&mut pixmap, doc);
        
        // Track which emitters are active this frame
        let mut active_emitter_keys = HashSet::new();
        
        // 2. Find Active Lines and render
        if let Some(line) = doc.get_active_line(time) {
             // We need the line index to create unique keys
             if let Some(line_idx) = doc.lines.iter().position(|l| l as *const _ == line as *const _) {
                 let mut line_renderer = LineRenderer {
                     pixmap: &mut pixmap,
                     doc,
                     time,
                     text_renderer: &mut self.text_renderer,
                     particle_system: &mut self.particle_system,
                     active_keys: &mut active_emitter_keys,
                     width: self.width,
                     height: self.height,
                 };
                 
                 line_renderer.render_line(line, line_idx)?;
             }
        }
        
        // 3. Update and render particles
        self.particle_system.update(dt as f32, &active_emitter_keys);
        self.particle_system.render(&mut pixmap);
        
        Ok(pixmap)
    }

    /// Add a manual particle effect (e.g. for testing)
    pub fn add_particle_effect(&mut self, preset: EffectPreset, bounds: CharBounds, seed: u64) {
        self.particle_system.add_manual_effect(preset, bounds, seed);
    }

    /// Trigger a burst effect at given position
    pub fn burst_effect(&mut self, preset: EffectPreset, x: f32, y: f32, width: f32, height: f32, seed: u64) {
        let bounds = CharBounds { x, y, width, height };
        self.particle_system.burst_effect(preset, bounds, seed);
    }

    /// Clear all particle emitters
    pub fn clear_particles(&mut self) {
        self.particle_system.clear();
    }
    
    fn draw_background(&self, pixmap: &mut Pixmap, doc: &KLyricDocumentV2) {
        if let Some(theme) = &doc.theme {
            if let Some(bg) = &theme.background {
                if let Some(hex) = &bg.color {
                     if let Some(color) = parse_color(hex) {
                         pixmap.fill(color);
                     }
                }
            }
        } else {
            // Default black
            pixmap.fill(Color::BLACK);
        }
    }
}

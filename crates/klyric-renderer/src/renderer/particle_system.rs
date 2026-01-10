use std::collections::{HashMap, HashSet};
use tiny_skia::{Pixmap, Color, Paint, FillRule, PathBuilder, Rect, Transform as SkiaTransform};
use crate::particle::{Particle, ParticleEmitter, ParticleShape, BlendMode, color_to_rgba};
use crate::presets::{CharBounds, EffectPreset, PresetFactory};

pub struct ParticleRenderSystem {
    /// Active particle emitters keyed by "{line_index}_{char_index}_{effect_name}"
    pub particle_emitters: HashMap<String, ParticleEmitter>,
    pub preset_factory: PresetFactory,
}

impl ParticleRenderSystem {
    pub fn new() -> Self {
        Self {
            particle_emitters: HashMap::new(),
            preset_factory: PresetFactory::new(),
        }
    }

    pub fn update(&mut self, dt: f32, active_keys: &HashSet<String>) {
        // Remove emitters that are not active this frame AND are empty
        // Or if they are burst effects (start with burst_), keep them until empty
        
        self.particle_emitters.retain(|key, emitter| {
            let is_manual_or_burst = key.starts_with("manual_") || key.starts_with("burst_");
            let is_active_frame = active_keys.contains(key);
            
            // If it's a frame-based emitter and not active this frame, stop spawning
            if !is_manual_or_burst {
                emitter.active = is_active_frame;
            }
            
            // Update the emitter
            emitter.update(dt);
            
            // Keep if it has particles or (it's active/manual/burst)
            !emitter.is_empty() || emitter.active
        });
    }

    pub fn render(&self, pixmap: &mut Pixmap) {
        for emitter in self.particle_emitters.values() {
            for particle in &emitter.particles {
                self.draw_particle(pixmap, particle, &emitter.config.blend_mode);
            }
        }
    }

    /// Add a manual particle effect (e.g. for testing)
    pub fn add_manual_effect(&mut self, preset: EffectPreset, bounds: CharBounds, seed: u64) {
        let key = format!("manual_{}", seed);
        let emitter = self.preset_factory.create_from_enum(preset, &bounds, seed);
        self.particle_emitters.insert(key, emitter);
    }

    /// Trigger a burst effect at given position
    pub fn burst_effect(&mut self, preset: EffectPreset, bounds: CharBounds, seed: u64) {
        let mut emitter = self.preset_factory.create_from_enum(preset, &bounds, seed);
        emitter.burst();
        // Burst effects are self-managed (they die when empty)
        // We use a random key to store them
        let key = format!("burst_{}_{}", seed, bounds.x); 
        self.particle_emitters.insert(key, emitter);
    }
    
    pub fn ensure_emitter(&mut self, key: String, preset_name: Option<String>, config: Option<crate::particle::ParticleConfig>, bounds: CharBounds, seed: u64) {
         if !self.particle_emitters.contains_key(&key) {
             let emitter = if let Some(name) = preset_name {
                 // Try to load by name first
                 if let Some(e) = self.preset_factory.create(&name, &bounds, seed) {
                     Some(e)
                 } else {
                     // Fallback: try enum parsing for legacy
                     if let Some(p) = EffectPreset::from_str(&name) {
                         Some(self.preset_factory.create_from_enum(p, &bounds, seed))
                     } else {
                         None
                     }
                 }
             } else if let Some(c) = config {
                 Some(ParticleEmitter::new(
                     c.clone(), 
                     bounds.spawn_center(), 
                     seed
                 ))
             } else {
                 None
             };
             
             if let Some(e) = emitter {
                 self.particle_emitters.insert(key, e);
             }
         } else {
             // Update existing emitter bounds
             if let Some(emitter) = self.particle_emitters.get_mut(&key) {
                 emitter.active = true;
                 
                 // Update spawn pattern based on new bounds
                 match &mut emitter.spawn_pattern {
                     crate::particle::SpawnPattern::Point { x, y } => {
                         *x = bounds.x + bounds.width / 2.0;
                         *y = bounds.y + bounds.height / 2.0;
                     }
                     crate::particle::SpawnPattern::Rect { x, y, w, h } => {
                         *x = bounds.x;
                         *y = bounds.y;
                         *w = bounds.width;
                         *h = bounds.height;
                     }
                     crate::particle::SpawnPattern::Line { x1, y1, x2, y2 } => {
                         let _width_diff = bounds.width - (*x2 - *x1);
                         *x1 = bounds.x;
                         *x2 = bounds.x + bounds.width;
                         *y1 = bounds.y - 50.0;
                         *y2 = bounds.y - 50.0;
                     }
                 }
             }
         }
    }

    pub fn clear(&mut self) {
        self.particle_emitters.clear();
    }

    fn draw_particle(&self, pixmap: &mut Pixmap, particle: &Particle, blend_mode: &BlendMode) {
        let (r, g, b, _a) = color_to_rgba(particle.color);
        let alpha = (particle.opacity * 255.0) as u8;
        
        let color = Color::from_rgba8(r, g, b, alpha);
        
        let mut paint = Paint::default();
        paint.set_color(color);
        paint.anti_alias = true;
        
        // Apply blend mode
        match blend_mode {
            BlendMode::Additive => {
                paint.blend_mode = tiny_skia::BlendMode::Plus;
            }
            BlendMode::Multiply => {
                paint.blend_mode = tiny_skia::BlendMode::Multiply;
            }
            BlendMode::Normal => {
                paint.blend_mode = tiny_skia::BlendMode::SourceOver;
            }
        }
        
        match &particle.shape {
            ParticleShape::Circle => {
                let mut pb = PathBuilder::new();
                pb.push_circle(particle.x, particle.y, particle.size / 2.0);
                if let Some(path) = pb.finish() {
                    let transform = SkiaTransform::from_rotate_at(
                        particle.rotation,
                        particle.x,
                        particle.y
                    );
                    pixmap.fill_path(&path, &paint, FillRule::Winding, transform, None);
                }
            }
            ParticleShape::Square => {
                let half = particle.size / 2.0;
                let mut pb = PathBuilder::new();
                if let Some(rect) = Rect::from_xywh(
                    particle.x - half,
                    particle.y - half,
                    particle.size,
                    particle.size,
                ) {
                    pb.push_rect(rect);
                }
                if let Some(path) = pb.finish() {
                    let transform = SkiaTransform::from_rotate_at(
                        particle.rotation,
                        particle.x,
                        particle.y
                    );
                    pixmap.fill_path(&path, &paint, FillRule::Winding, transform, None);
                }
            }
            ParticleShape::Char(_ch) => {
                let mut pb = PathBuilder::new();
                pb.push_circle(particle.x, particle.y, particle.size / 2.0);
                if let Some(path) = pb.finish() {
                    pixmap.fill_path(&path, &paint, FillRule::Winding, SkiaTransform::identity(), None);
                }
            }
            ParticleShape::Image(_path) => {
                let mut pb = PathBuilder::new();
                pb.push_circle(particle.x, particle.y, particle.size / 2.0);
                if let Some(path) = pb.finish() {
                    pixmap.fill_path(&path, &paint, FillRule::Winding, SkiaTransform::identity(), None);
                }
            }
        }
    }
}

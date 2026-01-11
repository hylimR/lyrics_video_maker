use std::collections::{HashMap, HashSet};
use skia_safe::{Canvas, Color, Paint, Rect, Matrix, Path, Point, Image, BlendMode as SkBlendMode};
use crate::particle::{Particle, ParticleEmitter, ParticleShape, BlendMode, color_to_rgba, ParticleConfig, SpawnPattern, RangeValue, ParticlePhysics};
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

    pub fn render(&self, canvas: &Canvas) {
        for emitter in self.particle_emitters.values() {
            for particle in &emitter.particles {
                self.draw_particle(canvas, particle, &emitter.config.blend_mode);
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

    /// Create and register a one-shot disintegration emitter from an image
    pub fn ensure_disintegration_emitter(
        &mut self,
        key: String,
        image: &Image,
        bounds: CharBounds,
        seed: u64,
        config: Option<ParticleConfig>,
    ) {
        if self.particle_emitters.contains_key(&key) {
             if let Some(emitter) = self.particle_emitters.get_mut(&key) {
                // Ensure it stays alive while needed, though usually this is a one-shot burst
                emitter.active = true;
             }
            return;
        }

        let mut rng = crate::particle::Rng::new(seed);

        let mut base_config = config.unwrap_or_else(|| ParticleConfig {
            count: 0, // Ignored for manual spawn
            spawn_rate: 0.0,
            lifetime: RangeValue::Range(0.5, 1.2),
            speed: RangeValue::Range(20.0, 60.0),
            direction: RangeValue::Range(220.0, 320.0), // Up-Left to Up-Right
            spread: 45.0,
            start_size: RangeValue::Range(2.0, 4.0),
            end_size: RangeValue::Single(0.0),
            rotation_speed: RangeValue::Range(-90.0, 90.0),
            color: "#FFFFFF".to_string(),
            shape: ParticleShape::Square,
            physics: ParticlePhysics {
                gravity: 200.0, // Fall down eventually
                drag: 0.5,
                wind_x: 0.0,
                wind_y: 0.0,
            },
            blend_mode: BlendMode::Normal,
        });

        // Disable automatic spawning
        base_config.spawn_rate = 0.0;

        let mut emitter = ParticleEmitter::new(
            base_config.clone(),
            SpawnPattern::Point { x: 0.0, y: 0.0 }, // Dummy pattern
            seed,
        );

        // Sampling step - don't spawn a particle for every single pixel, that's too heavy
        // Dynamically adjust step based on size to keep particle count reasonable
        let pixel_count = image.width() * image.height();
        let step = if pixel_count > 5000 {
            3
        } else if pixel_count > 1000 {
            2
        } else {
            1
        };

        let pm_w = image.width() as f32;
        let pm_h = image.height() as f32;

        // Map pixmap coordinates to screen bounds
        // bounds.x/y is top-left on screen

        if let Some(pixmap) = image.peek_pixels() {
            let width = pixmap.width();
            let height = pixmap.height();
            // Assuming N32 format (4 bytes per pixel)
            // skia uses row_bytes()
            let row_bytes = pixmap.row_bytes();
            
            if let Some(data) = pixmap.bytes() {
                // Check color type...
            
                for y in (0..height).step_by(step) {
                    for x in (0..width).step_by(step) {
                        let offset = y as usize * row_bytes + x as usize * 4;
                        if offset + 3 >= data.len() { break; }
                    
                    // Skia usually stores premultiplied coords. 
                    // Let's grab u32 to be safe if we want full color, 
                    // but we need to know byte order for R, G, B.
                    // For "Disintegration" often white particles are fine or we guess.
                    // Let's assume byte 3 is Alpha (if RGBA or BGRA).
                    // Actually, if it's BGRA, A is 3. If RGBA, A is 3.
                    // So data[offset+3] is likely Alpha.
                    
                    let a = data[offset + 3];

                    if a > 10 { // Threshold alpha
                        // Approximate color - we might get R/B swapped but usually OK for particles
                        let r = data[offset];
                        let g = data[offset + 1];
                        let b = data[offset + 2];

                        // Create particle
                        let px = bounds.x + (x as f32 / pm_w) * bounds.width;
                        let py = bounds.y + (y as f32 / pm_h) * bounds.height;

                        let color: u32 = ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (a as u32);

                        // Manual spawn logic from emitter
                        // Calculate velocity from direction + spread
                        let base_dir = base_config.direction.sample(&mut rng);
                        let spread_offset = rng.range(-base_config.spread / 2.0, base_config.spread / 2.0);
                        let dir_rad = (base_dir + spread_offset).to_radians();

                        let speed = base_config.speed.sample(&mut rng);
                        let vx = dir_rad.cos() * speed;
                        let vy = dir_rad.sin() * speed;

                        // Add some random jitter to position so grid isn't obvious
                        let jx = rng.range(-1.0, 1.0);
                        let jy = rng.range(-1.0, 1.0);

                        let particle = Particle {
                            x: px + jx,
                            y: py + jy,
                            vx,
                            vy,
                            life: 0.0,
                            max_life: base_config.lifetime.sample(&mut rng),
                            size: base_config.start_size.sample(&mut rng),
                            start_size: base_config.start_size.sample(&mut rng),
                            end_size: base_config.end_size.sample(&mut rng),
                            rotation: rng.range(0.0, 360.0),
                            rotation_speed: base_config.rotation_speed.sample(&mut rng),
                            color,
                            opacity: 1.0,
                            shape: base_config.shape.clone(),
                        };

                        emitter.particles.push(particle);
                    }
                }
            }
            }
        }

        self.particle_emitters.insert(key, emitter);
    }

    pub fn clear(&mut self) {
        self.particle_emitters.clear();
    }

    fn draw_particle(&self, canvas: &Canvas, particle: &Particle, blend_mode: &BlendMode) {
        let (r, g, b, _a) = color_to_rgba(particle.color);
        let alpha = (particle.opacity * 255.0) as u8;
        
        // Skia colors
        let color = Color::from_argb(alpha, r, g, b);
        
        let mut paint = Paint::default();
        paint.set_color(color);
        paint.set_anti_alias(true);
        
        // Apply blend mode
        match blend_mode {
            BlendMode::Additive => {
                paint.set_blend_mode(SkBlendMode::Plus);
            }
            BlendMode::Multiply => {
                 paint.set_blend_mode(SkBlendMode::Multiply);
            }
            BlendMode::Normal => {
                 paint.set_blend_mode(SkBlendMode::SrcOver);
            }
        }
        
        canvas.save();
        
        // Translate to particle position
        canvas.translate((particle.x, particle.y));
        canvas.rotate(particle.rotation, None);
        
        match &particle.shape {
            ParticleShape::Circle => {
                let radius = particle.size / 2.0;
                // Draw circle at (0,0) since we translated
                canvas.draw_circle(Point::new(0.0, 0.0), radius, &paint);
            }
            ParticleShape::Square => {
                let half = particle.size / 2.0;
                let rect = Rect::from_xywh(-half, -half, particle.size, particle.size);
                canvas.draw_rect(rect, &paint);
            }
            ParticleShape::Char(_ch) => {
                let radius = particle.size / 2.0;
                canvas.draw_circle(Point::new(0.0, 0.0), radius, &paint);
            }
            ParticleShape::Image(_path) => {
                let radius = particle.size / 2.0;
                canvas.draw_circle(Point::new(0.0, 0.0), radius, &paint);
            }
        }
        
        canvas.restore();
    }
}

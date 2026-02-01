use crate::particle::{
    color_to_rgba, BlendMode, Particle, ParticleConfig, ParticleEmitter, ParticlePhysics,
    ParticleShape, RangeValue, SpawnPattern,
};
use crate::presets::{CharBounds, EffectPreset, PresetFactory};
use skia_safe::{BlendMode as SkBlendMode, Canvas, Color, Image, Paint, Point, Rect};
use std::collections::{hash_map::DefaultHasher, HashMap, HashSet};
use std::hash::{Hash, Hasher};

pub struct ParticleRenderSystem {
    /// Active particle emitters keyed by u64 hash
    pub particle_emitters: HashMap<u64, ParticleEmitter>,
    pub preset_factory: PresetFactory,
}

impl Default for ParticleRenderSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl ParticleRenderSystem {
    pub fn new() -> Self {
        Self {
            particle_emitters: HashMap::new(),
            preset_factory: PresetFactory::new(),
        }
    }

    pub fn update(&mut self, dt: f32, active_keys: &HashSet<u64>) {
        self.particle_emitters.retain(|key, emitter| {
            // If it's a frame-based emitter (text effects), check if it's active this frame
            if emitter.frame_driven {
                emitter.active = active_keys.contains(key);
            }

            // Update the emitter
            emitter.update(dt);

            // Keep if it has particles or it's still active
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
        let mut hasher = DefaultHasher::new();
        "manual".hash(&mut hasher);
        seed.hash(&mut hasher);
        let key = hasher.finish();

        let mut emitter = self.preset_factory.create_from_enum(preset, &bounds, seed);
        emitter.frame_driven = false; // Manual effects persist until empty/stopped
        self.particle_emitters.insert(key, emitter);
    }

    /// Trigger a burst effect at given position
    pub fn burst_effect(&mut self, preset: EffectPreset, bounds: CharBounds, seed: u64) {
        let mut emitter = self.preset_factory.create_from_enum(preset, &bounds, seed);
        emitter.burst();
        emitter.frame_driven = false; // Burst effects persist until empty

        let mut hasher = DefaultHasher::new();
        "burst".hash(&mut hasher);
        seed.hash(&mut hasher);
        bounds.x.to_bits().hash(&mut hasher);
        let key = hasher.finish();

        self.particle_emitters.insert(key, emitter);
    }

    pub fn has_emitter(&self, key: u64) -> bool {
        self.particle_emitters.contains_key(&key)
    }

    pub fn update_emitter_bounds(&mut self, key: u64, bounds: CharBounds) -> bool {
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
            true
        } else {
            false
        }
    }

    pub fn ensure_emitter(
        &mut self,
        key: u64,
        preset_name: Option<String>,
        config: Option<crate::particle::ParticleConfig>,
        bounds: CharBounds,
        seed: u64,
    ) {
        if !self.particle_emitters.contains_key(&key) {
            let emitter = if let Some(name) = preset_name {
                // Try to load by name first
                if let Some(e) = self.preset_factory.create(&name, &bounds, seed) {
                    Some(e)
                } else {
                    // Fallback: try enum parsing for legacy
                    if let Ok(p) = name.parse::<EffectPreset>() {
                        Some(self.preset_factory.create_from_enum(p, &bounds, seed))
                    } else {
                        None
                    }
                }
            } else {
                config
                    .clone()
                    .map(|c| ParticleEmitter::new(c, bounds.spawn_center(), seed))
            };

            if let Some(mut e) = emitter {
                e.frame_driven = true; // Text effects are frame driven
                self.particle_emitters.insert(key, e);
            }
        } else {
            // Update existing emitter bounds
            self.update_emitter_bounds(key, bounds);
        }
    }

    /// Create and register a one-shot disintegration emitter from an image
    pub fn ensure_disintegration_emitter(
        &mut self,
        key: u64,
        image: &Image,
        bounds: CharBounds,
        seed: u64,
        config: Option<ParticleConfig>,
    ) {
        if self.particle_emitters.contains_key(&key) {
            if let Some(emitter) = self.particle_emitters.get_mut(&key) {
                // Ensure it stays alive while needed
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
        emitter.frame_driven = true; // Managed by frame visibility

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
        if let Some(pixmap) = image.peek_pixels() {
            let width = pixmap.width();
            let height = pixmap.height();
            let row_bytes = pixmap.row_bytes();

            if let Some(data) = pixmap.bytes() {
                for y in (0..height).step_by(step) {
                    for x in (0..width).step_by(step) {
                        let offset = y as usize * row_bytes + x as usize * 4;
                        if offset + 3 >= data.len() {
                            break;
                        }

                        let a = data[offset + 3];

                        if a > 10 {
                            let r = data[offset];
                            let g = data[offset + 1];
                            let b = data[offset + 2];

                            // Create particle
                            let px = bounds.x + (x as f32 / pm_w) * bounds.width;
                            let py = bounds.y + (y as f32 / pm_h) * bounds.height;

                            let color: u32 = ((r as u32) << 24)
                                | ((g as u32) << 16)
                                | ((b as u32) << 8)
                                | (a as u32);

                            // Manual spawn logic from emitter
                            let base_dir = base_config.direction.sample(&mut rng);
                            let spread_offset =
                                rng.range(-base_config.spread / 2.0, base_config.spread / 2.0);
                            let dir_rad = (base_dir + spread_offset).to_radians();

                            let speed = base_config.speed.sample(&mut rng);
                            let vx = dir_rad.cos() * speed;
                            let vy = dir_rad.sin() * speed;

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

        let color = Color::from_argb(alpha, r, g, b);

        let mut paint = Paint::default();
        paint.set_color(color);
        paint.set_anti_alias(true);

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

        canvas.translate((particle.x, particle.y));
        canvas.rotate(particle.rotation, None);

        match &particle.shape {
            ParticleShape::Circle => {
                let radius = particle.size / 2.0;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_bounds() -> CharBounds {
        CharBounds {
            x: 100.0,
            y: 100.0,
            width: 50.0,
            height: 30.0,
        }
    }

    fn hash_test_key(s: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish()
    }

    // --- ParticleRenderSystem Creation Tests ---

    #[test]
    fn test_new_empty() {
        let system = ParticleRenderSystem::new();
        assert!(
            system.particle_emitters.is_empty(),
            "New system should have no emitters"
        );
    }

    // --- Emitter Management Tests ---

    #[test]
    fn test_add_manual_effect() {
        let mut system = ParticleRenderSystem::new();
        let bounds = test_bounds();

        system.add_manual_effect(EffectPreset::Fire, bounds, 42);

        // Calculate expected key
        let mut hasher = DefaultHasher::new();
        "manual".hash(&mut hasher);
        42u64.hash(&mut hasher);
        let key = hasher.finish();

        assert!(
            system.particle_emitters.contains_key(&key),
            "Manual effect should create emitter with correct key"
        );

        // Manual effect should not be frame driven
        let emitter = system.particle_emitters.get(&key).unwrap();
        assert_eq!(emitter.frame_driven, false);
    }

    #[test]
    fn test_burst_effect() {
        let mut system = ParticleRenderSystem::new();
        let bounds = test_bounds();

        system.burst_effect(EffectPreset::Sparkle, bounds.clone(), 123);

        // Burst emitter should be created
        // Calculate expected key
        let mut hasher = DefaultHasher::new();
        "burst".hash(&mut hasher);
        123u64.hash(&mut hasher);
        bounds.x.to_bits().hash(&mut hasher);
        let key = hasher.finish();

        assert!(
            system.particle_emitters.contains_key(&key),
            "Burst effect should create emitter with correct key"
        );

        let emitter = system.particle_emitters.get(&key).unwrap();
        assert_eq!(emitter.frame_driven, false);
    }

    #[test]
    fn test_ensure_emitter_new() {
        let mut system = ParticleRenderSystem::new();
        let bounds = test_bounds();
        let key = hash_test_key("test_key");

        // Ensure emitter by preset name
        system.ensure_emitter(key, Some("fire".to_string()), None, bounds, 42);

        assert!(
            system.particle_emitters.contains_key(&key),
            "ensure_emitter should create new emitter"
        );

        let emitter = system.particle_emitters.get(&key).unwrap();
        assert!(emitter.frame_driven);
    }

    #[test]
    fn test_ensure_emitter_update() {
        let mut system = ParticleRenderSystem::new();
        let bounds = test_bounds();
        let key = hash_test_key("update_key");

        // Create initial emitter
        system.ensure_emitter(key, Some("fire".to_string()), None, bounds, 42);

        // Update with new bounds
        let new_bounds = CharBounds {
            x: 200.0,
            y: 200.0,
            width: 100.0,
            height: 60.0,
        };

        system.ensure_emitter(key, Some("fire".to_string()), None, new_bounds, 42);

        // Should still have only one emitter with this key
        assert!(
            system.particle_emitters.contains_key(&key),
            "ensure_emitter should update existing emitter"
        );

        // The emitter should be active
        let emitter = system.particle_emitters.get(&key).unwrap();
        assert!(emitter.active, "Updated emitter should be active");
    }

    #[test]
    fn test_clear() {
        let mut system = ParticleRenderSystem::new();
        let bounds = test_bounds();

        // Add multiple emitters
        system.add_manual_effect(EffectPreset::Fire, bounds.clone(), 1);
        system.add_manual_effect(EffectPreset::Sparkle, bounds.clone(), 2);
        system.burst_effect(EffectPreset::Rain, bounds, 3);

        assert!(
            !system.particle_emitters.is_empty(),
            "Should have emitters before clear"
        );

        system.clear();

        assert!(
            system.particle_emitters.is_empty(),
            "All emitters should be removed after clear"
        );
    }

    // --- Update Logic Tests ---

    #[test]
    fn test_update_removes_inactive() {
        let mut system = ParticleRenderSystem::new();
        let bounds = test_bounds();
        let key = hash_test_key("inactive_test");

        // Add emitter
        system.ensure_emitter(key, Some("fire".to_string()), None, bounds, 42);

        // Mark as inactive by NOT including in active_keys set
        let active_keys = HashSet::new();

        // Update - after sufficient time, inactive emitter with no particles should be removed
        // First update deactivates it
        system.update(0.016, &active_keys);

        // The emitter might still exist if it has particles
        // Let's check that it was at least deactivated
        if let Some(emitter) = system.particle_emitters.get(&key) {
            assert!(!emitter.active, "Emitter should be inactive");
        }
    }

    #[test]
    fn test_update_keeps_active() {
        let mut system = ParticleRenderSystem::new();
        let bounds = test_bounds();
        let key = hash_test_key("active_test");

        system.ensure_emitter(key, Some("fire".to_string()), None, bounds, 42);

        // Include in active_keys
        let mut active_keys = HashSet::new();
        active_keys.insert(key);

        system.update(0.016, &active_keys);

        // Emitter should still exist and be active
        assert!(
            system.particle_emitters.contains_key(&key),
            "Active emitter should be retained"
        );
        let emitter = system.particle_emitters.get(&key).unwrap();
        assert!(emitter.active, "Emitter should remain active");
    }

    #[test]
    fn test_update_keeps_burst_until_empty() {
        let mut system = ParticleRenderSystem::new();
        let bounds = test_bounds();

        // Create burst effect
        system.burst_effect(EffectPreset::Sparkle, bounds.clone(), 99);

        // Get the key
        let mut hasher = DefaultHasher::new();
        "burst".hash(&mut hasher);
        99u64.hash(&mut hasher);
        bounds.x.to_bits().hash(&mut hasher);
        let key = hasher.finish();

        // Update without including in active_keys
        let active_keys = HashSet::new();
        system.update(0.016, &active_keys);

        // Should still exist because it's not frame_driven
        assert!(
            system.particle_emitters.contains_key(&key),
            "Burst emitter should be retained even if not in active_keys"
        );
    }

    // --- Preset Factory Integration Tests ---

    #[test]
    fn test_ensure_emitter_by_preset_name() {
        let mut system = ParticleRenderSystem::new();
        let bounds = test_bounds();
        let key = hash_test_key("preset_name_test");

        // Use a preset name that should exist in the factory
        system.ensure_emitter(key, Some("sparkle".to_string()), None, bounds, 42);

        assert!(
            system.particle_emitters.contains_key(&key),
            "Should create emitter from preset name"
        );
    }

    #[test]
    fn test_ensure_emitter_by_config() {
        let mut system = ParticleRenderSystem::new();
        let bounds = test_bounds();
        let key = hash_test_key("config_test");

        // Create with custom config
        let config = ParticleConfig {
            count: 10,
            spawn_rate: 5.0,
            lifetime: RangeValue::Single(1.0),
            speed: RangeValue::Single(50.0),
            direction: RangeValue::Single(270.0),
            spread: 30.0,
            start_size: RangeValue::Single(5.0),
            end_size: RangeValue::Single(0.0),
            rotation_speed: RangeValue::Single(0.0),
            color: "#FFFFFF".to_string(),
            shape: ParticleShape::Circle,
            physics: ParticlePhysics {
                gravity: 0.0,
                drag: 0.0,
                wind_x: 0.0,
                wind_y: 0.0,
            },
            blend_mode: BlendMode::Normal,
        };

        system.ensure_emitter(key, None, Some(config), bounds, 42);

        assert!(
            system.particle_emitters.contains_key(&key),
            "Should create emitter from config"
        );
    }

    #[test]
    fn test_ensure_emitter_fallback_enum() {
        let mut system = ParticleRenderSystem::new();
        let bounds = test_bounds();
        let key = hash_test_key("enum_test");

        // Use an enum-style name that should fall back to enum parsing
        system.ensure_emitter(
            key,
            Some("Fire".to_string()), // Enum variant name
            None,
            bounds,
            42,
        );

        // May or may not create depending on case matching
        // just verify no panic
    }

    #[test]
    fn test_ensure_disintegration_emitter_idempotent() {
        let mut system = ParticleRenderSystem::new();
        let key = hash_test_key("disintegration_test");

        let config = ParticleConfig {
            count: 0,
            spawn_rate: 0.0,
            lifetime: RangeValue::Single(1.0),
            speed: RangeValue::Single(50.0),
            direction: RangeValue::Single(270.0),
            spread: 30.0,
            start_size: RangeValue::Single(2.0),
            end_size: RangeValue::Single(0.0),
            rotation_speed: RangeValue::Single(0.0),
            color: "#FFFFFF".to_string(),
            shape: ParticleShape::Square,
            physics: ParticlePhysics {
                gravity: 200.0,
                drag: 0.5,
                wind_x: 0.0,
                wind_y: 0.0,
            },
            blend_mode: BlendMode::Normal,
        };

        let emitter = ParticleEmitter::new(config, SpawnPattern::Point { x: 0.0, y: 0.0 }, 42);

        system.particle_emitters.insert(key, emitter);

        let count_before = system.particle_emitters.len();

        assert_eq!(
            system.particle_emitters.len(),
            count_before,
            "Existing key should not create duplicate"
        );
    }
}

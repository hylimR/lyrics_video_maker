//! Particle emitter - spawns and manages particle lifecycles

use super::config::{ParticleConfig, SpawnPattern};
use super::rng::Rng;
use super::types::{parse_hex_color, Particle};

/// Particle emitter that spawns and manages particles
#[derive(Debug, Clone)]
pub struct ParticleEmitter {
    /// Active particles
    pub particles: Vec<Particle>,
    /// Emission configuration
    pub config: ParticleConfig,
    /// Spawn location pattern
    pub spawn_pattern: SpawnPattern,
    /// Time accumulator for spawn rate
    spawn_accumulator: f32,
    /// Random number generator
    pub rng: Rng,
    /// Whether emitter is active
    pub active: bool,
    /// Parsed color from config
    color_rgba: u32,
    /// Total time emitter has been running
    pub elapsed: f32,
    /// If true, the emitter is managed by the frame loop (auto-deactivated if not touched)
    pub frame_driven: bool,
}

impl ParticleEmitter {
    pub fn new(config: ParticleConfig, spawn_pattern: SpawnPattern, seed: u64) -> Self {
        let color_rgba = parse_hex_color(&config.color);
        Self {
            particles: Vec::with_capacity(config.count as usize * 2),
            config,
            spawn_pattern,
            spawn_accumulator: 0.0,
            rng: Rng::new(seed),
            active: true,
            color_rgba,
            elapsed: 0.0,
            frame_driven: true,
        }
    }

    /// Emit a burst of particles immediately
    pub fn burst(&mut self) {
        for _ in 0..self.config.count {
            self.spawn_particle();
        }
    }

    /// Spawn a single particle
    fn spawn_particle(&mut self) {
        let (x, y) = self.spawn_pattern.sample(&mut self.rng);

        // Calculate velocity from direction + spread
        let base_dir = self.config.direction.sample(&mut self.rng);
        let spread_offset = self
            .rng
            .range(-self.config.spread / 2.0, self.config.spread / 2.0);
        let dir_rad = (base_dir + spread_offset).to_radians();

        let speed = self.config.speed.sample(&mut self.rng);
        let vx = dir_rad.cos() * speed;
        let vy = dir_rad.sin() * speed;

        let particle = Particle {
            x,
            y,
            vx,
            vy,
            life: 0.0,
            max_life: self.config.lifetime.sample(&mut self.rng),
            size: self.config.start_size.sample(&mut self.rng),
            start_size: self.config.start_size.sample(&mut self.rng),
            end_size: self.config.end_size.sample(&mut self.rng),
            rotation: self.rng.range(0.0, 360.0),
            rotation_speed: self.config.rotation_speed.sample(&mut self.rng),
            color: self.color_rgba,
            opacity: 1.0,
            shape: self.config.shape.clone(),
        };

        self.particles.push(particle);
    }

    /// Update all particles and spawn new ones based on spawn_rate
    pub fn update(&mut self, dt: f32) {
        self.elapsed += dt;

        // Update existing particles
        for particle in &mut self.particles {
            particle.update(dt, &self.config.physics);
        }

        // Remove dead particles
        self.particles.retain(|p| p.is_alive());

        // Spawn new particles if spawn_rate > 0
        if self.active && self.config.spawn_rate > 0.0 {
            self.spawn_accumulator += dt;
            let spawn_interval = 1.0 / self.config.spawn_rate;
            while self.spawn_accumulator >= spawn_interval {
                self.spawn_accumulator -= spawn_interval;
                for _ in 0..self.config.count {
                    self.spawn_particle();
                }
            }
        }
    }

    /// Check if emitter has any active particles
    pub fn is_empty(&self) -> bool {
        self.particles.is_empty()
    }

    /// Deactivate emitter (stops spawning but lets existing particles die)
    pub fn stop(&mut self) {
        self.active = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::particle::RangeValue;

    #[test]
    fn test_emitter_burst() {
        let config = ParticleConfig {
            count: 5,
            ..Default::default()
        };

        let mut emitter =
            ParticleEmitter::new(config, SpawnPattern::Point { x: 100.0, y: 100.0 }, 42);

        assert!(emitter.particles.is_empty());
        emitter.burst();
        assert_eq!(emitter.particles.len(), 5);
    }

    #[test]
    fn test_emitter_update() {
        let config = ParticleConfig {
            count: 1,
            spawn_rate: 10.0, // 10 per second
            ..Default::default()
        };

        let mut emitter = ParticleEmitter::new(config, SpawnPattern::Point { x: 0.0, y: 0.0 }, 42);

        // Should spawn particles over time
        emitter.update(0.5);
        assert!(!emitter.particles.is_empty());
    }

    #[test]
    fn test_emitter_lifecycle() {
        let config = ParticleConfig {
            count: 3,
            lifetime: RangeValue::Single(0.1), // Short life
            ..Default::default()
        };

        let mut emitter = ParticleEmitter::new(config, SpawnPattern::Point { x: 0.0, y: 0.0 }, 42);

        emitter.burst();
        assert_eq!(emitter.particles.len(), 3);

        // Update past lifetime
        emitter.update(0.2);
        assert!(emitter.particles.is_empty());
    }
}

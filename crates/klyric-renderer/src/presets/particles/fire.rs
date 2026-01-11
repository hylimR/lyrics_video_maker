use crate::particle::{
    BlendMode, ParticleConfig, ParticleEmitter, ParticlePhysics, ParticleShape, RangeValue,
    SpawnPattern,
};
use super::super::types::CharBounds;
use super::super::traits::ParticlePreset;

/// 🔥 Fire effect - flames rising
pub struct FirePreset;
impl ParticlePreset for FirePreset {
    fn create_emitter(&self, bounds: &CharBounds, seed: u64) -> ParticleEmitter {
        let config = ParticleConfig {
            count: 5,
            spawn_rate: 15.0,
            lifetime: RangeValue::Range(0.3, 0.8),
            speed: RangeValue::Range(80.0, 150.0),
            direction: RangeValue::Single(270.0),
            spread: 30.0,
            start_size: RangeValue::Range(8.0, 16.0),
            end_size: RangeValue::Range(2.0, 4.0),
            rotation_speed: RangeValue::Single(0.0),
            color: "#FF6600".to_string(),
            shape: ParticleShape::Circle,
            physics: ParticlePhysics {
                gravity: -200.0,
                wind_x: 0.0,
                wind_y: 0.0,
                drag: 2.0,
            },
            blend_mode: BlendMode::Additive,
        };

        let spawn = SpawnPattern::Line {
            x1: bounds.x,
            y1: bounds.y + bounds.height,
            x2: bounds.x + bounds.width,
            y2: bounds.y + bounds.height,
        };

        ParticleEmitter::new(config, spawn, seed)
    }
}

use crate::particle::{
    BlendMode, ParticleConfig, ParticleEmitter, ParticlePhysics, ParticleShape, RangeValue,
};
use super::super::types::CharBounds;
use super::super::traits::ParticlePreset;

/// ✨ Sparkle effect - glitter burst at center
pub struct SparklePreset;
impl ParticlePreset for SparklePreset {
    fn create_emitter(&self, bounds: &CharBounds, seed: u64) -> ParticleEmitter {
        let config = ParticleConfig {
            count: 12,
            spawn_rate: 0.0,
            lifetime: RangeValue::Range(0.3, 0.8),
            speed: RangeValue::Range(80.0, 200.0),
            direction: RangeValue::Range(0.0, 360.0),
            spread: 0.0,
            start_size: RangeValue::Range(3.0, 8.0),
            end_size: RangeValue::Single(0.0),
            rotation_speed: RangeValue::Range(-360.0, 360.0),
            color: "#FFFF88".to_string(),
            shape: ParticleShape::Circle,
            physics: ParticlePhysics {
                gravity: -50.0,
                wind_x: 0.0,
                wind_y: 0.0,
                drag: 2.0,
            },
            blend_mode: BlendMode::Additive,
        };
        ParticleEmitter::new(config, bounds.spawn_center(), seed)
    }
}

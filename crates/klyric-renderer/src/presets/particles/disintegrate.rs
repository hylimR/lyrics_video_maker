use super::super::traits::ParticlePreset;
use super::super::types::CharBounds;
use crate::particle::{
    BlendMode, ParticleConfig, ParticleEmitter, ParticlePhysics, ParticleShape, RangeValue,
};

/// 💥 Disintegrate effect - character explodes into particles
pub struct DisintegratePreset;
impl ParticlePreset for DisintegratePreset {
    fn create_emitter(&self, bounds: &CharBounds, seed: u64) -> ParticleEmitter {
        let config = ParticleConfig {
            count: 30,
            spawn_rate: 0.0,
            lifetime: RangeValue::Range(0.5, 1.2),
            speed: RangeValue::Range(50.0, 200.0),
            direction: RangeValue::Range(0.0, 360.0),
            spread: 0.0,
            start_size: RangeValue::Range(3.0, 6.0),
            end_size: RangeValue::Single(0.0),
            rotation_speed: RangeValue::Range(-540.0, 540.0),
            color: "#FFFFFF".to_string(),
            shape: ParticleShape::Square,
            physics: ParticlePhysics {
                gravity: 100.0,
                wind_x: 0.0,
                wind_y: 0.0,
                drag: 3.0,
            },
            blend_mode: BlendMode::Normal,
        };
        ParticleEmitter::new(config, bounds.spawn_fill(), seed)
    }
}

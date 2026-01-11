use crate::particle::{
    BlendMode, ParticleConfig, ParticleEmitter, ParticlePhysics, ParticleShape, RangeValue,
};
use super::super::types::CharBounds;
use super::super::traits::ParticlePreset;

/// 🌧️ Rain effect - droplets falling from above
pub struct RainPreset;
impl ParticlePreset for RainPreset {
    fn create_emitter(&self, bounds: &CharBounds, seed: u64) -> ParticleEmitter {
        let config = ParticleConfig {
            count: 3,
            spawn_rate: 8.0,
            lifetime: RangeValue::Range(0.8, 1.5),
            speed: RangeValue::Range(150.0, 300.0),
            direction: RangeValue::Single(90.0),
            spread: 15.0,
            start_size: RangeValue::Range(2.0, 4.0),
            end_size: RangeValue::Range(1.0, 2.0),
            rotation_speed: RangeValue::Single(0.0),
            color: "#6699CC".to_string(),
            shape: ParticleShape::Char("|".to_string()),
            physics: ParticlePhysics {
                gravity: 400.0,
                wind_x: 20.0,
                wind_y: 0.0,
                drag: 0.0,
            },
            blend_mode: BlendMode::Normal,
        };
        ParticleEmitter::new(config, bounds.spawn_above(50.0), seed)
    }
}

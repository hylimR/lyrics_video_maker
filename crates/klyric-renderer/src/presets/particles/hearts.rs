use super::super::traits::ParticlePreset;
use super::super::types::CharBounds;
use crate::particle::{
    BlendMode, ParticleConfig, ParticleEmitter, ParticlePhysics, ParticleShape, RangeValue,
};

/// 💕 Hearts effect - floating hearts
pub struct HeartsPreset;
impl ParticlePreset for HeartsPreset {
    fn create_emitter(&self, bounds: &CharBounds, seed: u64) -> ParticleEmitter {
        let config = ParticleConfig {
            count: 5,
            spawn_rate: 2.0,
            lifetime: RangeValue::Range(1.0, 2.0),
            speed: RangeValue::Range(30.0, 80.0),
            direction: RangeValue::Single(270.0),
            spread: 60.0,
            start_size: RangeValue::Range(12.0, 24.0),
            end_size: RangeValue::Range(8.0, 16.0),
            rotation_speed: RangeValue::Range(-30.0, 30.0),
            color: "#FF6699".to_string(),
            shape: ParticleShape::Char("♥".to_string()),
            physics: ParticlePhysics {
                gravity: -30.0,
                wind_x: 0.0,
                wind_y: 0.0,
                drag: 0.5,
            },
            blend_mode: BlendMode::Normal,
        };
        ParticleEmitter::new(config, bounds.spawn_center(), seed)
    }
}

use crate::particle::{
    BlendMode, ParticleConfig, ParticleEmitter, ParticlePhysics, ParticleShape, RangeValue,
};
use super::super::types::CharBounds;
use super::super::traits::ParticlePreset;

/// 🎊 Confetti effect - colorful explosion
pub struct ConfettiPreset;
impl ParticlePreset for ConfettiPreset {
    fn create_emitter(&self, bounds: &CharBounds, seed: u64) -> ParticleEmitter {
        let config = ParticleConfig {
            count: 20,
            spawn_rate: 0.0,
            lifetime: RangeValue::Range(1.0, 2.5),
            speed: RangeValue::Range(100.0, 300.0),
            direction: RangeValue::Range(200.0, 340.0),
            spread: 0.0,
            start_size: RangeValue::Range(6.0, 12.0),
            end_size: RangeValue::Range(4.0, 8.0),
            rotation_speed: RangeValue::Range(-720.0, 720.0),
            color: "#FF00FF".to_string(),
            shape: ParticleShape::Square,
            physics: ParticlePhysics {
                gravity: 300.0,
                wind_x: 0.0,
                wind_y: 0.0,
                drag: 1.0,
            },
            blend_mode: BlendMode::Normal,
        };
        ParticleEmitter::new(config, bounds.spawn_center(), seed)
    }
}

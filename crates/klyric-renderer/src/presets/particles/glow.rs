use crate::particle::{
    BlendMode, ParticleConfig, ParticleEmitter, ParticlePhysics, ParticleShape, RangeValue,
};
use super::super::types::CharBounds;
use super::super::traits::ParticlePreset;

/// ✨ Glow pulse - subtle pulsing aura
pub struct GlowPulsePreset;
impl ParticlePreset for GlowPulsePreset {
    fn create_emitter(&self, bounds: &CharBounds, seed: u64) -> ParticleEmitter {
        let config = ParticleConfig {
            count: 3,
            spawn_rate: 5.0,
            lifetime: RangeValue::Range(0.4, 0.8),
            speed: RangeValue::Range(20.0, 50.0),
            direction: RangeValue::Range(0.0, 360.0),
            spread: 0.0,
            start_size: RangeValue::Range(20.0, 40.0),
            end_size: RangeValue::Range(40.0, 80.0),
            rotation_speed: RangeValue::Single(0.0),
            color: "#FFFF0044".to_string(),
            shape: ParticleShape::Circle,
            physics: ParticlePhysics {
                gravity: 0.0,
                wind_x: 0.0,
                wind_y: 0.0,
                drag: 0.0,
            },
            blend_mode: BlendMode::Additive,
        };
        ParticleEmitter::new(config, bounds.spawn_center(), seed)
    }
}

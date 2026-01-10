use crate::particle::{
    BlendMode, ParticleConfig, ParticleEmitter, ParticlePhysics, ParticleShape, RangeValue,
    SpawnPattern,
};
use super::types::CharBounds;
use super::traits::ParticlePreset;

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

//! Particle configuration and spawn patterns

use serde::{Deserialize, Serialize};

use super::physics::ParticlePhysics;
use super::rng::Rng;
use super::types::{BlendMode, ParticleShape};

/// Range of values for randomization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RangeValue {
    Single(f32),
    Range(f32, f32),
}

impl RangeValue {
    pub fn sample(&self, rng: &mut Rng) -> f32 {
        match self {
            RangeValue::Single(v) => *v,
            RangeValue::Range(min, max) => rng.range(*min, *max),
        }
    }
}

impl Default for RangeValue {
    fn default() -> Self {
        RangeValue::Single(1.0)
    }
}

/// Configuration for particle emission
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticleConfig {
    /// Number of particles per emission burst
    #[serde(default = "default_count")]
    pub count: u32,

    /// Emissions per second (0 = single burst)
    #[serde(default)]
    pub spawn_rate: f32,

    /// Particle lifetime range in seconds
    #[serde(default = "default_lifetime")]
    pub lifetime: RangeValue,

    /// Initial speed range (pixels/second)
    #[serde(default = "default_speed")]
    pub speed: RangeValue,

    /// Direction range in degrees (0 = right, 90 = down)
    #[serde(default = "default_direction")]
    pub direction: RangeValue,

    /// Direction spread in degrees (cone angle)
    #[serde(default)]
    pub spread: f32,

    /// Start size range in pixels
    #[serde(default = "default_size")]
    pub start_size: RangeValue,

    /// End size range in pixels (for size-over-lifetime)
    #[serde(default = "default_size")]
    pub end_size: RangeValue,

    /// Rotation speed range (degrees/second)
    #[serde(default)]
    pub rotation_speed: RangeValue,

    /// Particle color (hex string)
    #[serde(default = "default_color")]
    pub color: String,

    /// Particle shape
    #[serde(default)]
    pub shape: ParticleShape,

    /// Physics parameters
    #[serde(default)]
    pub physics: ParticlePhysics,

    /// Blend mode for rendering
    #[serde(default)]
    pub blend_mode: BlendMode,
}

fn default_count() -> u32 {
    10
}
fn default_lifetime() -> RangeValue {
    RangeValue::Range(0.5, 1.5)
}
fn default_speed() -> RangeValue {
    RangeValue::Range(50.0, 150.0)
}
fn default_direction() -> RangeValue {
    RangeValue::Single(270.0)
}
fn default_size() -> RangeValue {
    RangeValue::Range(4.0, 8.0)
}
fn default_color() -> String {
    "#FFFFFF".to_string()
}

impl Default for ParticleConfig {
    fn default() -> Self {
        Self {
            count: 10,
            spawn_rate: 0.0,
            lifetime: default_lifetime(),
            speed: default_speed(),
            direction: default_direction(),
            spread: 30.0,
            start_size: default_size(),
            end_size: RangeValue::Single(0.0),
            rotation_speed: RangeValue::Single(0.0),
            color: default_color(),
            shape: ParticleShape::Circle,
            physics: ParticlePhysics::default(),
            blend_mode: BlendMode::Normal,
        }
    }
}

/// Spawn pattern determining where particles originate
#[derive(Debug, Clone)]
pub enum SpawnPattern {
    /// Single point
    Point { x: f32, y: f32 },
    /// Line segment
    Line { x1: f32, y1: f32, x2: f32, y2: f32 },
    /// Rectangle area
    Rect { x: f32, y: f32, w: f32, h: f32 },
}

impl SpawnPattern {
    pub fn sample(&self, rng: &mut Rng) -> (f32, f32) {
        match self {
            SpawnPattern::Point { x, y } => (*x, *y),
            SpawnPattern::Line { x1, y1, x2, y2 } => {
                let t = rng.next_f32();
                (x1 + (x2 - x1) * t, y1 + (y2 - y1) * t)
            }
            SpawnPattern::Rect { x, y, w, h } => (x + rng.next_f32() * w, y + rng.next_f32() * h),
        }
    }
}

/// Apply expression overrides to particle configuration
pub fn apply_particle_overrides(
    config: &mut ParticleConfig,
    overrides: &std::collections::HashMap<String, String>,
    ctx: &crate::expressions::EvaluationContext,
) {
    use crate::expressions::ExpressionEvaluator;

    for (key, expr_str) in overrides {
        if let Ok(val) = ExpressionEvaluator::evaluate(expr_str, ctx) {
            match key.as_str() {
                "count" => config.count = val as u32,
                "spawn_rate" | "rate" => config.spawn_rate = val as f32,
                "spread" => config.spread = val as f32,

                // Ranges (set as Single or specific min/max)
                "speed" => config.speed = RangeValue::Single(val as f32),
                "speed.min" => match &mut config.speed {
                    RangeValue::Range(min, _) | RangeValue::Single(min) => *min = val as f32,
                },
                "speed.max" => {
                    if let RangeValue::Range(_, max) = &mut config.speed {
                        *max = val as f32;
                    } else {
                        config.speed = RangeValue::Range(val as f32, val as f32);
                    }
                }

                "lifetime" => config.lifetime = RangeValue::Single(val as f32),
                "direction" => config.direction = RangeValue::Single(val as f32),
                "size" | "start_size" => config.start_size = RangeValue::Single(val as f32),
                "end_size" => config.end_size = RangeValue::Single(val as f32),
                "rotation_speed" => config.rotation_speed = RangeValue::Single(val as f32),

                // Physics
                "gravity" => config.physics.gravity = val as f32,
                "drag" => config.physics.drag = val as f32,
                "wind_x" => config.physics.wind_x = val as f32,
                "wind_y" => config.physics.wind_y = val as f32,

                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_pattern_point() {
        let pattern = SpawnPattern::Point { x: 100.0, y: 200.0 };
        let mut rng = Rng::new(42);
        let (x, y) = pattern.sample(&mut rng);
        assert_eq!((x, y), (100.0, 200.0));
    }

    #[test]
    fn test_spawn_pattern_line() {
        let pattern = SpawnPattern::Line {
            x1: 0.0,
            y1: 0.0,
            x2: 100.0,
            y2: 0.0,
        };
        let mut rng = Rng::new(42);
        for _ in 0..10 {
            let (x, y) = pattern.sample(&mut rng);
            assert!((0.0..=100.0).contains(&x));
            assert_eq!(y, 0.0);
        }
    }

    #[test]
    fn test_range_value_sample() {
        let mut rng = Rng::new(42);

        let single = RangeValue::Single(5.0);
        assert_eq!(single.sample(&mut rng), 5.0);

        let range = RangeValue::Range(0.0, 100.0);
        let val = range.sample(&mut rng);
        assert!((0.0..=100.0).contains(&val));
    }
}

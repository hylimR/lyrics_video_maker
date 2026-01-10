//! Physics simulation for particles

use serde::{Deserialize, Serialize};

/// Physics parameters for particle simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticlePhysics {
    /// Gravity acceleration (pixels/s², positive = down)
    #[serde(default = "default_gravity")]
    pub gravity: f32,
    /// Wind acceleration X (pixels/s²)
    #[serde(default)]
    pub wind_x: f32,
    /// Wind acceleration Y (pixels/s²)
    #[serde(default)]
    pub wind_y: f32,
    /// Drag coefficient [0, 1] (velocity reduction per second)
    #[serde(default)]
    pub drag: f32,
}

fn default_gravity() -> f32 {
    200.0
}

impl Default for ParticlePhysics {
    fn default() -> Self {
        Self {
            gravity: 200.0,
            wind_x: 0.0,
            wind_y: 0.0,
            drag: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_default() {
        let physics = ParticlePhysics::default();
        assert_eq!(physics.gravity, 200.0);
        assert_eq!(physics.wind_x, 0.0);
        assert_eq!(physics.drag, 0.0);
    }
}

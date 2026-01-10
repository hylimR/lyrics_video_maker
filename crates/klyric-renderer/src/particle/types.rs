//! Core particle types and structures

use serde::{Deserialize, Serialize};

use super::physics::ParticlePhysics;

/// A single particle instance
#[derive(Debug, Clone)]
pub struct Particle {
    /// Position X
    pub x: f32,
    /// Position Y
    pub y: f32,
    /// Velocity X (pixels per second)
    pub vx: f32,
    /// Velocity Y (pixels per second)
    pub vy: f32,
    /// Current lifetime (seconds since spawn)
    pub life: f32,
    /// Maximum lifetime before death
    pub max_life: f32,
    /// Current size in pixels
    pub size: f32,
    /// Initial size (for interpolation)
    pub start_size: f32,
    /// End size (for interpolation)
    pub end_size: f32,
    /// Rotation in degrees
    pub rotation: f32,
    /// Angular velocity (degrees per second)
    pub rotation_speed: f32,
    /// RGBA color as u32 (0xRRGGBBAA)
    pub color: u32,
    /// Current opacity [0, 1]
    pub opacity: f32,
    /// Shape type for rendering
    pub shape: ParticleShape,
}

impl Particle {
    /// Check if particle is still alive
    pub fn is_alive(&self) -> bool {
        self.life < self.max_life
    }

    /// Get normalized progress [0, 1] through lifetime
    pub fn progress(&self) -> f32 {
        (self.life / self.max_life).clamp(0.0, 1.0)
    }

    /// Update particle state by delta time
    pub fn update(&mut self, dt: f32, physics: &ParticlePhysics) {
        // Apply gravity
        self.vy += physics.gravity * dt;

        // Apply drag
        self.vx *= 1.0 - physics.drag * dt;
        self.vy *= 1.0 - physics.drag * dt;

        // Apply wind
        self.vx += physics.wind_x * dt;
        self.vy += physics.wind_y * dt;

        // Update position
        self.x += self.vx * dt;
        self.y += self.vy * dt;

        // Update rotation
        self.rotation += self.rotation_speed * dt;

        // Update lifetime
        self.life += dt;

        // Interpolate size
        let p = self.progress();
        self.size = self.start_size + (self.end_size - self.start_size) * p;

        // Fade out near end of life (last 30%)
        if p > 0.7 {
            self.opacity = 1.0 - (p - 0.7) / 0.3;
        }
    }
}

/// Shape of a particle for rendering
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ParticleShape {
    #[default]
    Circle,
    Square,
    /// Single character (e.g., "♥", "★", "|")
    Char(String),
    /// Path to image asset
    Image(String),
}

/// Blend mode for particle rendering
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum BlendMode {
    #[default]
    Normal,
    Additive,
    Multiply,
}

/// Extract RGBA components from u32 color
pub fn color_to_rgba(color: u32) -> (u8, u8, u8, u8) {
    (
        ((color >> 24) & 0xFF) as u8,
        ((color >> 16) & 0xFF) as u8,
        ((color >> 8) & 0xFF) as u8,
        (color & 0xFF) as u8,
    )
}

/// Parse hex color string to RGBA u32
pub fn parse_hex_color(hex: &str) -> u32 {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
        (r as u32) << 24 | (g as u32) << 16 | (b as u32) << 8 | 255
    } else if hex.len() == 8 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
        let a = u8::from_str_radix(&hex[6..8], 16).unwrap_or(255);
        (r as u32) << 24 | (g as u32) << 16 | (b as u32) << 8 | a as u32
    } else {
        0xFFFFFFFF // Default white
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_parsing() {
        assert_eq!(parse_hex_color("#FF0000"), 0xFF0000FF);
        assert_eq!(parse_hex_color("#00FF00"), 0x00FF00FF);
        assert_eq!(parse_hex_color("#0000FF80"), 0x0000FF80);
    }

    #[test]
    fn test_color_to_rgba() {
        let (r, g, b, a) = color_to_rgba(0xFF00FF80);
        assert_eq!((r, g, b, a), (255, 0, 255, 128));
    }
}

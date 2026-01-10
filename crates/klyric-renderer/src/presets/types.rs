use crate::particle::SpawnPattern;

/// Effect preset identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectPreset {
    Rain,
    Sparkle,
    Hearts,
    Confetti,
    Disintegrate,
    Fire,
    GlowPulse,
}

impl EffectPreset {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "rain" => Some(Self::Rain),
            "sparkle" => Some(Self::Sparkle),
            "hearts" => Some(Self::Hearts),
            "confetti" => Some(Self::Confetti),
            "disintegrate" => Some(Self::Disintegrate),
            "fire" => Some(Self::Fire),
            "glow" | "glowpulse" | "glow_pulse" => Some(Self::GlowPulse),
            _ => None,
        }
    }
}

/// Character/syllable bounds for spawning particles
#[derive(Debug, Clone)]
pub struct CharBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl CharBounds {
    /// Create spawn pattern for particles above the character
    pub fn spawn_above(&self, offset: f32) -> SpawnPattern {
        SpawnPattern::Line {
            x1: self.x,
            y1: self.y - offset,
            x2: self.x + self.width,
            y2: self.y - offset,
        }
    }

    /// Create spawn pattern at character center
    pub fn spawn_center(&self) -> SpawnPattern {
        SpawnPattern::Point {
            x: self.x + self.width / 2.0,
            y: self.y + self.height / 2.0,
        }
    }

    /// Create spawn pattern filling character area
    pub fn spawn_fill(&self) -> SpawnPattern {
        SpawnPattern::Rect {
            x: self.x,
            y: self.y,
            w: self.width,
            h: self.height,
        }
    }
}

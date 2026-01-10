use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::particle::ParticleConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Effect {
    /// Effect type
    #[serde(rename = "type")]
    pub effect_type: EffectType,
    
    /// When the effect triggers
    #[serde(default)]
    pub trigger: EffectTrigger,
    
    /// Effect duration in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
    
    /// Delay before effect starts
    #[serde(default)]
    pub delay: f64,
    
    /// Easing function
    #[serde(default)]
    pub easing: Easing,
    
    // Transition-specific
    /// Properties to animate (for transition type)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, AnimatedValue>,
    
    // Karaoke-specific
    /// Karaoke mode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<KaraokeMode>,
    
    /// Effect direction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<Direction>,
    
    // Keyframe-specific
    /// Keyframe array (for keyframe type)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keyframes: Vec<Keyframe>,
    
    // Particle-specific
    /// Preset name for particle effect
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preset: Option<String>,
    
    /// Custom particle configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub particle_config: Option<ParticleConfig>,
    
    // Common
    /// Number of iterations
    #[serde(default = "default_iterations")]
    pub iterations: u32,
}

fn default_iterations() -> u32 { 1 }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EffectType {
    Transition,
    Karaoke,
    Keyframe,
    Particle,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum EffectTrigger {
    #[default]
    Enter,
    Exit,
    Active,
    Inactive,
    Always,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KaraokeMode {
    Mask,
    Color,
    Wipe,
    Reveal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Ltr,
    Rtl,
    Ttb,
    Btt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimatedValue {
    /// Starting value
    pub from: f64,
    /// Ending value
    pub to: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Keyframe {
    /// Position in animation (0-1)
    pub time: f64,
    
    /// Opacity at this keyframe
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f32>,
    
    /// Scale at this keyframe
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<f32>,
    
    /// Horizontal scale
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale_x: Option<f32>,
    
    /// Vertical scale
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale_y: Option<f32>,
    
    /// Rotation in degrees
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<f32>,
    
    /// X offset in pixels
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<f32>,
    
    /// Y offset in pixels
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<f32>,
    
    /// Color at this keyframe
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    
    /// Easing to next keyframe
    #[serde(skip_serializing_if = "Option::is_none")]
    pub easing: Option<Easing>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum Easing {
    #[default]
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,
    EaseInQuint,
    EaseOutQuint,
    EaseInOutQuint,
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,
    EaseInExpo,
    EaseOutExpo,
    EaseInOutExpo,
    EaseInCirc,
    EaseOutCirc,
    EaseInOutCirc,
    EaseInElastic,
    EaseOutElastic,
    EaseInOutElastic,
    EaseInBack,
    EaseOutBack,
    EaseInOutBack,
    EaseInBounce,
    EaseOutBounce,
    EaseInOutBounce,
}

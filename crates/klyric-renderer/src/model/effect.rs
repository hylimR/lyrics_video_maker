use crate::particle::ParticleConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

    /// Dynamic particle overrides (property -> expression)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub particle_override: Option<HashMap<String, String>>,

    // Common
    /// Number of iterations
    #[serde(default = "default_iterations")]
    pub iterations: u32,
}

fn default_iterations() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum EffectType {
    #[default]
    Transition,
    Karaoke,
    Keyframe,
    Particle,
    Disintegrate,
    Typewriter,
    StrokeReveal,
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
#[serde(untagged)]
pub enum AnimatedValue {
    Range { from: f64, to: f64 },
    Expression(String),
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

    /// Blur sigma
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blur: Option<f32>,

    /// Glitch offset (pixels)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub glitch_offset: Option<f32>,

    /// Color at this keyframe
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,

    /// Easing to next keyframe
    #[serde(skip_serializing_if = "Option::is_none")]
    pub easing: Option<Easing>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_deserialization_defaults() {
        let json = r#"{
            "type": "transition"
        }"#;

        let effect: Effect = serde_json::from_str(json).unwrap();
        assert_eq!(effect.effect_type, EffectType::Transition);
        assert_eq!(effect.trigger, EffectTrigger::Enter);
        assert_eq!(effect.iterations, 1);
        assert_eq!(effect.delay, 0.0);
        assert_eq!(effect.easing, Easing::Linear);
        assert!(effect.duration.is_none());
    }

    #[test]
    fn test_animated_value_deserialization() {
        let json_range = r#"{"from": 0.0, "to": 1.0}"#;
        let value: AnimatedValue = serde_json::from_str(json_range).unwrap();
        match value {
            AnimatedValue::Range { from, to } => {
                assert_eq!(from, 0.0);
                assert_eq!(to, 1.0);
            }
            _ => panic!("Expected Range"),
        }

        let json_expr = r#""progress * 100""#;
        let value: AnimatedValue = serde_json::from_str(json_expr).unwrap();
        match value {
            AnimatedValue::Expression(expr) => {
                assert_eq!(expr, "progress * 100");
            }
            _ => panic!("Expected Expression"),
        }
    }

    #[test]
    fn test_keyframe_deserialization() {
        let json = r#"{
            "time": 0.5,
            "opacity": 0.8,
            "color": "#FF0000",
            "easing": "easeInOutSine"
        }"#;

        let kf: Keyframe = serde_json::from_str(json).unwrap();
        assert_eq!(kf.time, 0.5);
        assert_eq!(kf.opacity, Some(0.8));
        assert_eq!(kf.color.as_deref(), Some("#FF0000"));
        assert_eq!(kf.easing, Some(Easing::EaseInOutSine));
        assert!(kf.scale.is_none());
    }

    #[test]
    fn test_effect_enums_deserialization() {
        let json_karaoke_mode = r#""wipe""#;
        let mode: KaraokeMode = serde_json::from_str(json_karaoke_mode).unwrap();
        match mode {
            KaraokeMode::Wipe => {}
            _ => panic!("Expected Wipe"),
        }

        let json_direction = r#""ttb""#;
        let dir: Direction = serde_json::from_str(json_direction).unwrap();
        match dir {
            Direction::Ttb => {}
            _ => panic!("Expected Ttb"),
        }
    }
}

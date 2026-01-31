use crate::model::{Easing, Effect, EffectTrigger, EffectType, Keyframe};
use std::collections::HashMap;

/// Creates a Blur Dissolve transition
pub fn blur_dissolve(duration: f64) -> Effect {
    // Simulating "Zoom Blur" appearance
    Effect {
        effect_type: EffectType::Keyframe,
        trigger: EffectTrigger::Enter,
        duration: Some(duration),
        delay: 0.0,
        easing: Easing::EaseOutCubic,
        properties: HashMap::new(),
        mode: None,
        direction: None,
        keyframes: vec![
            Keyframe {
                time: 0.0,
                opacity: Some(0.0),
                scale: Some(1.5),
                blur: Some(10.0), // Start with strong blur
                ..Default::default()
            },
            Keyframe {
                time: 1.0,
                opacity: Some(1.0),
                scale: Some(1.0),
                blur: Some(0.0), // End with no blur
                ..Default::default()
            },
        ],
        preset: None,
        particle_config: None,
        iterations: 1,
        particle_override: None,
    }
}

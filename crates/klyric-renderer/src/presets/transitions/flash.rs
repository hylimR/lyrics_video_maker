use crate::model::{Easing, Effect, EffectTrigger, EffectType, Keyframe};
use std::collections::HashMap;

/// Creates a Dip to Color (Fade to Black/Color) transition
pub fn flash_in(duration: f64) -> Effect {
    Effect {
        effect_type: EffectType::Keyframe,
        trigger: EffectTrigger::Enter,
        duration: Some(duration),
        delay: 0.0,
        easing: Easing::EaseOutExpo,
        properties: HashMap::new(),
        mode: None,
        direction: None,
        keyframes: vec![
            Keyframe {
                time: 0.0,
                opacity: Some(0.0),
                scale: Some(2.0),
                ..Default::default()
            },
            Keyframe {
                time: 1.0,
                opacity: Some(1.0),
                scale: Some(1.0),
                ..Default::default()
            },
        ],
        preset: None,
        particle_config: None,
        iterations: 1,
        particle_override: None,
    }
}

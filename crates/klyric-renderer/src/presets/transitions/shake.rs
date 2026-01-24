use crate::model::{Effect, EffectType, EffectTrigger, Easing, Keyframe};
use std::collections::HashMap;

/// Creates a Shake transition
pub fn shake(duration: f64) -> Effect {
    Effect {
        effect_type: EffectType::Keyframe,
        trigger: EffectTrigger::Enter,
        duration: Some(duration),
        delay: 0.0,
        easing: Easing::Linear,
        properties: HashMap::new(),
        mode: None,
        direction: None,
        keyframes: vec![
            Keyframe { time: 0.0, opacity: Some(0.0), x: Some(0.0), rotation: Some(0.0), ..Default::default() },
            Keyframe { time: 0.1, opacity: Some(1.0), x: Some(-5.0), rotation: Some(-2.0), ..Default::default() },
            Keyframe { time: 0.3, x: Some(5.0), rotation: Some(2.0), ..Default::default() },
            Keyframe { time: 0.5, x: Some(-5.0), rotation: Some(-2.0), ..Default::default() },
            Keyframe { time: 0.7, x: Some(5.0), rotation: Some(2.0), ..Default::default() },
            Keyframe { time: 0.9, x: Some(-2.0), rotation: Some(-1.0), ..Default::default() },
            Keyframe { time: 1.0, x: Some(0.0), rotation: Some(0.0), ..Default::default() },
        ],
        preset: None,
        particle_config: None,
        iterations: 1,
        particle_override: None,
    }
}

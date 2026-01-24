use crate::model::{Effect, EffectType, EffectTrigger, Easing, Keyframe};
use std::collections::HashMap;

/// Creates a Glitch transition
pub fn glitch(duration: f64) -> Effect {
    // Random jittery movement and scaling
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
            Keyframe { time: 0.0, opacity: Some(0.0), x: Some(10.0), scale_x: Some(1.2), glitch_offset: Some(5.0), ..Default::default() },
            Keyframe { time: 0.2, opacity: Some(0.5), x: Some(-10.0), scale_x: Some(0.8), glitch_offset: Some(-5.0), ..Default::default() },
            Keyframe { time: 0.4, opacity: Some(0.8), x: Some(5.0), scale_y: Some(1.2), glitch_offset: Some(10.0), ..Default::default() },
            Keyframe { time: 0.6, opacity: Some(1.0), x: Some(-5.0), glitch_offset: Some(-2.0), ..Default::default() },
            Keyframe { time: 0.8, x: Some(2.0), glitch_offset: Some(2.0), ..Default::default() },
            Keyframe { time: 1.0, x: Some(0.0), scale_x: Some(1.0), scale_y: Some(1.0), glitch_offset: Some(0.0), ..Default::default() },
        ],
        preset: None,
        particle_config: None,
        iterations: 1,
        particle_override: None,
    }
}

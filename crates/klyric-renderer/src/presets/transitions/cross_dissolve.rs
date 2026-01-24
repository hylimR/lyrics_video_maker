use crate::model::{Effect, EffectType, EffectTrigger, AnimatedValue, Easing};
use std::collections::HashMap;

/// Creates a standard Cross Dissolve (Fade In/Out) transition
pub fn cross_dissolve(duration: f64) -> Effect {
    let mut properties = HashMap::new();
    properties.insert("opacity".to_string(), AnimatedValue::Range { from: 0.0, to: 1.0 });

    Effect {
        effect_type: EffectType::Transition,
        trigger: EffectTrigger::Enter,
        duration: Some(duration),
        delay: 0.0,
        easing: Easing::Linear,
        properties,
        mode: None,
        direction: None,
        keyframes: vec![],
        preset: None,
        particle_config: None,
        iterations: 1,
        particle_override: None,
    }
}

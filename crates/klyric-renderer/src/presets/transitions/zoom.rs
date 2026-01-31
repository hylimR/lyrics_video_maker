use crate::model::{AnimatedValue, Easing, Effect, EffectTrigger, EffectType};
use std::collections::HashMap;

/// Creates a Zoom (Crash Zoom) transition
/// Mode: In or Out
pub fn zoom(duration: f64, zoom_in: bool) -> Effect {
    let start_scale = if zoom_in { 0.0 } else { 3.0 };

    let mut properties = HashMap::new();
    properties.insert(
        "scale".to_string(),
        AnimatedValue::Range {
            from: start_scale,
            to: 1.0,
        },
    );
    properties.insert(
        "opacity".to_string(),
        AnimatedValue::Range { from: 0.0, to: 1.0 },
    );

    Effect {
        effect_type: EffectType::Transition,
        trigger: EffectTrigger::Enter,
        duration: Some(duration),
        delay: 0.0,
        easing: if zoom_in {
            Easing::EaseOutBack
        } else {
            Easing::EaseOutExpo
        },
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

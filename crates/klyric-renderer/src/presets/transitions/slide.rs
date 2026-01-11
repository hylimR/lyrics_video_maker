use crate::model::{Effect, EffectType, EffectTrigger, AnimatedValue, Easing, Direction};
use std::collections::HashMap;

/// Creates a Slide (Push) transition
pub fn slide(duration: f64, direction: Direction) -> Effect {
    let (start_x, start_y) = match direction {
        Direction::Ltr => (-100.0, 0.0), // From Left
        Direction::Rtl => (100.0, 0.0),  // From Right
        Direction::Ttb => (0.0, -50.0),  // From Top
        Direction::Btt => (0.0, 50.0),   // From Bottom
    };

    let mut properties = HashMap::new();
    properties.insert("x".to_string(), AnimatedValue { from: start_x, to: 0.0 });
    properties.insert("y".to_string(), AnimatedValue { from: start_y, to: 0.0 });
    properties.insert("opacity".to_string(), AnimatedValue { from: 0.0, to: 1.0 });

    Effect {
        effect_type: EffectType::Transition,
        trigger: EffectTrigger::Enter,
        duration: Some(duration),
        delay: 0.0,
        easing: Easing::EaseOutQuart,
        properties,
        mode: None,
        direction: Some(direction),
        keyframes: vec![],
        preset: None,
        particle_config: None,
        iterations: 1,
    }
}

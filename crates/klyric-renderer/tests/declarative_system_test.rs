use klyric_renderer::effects::{EffectEngine, TriggerContext};
use klyric_renderer::expressions::EvaluationContext;
use klyric_renderer::model::{AnimatedValue, Easing, Effect, EffectTrigger, EffectType, Transform};
use klyric_renderer::particle::config::{apply_particle_overrides, ParticleConfig};
use std::collections::HashMap;

#[test]
fn test_expression_transform() {
    // 1. Setup an effect with expression-based property
    let mut properties = HashMap::new();
    // y = t * 100
    properties.insert(
        "y".to_string(),
        AnimatedValue::Expression("t * 100.0".to_string()),
    );

    let effect = Effect {
        effect_type: EffectType::Transition,
        trigger: EffectTrigger::Active,
        duration: Some(1.0),
        delay: 0.0,
        easing: Easing::Linear,
        properties,
        mode: None,
        direction: None,
        keyframes: Vec::new(),
        preset: None,
        particle_config: None,
        iterations: 1,
        particle_override: None,
    };

    let ctx = TriggerContext {
        start_time: 0.0,
        end_time: 2.0,
        current_time: 0.5, // t = 0.5 (relative to start?)
        // Note: EvaluationContext usually uses 't' as relative time or absolute?
        // EffectEngine::compute_transform typically passes relative time if it calculates progress?
        // Let's check EffectEngine implementation. Usually it passes absolute 'time' as 't' and 'progress' as 0-1.
        active: true,
        char_index: Some(0),
        char_count: Some(5),
    };

    // We need to match how EffectEngine constructs EvaluationContext.
    // In effects.rs, it does:
    // let eval_ctx = EvaluationContext { t: time, progress: eased_progress, ... }

    let base = Transform::default();
    let effects = vec![effect.clone()];
    let final_transform = EffectEngine::compute_transform(0.5, base, &effects, ctx);

    // At t=0.5, y should be 50.0
    assert_eq!(final_transform.y_val(), 50.0);
}

#[test]
fn test_typewriter_opacity() {
    let effect = Effect {
        effect_type: EffectType::Typewriter,
        trigger: EffectTrigger::Active,
        duration: Some(1.0), // 1 second total
        delay: 0.0,
        easing: Easing::Linear,
        properties: HashMap::new(),
        mode: None,
        direction: None,
        keyframes: Vec::new(),
        preset: None,
        particle_config: None,
        iterations: 1,
        particle_override: None,
    };

    // Char 0 of 10. At t=0.05, progress=0.05. 0.05 * 10 = 0.5.
    // Visible limit = 0.5. Char 0 is visible?
    // Logic in effects.rs:
    // let limit = eased_progress * count as f64;
    // if index as f64 <= limit { 1.0 } else { 0.0 }

    // Case 1: Early in animation
    let ctx_start = TriggerContext {
        start_time: 0.0,
        end_time: 1.0,
        current_time: 0.05,
        active: true,
        char_index: Some(0),
        char_count: Some(10),
    };

    let base = Transform::default();
    let effects_start = vec![effect.clone()];
    let result = EffectEngine::compute_transform(0.05, base.clone(), &effects_start, ctx_start);
    // limit = 0.5. index 0 <= 0.5? Yes. Opacity 1.0?
    // Wait, usually it's floor? Or strictly less?
    // If limit is 0.5, half of char 0 is shown? No, opacity is 0 or 1 usually for typewriter.
    // Let's assume standard logic provided in effects.rs:
    // if idx < limit.ceil() ?
    // We'll see what the test says.
    assert_eq!(result.opacity_val(), 1.0);

    // Case 2: Last char at 50% progress
    let ctx_mid = TriggerContext {
        start_time: 0.0,
        end_time: 1.0,
        current_time: 0.5,
        active: true,
        char_index: Some(9), // Last char
        char_count: Some(10),
    };
    let effects_vec = vec![effect.clone()];
    let result_mid = EffectEngine::compute_transform(0.5, base, &effects_vec, ctx_mid);
    // Limit = 5.0. Index 9 > 5.0. Should be hidden.
    assert_eq!(result_mid.opacity_val(), 0.0);
}

#[test]
fn test_particle_overrides() {
    let mut config = ParticleConfig {
        count: 10,
        ..Default::default()
    };

    let mut overrides = HashMap::new();
    overrides.insert("count".to_string(), "index * 10".to_string()); // dynamic count
    overrides.insert("speed".to_string(), "100.0".to_string()); // static expression

    let ctx = EvaluationContext {
        t: 1.0,
        progress: 0.5,
        index: Some(5),
        count: Some(10),
        ..Default::default()
    };

    apply_particle_overrides(&mut config, &overrides, &ctx);

    assert_eq!(config.count, 50); // 5 * 10

    // Check range value
    use klyric_renderer::particle::RangeValue;
    if let RangeValue::Single(v) = config.speed {
        assert_eq!(v, 100.0);
    } else {
        panic!("Speed should be Single");
    }
}

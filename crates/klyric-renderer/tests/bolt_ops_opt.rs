use klyric_renderer::effects::{
    CompiledRenderOp, EffectEngine, RenderProperty, RenderValueOp, ResolvedEffect, TriggerContext,
};
use klyric_renderer::expressions::{ExpressionEvaluator, FastEvaluationContext};
use klyric_renderer::model::{
    AnimatedValue, Easing, Effect, EffectTrigger, EffectType, RenderTransform,
};
use std::collections::HashMap;
use std::sync::Arc;

// Helper to wrap effect in ResolvedEffect
fn resolve(effect: Effect) -> ResolvedEffect {
    let mut map = HashMap::new();
    for (_k, v) in &effect.properties {
        if let AnimatedValue::Expression(e) = v {
            if let Ok(node) = ExpressionEvaluator::compile(e) {
                map.insert(e.clone(), Arc::new(node));
            }
        }
    }
    ResolvedEffect {
        effect,
        compiled_expressions: map,
        name_hash: 0,
    }
}

#[test]
fn test_mixed_constant_and_dynamic_ops_optimization() {
    // Scenario: Constant followed by Dynamic on SAME property.
    // Expected: Constant should be hoisted and REMOVED from ops. Dynamic should remain.
    // Resulting ops length: 1.

    // To simulate multiple effects, we create two effects.
    let effect1 = Effect {
        effect_type: EffectType::Transition,
        trigger: EffectTrigger::Enter,
        duration: Some(1.0),
        delay: 0.0,
        easing: Easing::Linear,
        properties: HashMap::from([(
            "opacity".to_string(),
            AnimatedValue::Range { from: 0.0, to: 1.0 },
        )]),
        mode: None,
        direction: None,
        keyframes: Vec::new(),
        preset: None,
        particle_config: None,
        iterations: 1,
        particle_override: None,
    };

    let effect2 = Effect {
        effect_type: EffectType::Transition,
        trigger: EffectTrigger::Enter,
        duration: Some(1.0),
        delay: 0.0,
        easing: Easing::Linear,
        properties: HashMap::from([(
            "opacity".to_string(),
            AnimatedValue::Expression("sin(t)".to_string()),
        )]),
        mode: None,
        direction: None,
        keyframes: Vec::new(),
        preset: None,
        particle_config: None,
        iterations: 1,
        particle_override: None,
    };

    let resolved = vec![resolve(effect1), resolve(effect2)];
    let active_indices = vec![(0, 0.5), (1, 0.5)]; // Both active
    let ctx = TriggerContext::default();

    let mut ops = Vec::new();
    let mut hoisted = RenderTransform::default();
    let mut mask = 0;

    EffectEngine::compile_active_effects(
        &resolved,
        &active_indices,
        &ctx,
        &mut ops,
        &mut hoisted,
        &mut mask,
    );

    // With optimization:
    // Constant comes first, so it updates hoisted transform and is skipped from ops (dynamic_seen not set).
    // Dynamic comes second, so it is added to ops.
    // Result: ops.len() == 1
    assert_eq!(
        ops.len(),
        1,
        "Optimized: Should contain only Dynamic op"
    );

    // Verify content
    // Op 0: Expression
    match &ops[0].value {
        RenderValueOp::Expression(_, _) => {}
        _ => panic!("Expected Expression only"),
    }
}

#[test]
fn test_dynamic_then_constant_optimization() {
    // Scenario: Dynamic followed by Constant on SAME property.
    // Expected: Constant MUST remain in ops to override Dynamic.
    // Resulting ops length: 2.

    let effect1 = Effect {
        effect_type: EffectType::Transition,
        trigger: EffectTrigger::Enter,
        duration: Some(1.0),
        delay: 0.0,
        easing: Easing::Linear,
        properties: HashMap::from([(
            "opacity".to_string(),
            AnimatedValue::Expression("sin(t)".to_string()),
        )]),
        mode: None,
        direction: None,
        keyframes: Vec::new(),
        preset: None,
        particle_config: None,
        iterations: 1,
        particle_override: None,
    };

    let effect2 = Effect {
        effect_type: EffectType::Transition,
        trigger: EffectTrigger::Enter,
        duration: Some(1.0),
        delay: 0.0,
        easing: Easing::Linear,
        properties: HashMap::from([(
            "opacity".to_string(),
            AnimatedValue::Range { from: 0.8, to: 0.8 },
        )]),
        mode: None,
        direction: None,
        keyframes: Vec::new(),
        preset: None,
        particle_config: None,
        iterations: 1,
        particle_override: None,
    };

    let resolved = vec![resolve(effect1), resolve(effect2)];
    let active_indices = vec![(0, 0.5), (1, 0.5)];
    let ctx = TriggerContext::default();

    let mut ops = Vec::new();
    let mut hoisted = RenderTransform::default();
    let mut mask = 0;

    EffectEngine::compile_active_effects(
        &resolved,
        &active_indices,
        &ctx,
        &mut ops,
        &mut hoisted,
        &mut mask,
    );

    assert_eq!(ops.len(), 2, "Should contain both ops");
}

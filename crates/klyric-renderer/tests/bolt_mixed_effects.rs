use klyric_renderer::effects::{
    CompiledRenderOp, EffectEngine, RenderProperty, RenderValueOp, ResolvedEffect, TriggerContext,
};
use klyric_renderer::expressions::{ExpressionEvaluator, FastEvaluationContext, EvaluationContext};
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
fn test_mixed_properties_logic() {
    // Scenario:
    // 1. Constant Opacity = 0.5
    // 2. Dynamic Scale = 2.0 (via Expression)

    let effect1 = Effect {
        effect_type: EffectType::Transition,
        trigger: EffectTrigger::Enter,
        duration: Some(1.0),
        delay: 0.0,
        easing: Easing::Linear,
        properties: HashMap::from([(
            "opacity".to_string(),
            AnimatedValue::Range { from: 0.5, to: 0.5 },
        )]),
        mode: None, direction: None, keyframes: Vec::new(), preset: None, particle_config: None, iterations: 1, particle_override: None,
    };

    let effect2 = Effect {
        effect_type: EffectType::Transition,
        trigger: EffectTrigger::Enter,
        duration: Some(1.0),
        delay: 0.0,
        easing: Easing::Linear,
        properties: HashMap::from([(
            "scale".to_string(),
            AnimatedValue::Expression("2.0".to_string()),
        )]),
        mode: None, direction: None, keyframes: Vec::new(), preset: None, particle_config: None, iterations: 1, particle_override: None,
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

    // Verify compile logic (Hoisting worked)
    assert_eq!(ops.len(), 1, "Should have 1 dynamic op");
    assert_eq!(ops[0].prop, RenderProperty::Scale);
    assert!((hoisted.opacity - 0.5).abs() < 0.001, "Hoisted should have opacity 0.5");

    // Simulate render_line logic (BUGGY VERSION)
    let mut final_transform_buggy = RenderTransform::default();
    let eval_ctx = EvaluationContext::default();
    let mut fast_ctx = FastEvaluationContext::new(&eval_ctx);

    if ops.is_empty() {
        if mask != 0 {
            final_transform_buggy.apply_mask(&hoisted, mask);
        }
    } else {
        // BUG: hoisted is NOT applied here
        final_transform_buggy = EffectEngine::apply_compiled_ops(final_transform_buggy, &ops, &mut fast_ctx);
    }

    // Verify buggy behavior (Opacity lost)
    // Scale applied?
    assert!((final_transform_buggy.scale - 2.0).abs() < 0.001);
    // Opacity lost? (Should be 1.0, not 0.5)
    assert!((final_transform_buggy.opacity - 1.0).abs() < 0.001, "Buggy version loses opacity");


    // Simulate render_line logic (FIXED VERSION)
    let mut final_transform_fixed = RenderTransform::default();

    if ops.is_empty() {
        if mask != 0 {
            final_transform_fixed.apply_mask(&hoisted, mask);
        }
    } else {
        // FIX: Apply hoisted mask first
        if mask != 0 {
            final_transform_fixed.apply_mask(&hoisted, mask);
        }
        final_transform_fixed = EffectEngine::apply_compiled_ops(final_transform_fixed, &ops, &mut fast_ctx);
    }

    // Verify fixed behavior
    assert!((final_transform_fixed.scale - 2.0).abs() < 0.001, "Scale should be 2.0");
    assert!((final_transform_fixed.opacity - 0.5).abs() < 0.001, "Opacity should be 0.5");
}

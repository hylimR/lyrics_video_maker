use super::model::{AnimatedValue, Easing, Effect, EffectType, RenderTransform, Transform};
use crate::expressions::{EvaluationContext, ExpressionEvaluator};
use evalexpr::Node;
use std::collections::HashMap;
use std::f64::consts::PI;
use std::sync::Arc;

#[derive(Clone)]
pub struct ResolvedEffect {
    pub effect: Effect,
    pub compiled_expressions: HashMap<String, Arc<Node>>,
    pub name_hash: u64,
}

impl std::borrow::Borrow<Effect> for ResolvedEffect {
    fn borrow(&self) -> &Effect {
        &self.effect
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderProperty {
    Opacity,
    Scale,
    ScaleX,
    ScaleY,
    X,
    Y,
    Rotation,
    Blur,
    GlitchOffset,
    AnchorX,
    AnchorY,
    HueShift,
}

impl RenderProperty {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "opacity" => Some(Self::Opacity),
            "scale" => Some(Self::Scale),
            "scale_x" => Some(Self::ScaleX),
            "scale_y" => Some(Self::ScaleY),
            "x" => Some(Self::X),
            "y" => Some(Self::Y),
            "rotation" => Some(Self::Rotation),
            "blur" => Some(Self::Blur),
            "glitch_offset" | "glitch" => Some(Self::GlitchOffset),
            "anchor_x" => Some(Self::AnchorX),
            "anchor_y" => Some(Self::AnchorY),
            "hue_shift" => Some(Self::HueShift),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum RenderValueOp {
    Constant(f32),
    Expression(Arc<Node>, f64), // Store (expression, progress)
    TypewriterLimit(f64),
}

pub struct CompiledRenderOp {
    pub prop: RenderProperty,
    pub value: RenderValueOp,
}

use crate::model::modifiers::{Modifier, EffectLayer, Selector, ScopeType, AppearMode};
use crate::effects::drivers::DriverManager;

pub mod drivers; // Added drivers module

/// Engine for applying effects and calculating animations
pub struct EffectEngine;

impl EffectEngine {
    /// Calculate the easing value for a given time t (0.0 to 1.0)
    pub fn ease(t: f64, easing: &Easing) -> f64 {
        match easing {
            Easing::Linear => t,

            // Quad
            Easing::EaseIn | Easing::EaseInQuad => t * t,
            Easing::EaseOut | Easing::EaseOutQuad => t * (2.0 - t),
            Easing::EaseInOut | Easing::EaseInOutQuad => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }

            // Cubic
            Easing::EaseInCubic => t * t * t,
            Easing::EaseOutCubic => {
                let t = t - 1.0;
                t * t * t + 1.0
            }
            Easing::EaseInOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    (t - 1.0) * (2.0 * t - 2.0).powi(2) + 1.0
                }
            }

            // Sine
            Easing::EaseInSine => 1.0 - (t * PI / 2.0).cos(),
            Easing::EaseOutSine => (t * PI / 2.0).sin(),
            Easing::EaseInOutSine => -(text_cos(PI * t) - 1.0) / 2.0,

            // Exponential
            Easing::EaseInExpo => {
                if t == 0.0 {
                    0.0
                } else {
                    2.0f64.powf(10.0 * (t - 1.0))
                }
            }
            Easing::EaseOutExpo => {
                if t == 1.0 {
                    1.0
                } else {
                    1.0 - 2.0f64.powf(-10.0 * t)
                }
            }

            // Elastic
            Easing::EaseOutElastic => {
                let c4 = (2.0 * PI) / 3.0;
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    2.0f64.powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0
                }
            }

            // Bounce
            Easing::EaseOutBounce => {
                let n1 = 7.5625;
                let d1 = 2.75;
                if t < 1.0 / d1 {
                    n1 * t * t
                } else if t < 2.0 / d1 {
                    let t = t - 1.5 / d1;
                    n1 * t * t + 0.75
                } else if t < 2.5 / d1 {
                    let t = t - 2.25 / d1;
                    n1 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / d1;
                    n1 * t * t + 0.984375
                }
            }

            // Fallback for others
            _ => t,
        }
    }

    /// Interpolate between two values
    pub fn lerp(start: f64, end: f64, t: f64) -> f64 {
        start + (end - start) * t
    }

    /// Compile active effects into a list of optimized render operations.
    /// This pre-calculates any values that are constant for the current frame (e.g. lerped ranges, keyframe values).
    pub fn compile_active_effects(
        effects_source: &[ResolvedEffect],
        active_indices: &[(usize, f64)],
        ctx: &TriggerContext,
        ops: &mut Vec<CompiledRenderOp>
    ) {
        ops.clear();
        ops.reserve(active_indices.len() * 2);

        for (idx, eased_progress) in active_indices {
            let resolved_effect = &effects_source[*idx];
            let effect = &resolved_effect.effect;
            match effect.effect_type {
                EffectType::Transition => {
                    for (prop_name, value) in &effect.properties {
                        if let Some(prop) = RenderProperty::from_str(prop_name) {
                            match value {
                                AnimatedValue::Range { from, to } => {
                                    // Pre-calculate lerp once per frame
                                    let val = Self::lerp(*from, *to, *eased_progress);
                                    ops.push(CompiledRenderOp {
                                        prop,
                                        value: RenderValueOp::Constant(val as f32),
                                    });
                                }
                                AnimatedValue::Expression(expr_str) => {
                                    // Look up compiled expression
                                    if let Some(node) =
                                        resolved_effect.compiled_expressions.get(expr_str)
                                    {
                                        ops.push(CompiledRenderOp {
                                            prop,
                                            value: RenderValueOp::Expression(node.clone(), *eased_progress),
                                        });
                                    } else {
                                        // Fallback if not compiled? Should ideally not happen if setup correctly.
                                        // Since we can't easily parse here without returning error or allocating,
                                        // we just skip or log.
                                        log::trace!("Skipping uncompiled expression: {}", expr_str);
                                    }
                                }
                            }
                        }
                    }
                }
                EffectType::Typewriter => {
                    // Pre-calculate visible limit logic
                    let total_chars = ctx.char_count.unwrap_or(1) as f64;
                    let visible_limit = *eased_progress * total_chars;
                    ops.push(CompiledRenderOp {
                        prop: RenderProperty::Opacity,
                        value: RenderValueOp::TypewriterLimit(visible_limit),
                    });
                }
                EffectType::Keyframe => {
                    if effect.keyframes.is_empty() {
                        continue;
                    }

                    // Keyframe search logic (hoisted)
                    let mut start_kf = &effect.keyframes[0];
                    let mut end_kf = &effect.keyframes[0];
                    let mut found = false;

                    for kf in &effect.keyframes {
                        if kf.time >= *eased_progress {
                            end_kf = kf;
                            found = true;
                            break;
                        }
                        start_kf = kf;
                    }

                    if !found {
                        start_kf = effect.keyframes.last().unwrap();
                        end_kf = start_kf;
                    }

                    let segment_duration = end_kf.time - start_kf.time;
                    let t = if segment_duration <= 0.0 {
                        if *eased_progress >= end_kf.time {
                            1.0
                        } else {
                            0.0
                        }
                    } else {
                        (*eased_progress - start_kf.time) / segment_duration
                    };

                    let segment_eased = if let Some(e) = &start_kf.easing {
                        Self::ease(t, e)
                    } else {
                        t
                    };

                    // Emit Constant ops for all keyframe properties
                    if let (Some(s), Some(e)) = (start_kf.opacity, end_kf.opacity) {
                        ops.push(CompiledRenderOp {
                            prop: RenderProperty::Opacity,
                            value: RenderValueOp::Constant(Self::lerp(
                                s as f64,
                                e as f64,
                                segment_eased,
                            ) as f32),
                        });
                    }
                    if let (Some(s), Some(e)) = (start_kf.scale, end_kf.scale) {
                        ops.push(CompiledRenderOp {
                            prop: RenderProperty::Scale,
                            value: RenderValueOp::Constant(Self::lerp(
                                s as f64,
                                e as f64,
                                segment_eased,
                            ) as f32),
                        });
                    }
                    if let (Some(s), Some(e)) = (start_kf.scale_x, end_kf.scale_x) {
                        ops.push(CompiledRenderOp {
                            prop: RenderProperty::ScaleX,
                            value: RenderValueOp::Constant(Self::lerp(
                                s as f64,
                                e as f64,
                                segment_eased,
                            ) as f32),
                        });
                    }
                    if let (Some(s), Some(e)) = (start_kf.scale_y, end_kf.scale_y) {
                        ops.push(CompiledRenderOp {
                            prop: RenderProperty::ScaleY,
                            value: RenderValueOp::Constant(Self::lerp(
                                s as f64,
                                e as f64,
                                segment_eased,
                            ) as f32),
                        });
                    }
                    if let (Some(s), Some(e)) = (start_kf.rotation, end_kf.rotation) {
                        ops.push(CompiledRenderOp {
                            prop: RenderProperty::Rotation,
                            value: RenderValueOp::Constant(Self::lerp(
                                s as f64,
                                e as f64,
                                segment_eased,
                            ) as f32),
                        });
                    }
                    if let (Some(s), Some(e)) = (start_kf.x, end_kf.x) {
                        ops.push(CompiledRenderOp {
                            prop: RenderProperty::X,
                            value: RenderValueOp::Constant(Self::lerp(
                                s as f64,
                                e as f64,
                                segment_eased,
                            ) as f32),
                        });
                    }
                    if let (Some(s), Some(e)) = (start_kf.y, end_kf.y) {
                        ops.push(CompiledRenderOp {
                            prop: RenderProperty::Y,
                            value: RenderValueOp::Constant(Self::lerp(
                                s as f64,
                                e as f64,
                                segment_eased,
                            ) as f32),
                        });
                    }
                    if let (Some(s), Some(e)) = (start_kf.blur, end_kf.blur) {
                        ops.push(CompiledRenderOp {
                            prop: RenderProperty::Blur,
                            value: RenderValueOp::Constant(Self::lerp(
                                s as f64,
                                e as f64,
                                segment_eased,
                            ) as f32),
                        });
                    }
                    if let (Some(s), Some(e)) = (start_kf.glitch_offset, end_kf.glitch_offset) {
                        ops.push(CompiledRenderOp {
                            prop: RenderProperty::GlitchOffset,
                            value: RenderValueOp::Constant(Self::lerp(
                                s as f64,
                                e as f64,
                                segment_eased,
                            ) as f32),
                        });
                    }
                }
                _ => {}
            }
        }
    }

    /// Apply compiled operations to a RenderTransform.
    /// This is optimized to run inside tight loops (per-glyph).
    pub fn apply_compiled_ops(
        mut transform: RenderTransform,
        ops: &[CompiledRenderOp],
        fast_ctx: &mut crate::expressions::FastEvaluationContext,
    ) -> RenderTransform {
        use evalexpr::Context;
        for op in ops {
            match &op.value {
                RenderValueOp::Constant(v) => apply_property_enum(&mut transform, op.prop, *v),
                RenderValueOp::Expression(node, progress) => {
                    // Update progress in existing context
                    fast_ctx.set_progress(*progress);

                    // Use pre-compiled node
                    match ExpressionEvaluator::evaluate_node_fast(node, fast_ctx) {
                        Ok(v) => apply_property_enum(&mut transform, op.prop, v as f32),
                        Err(e) => {
                            log::trace!("Expr error: {}", e);
                        }
                    }
                }
                RenderValueOp::TypewriterLimit(visible_limit) => {
                    // Get index from fast context
                    if let Some(val) = fast_ctx.get_value("index") {
                        if let evalexpr::Value::Int(idx) = val {
                            if (*idx as f64) < visible_limit {
                                apply_property_enum(&mut transform, op.prop, 1.0);
                            } else {
                                apply_property_enum(&mut transform, op.prop, 0.0);
                            }
                        }
                    }
                }
            }
        }
        transform
    }

    /// Calculate current transform based on active effects
    pub fn compute_transform<T: std::borrow::Borrow<Effect>>(
        current_time: f64,
        base_transform: Transform,
        effects: &[T],
        trigger_context: &TriggerContext,
    ) -> Transform {
        let mut final_transform = base_transform;

        for effect_wrapper in effects {
            let effect = effect_wrapper.borrow();
            if !Self::should_trigger(effect, trigger_context) {
                continue;
            }

            let progress = Self::calculate_progress(current_time, effect, trigger_context);
            if !(0.0..=1.0).contains(&progress) {
                continue;
            }

            let eased_progress = Self::ease(progress, &effect.easing);

            match effect.effect_type {
                EffectType::Transition => {
                    // Create evaluation context for this frame
                    let eval_ctx = EvaluationContext {
                        t: current_time,
                        progress: eased_progress,
                        index: trigger_context.char_index,
                        count: trigger_context.char_count,
                        ..Default::default()
                    };

                    for (prop, value) in &effect.properties {
                        let val = match value {
                            AnimatedValue::Range { from, to } => {
                                Self::lerp(*from, *to, eased_progress)
                            }
                            AnimatedValue::Expression(expr) => {
                                match ExpressionEvaluator::evaluate(expr, &eval_ctx) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        log::warn!("Expression error in {}: {}", prop, e);
                                        continue;
                                    }
                                }
                            }
                        };
                        apply_property(&mut final_transform, prop, val);
                    }
                }
                EffectType::Typewriter => {
                    // Hardcoded typewriter effect logic
                    if let (Some(idx), Some(_count)) =
                        (trigger_context.char_index, trigger_context.char_count)
                    {
                        let total_chars = trigger_context.char_count.unwrap_or(1) as f64;
                        let visible_limit = eased_progress * total_chars;

                        if (idx as f64) < visible_limit {
                            apply_property(&mut final_transform, "opacity", 1.0);
                        } else {
                            apply_property(&mut final_transform, "opacity", 0.0);
                        }
                    }
                }
                EffectType::Keyframe => {
                    if effect.keyframes.is_empty() {
                        continue;
                    }
                    // Logic preserved...
                    let mut start_kf = &effect.keyframes[0];
                    let mut end_kf = &effect.keyframes[0];
                    let mut found = false;

                    for kf in &effect.keyframes {
                        if kf.time >= eased_progress {
                            end_kf = kf;
                            found = true;
                            break;
                        }
                        start_kf = kf;
                    }

                    if !found {
                        start_kf = effect.keyframes.last().unwrap();
                        end_kf = start_kf;
                    }

                    let segment_duration = end_kf.time - start_kf.time;
                    let t = if segment_duration <= 0.0 {
                        if eased_progress >= end_kf.time {
                            1.0
                        } else {
                            0.0
                        }
                    } else {
                        (eased_progress - start_kf.time) / segment_duration
                    };

                    let segment_eased = if let Some(e) = &start_kf.easing {
                        Self::ease(t, e)
                    } else {
                        t
                    };

                    if let (Some(s), Some(e)) = (start_kf.opacity, end_kf.opacity) {
                        apply_property(
                            &mut final_transform,
                            "opacity",
                            Self::lerp(s as f64, e as f64, segment_eased),
                        );
                    }
                    if let (Some(s), Some(e)) = (start_kf.scale, end_kf.scale) {
                        apply_property(
                            &mut final_transform,
                            "scale",
                            Self::lerp(s as f64, e as f64, segment_eased),
                        );
                    }
                    if let (Some(s), Some(e)) = (start_kf.scale_x, end_kf.scale_x) {
                        final_transform.scale_x =
                            Some(Self::lerp(s as f64, e as f64, segment_eased) as f32);
                    }
                    if let (Some(s), Some(e)) = (start_kf.scale_y, end_kf.scale_y) {
                        final_transform.scale_y =
                            Some(Self::lerp(s as f64, e as f64, segment_eased) as f32);
                    }
                    if let (Some(s), Some(e)) = (start_kf.rotation, end_kf.rotation) {
                        apply_property(
                            &mut final_transform,
                            "rotation",
                            Self::lerp(s as f64, e as f64, segment_eased),
                        );
                    }
                    if let (Some(s), Some(e)) = (start_kf.x, end_kf.x) {
                        apply_property(
                            &mut final_transform,
                            "x",
                            Self::lerp(s as f64, e as f64, segment_eased),
                        );
                    }
                    if let (Some(s), Some(e)) = (start_kf.y, end_kf.y) {
                        apply_property(
                            &mut final_transform,
                            "y",
                            Self::lerp(s as f64, e as f64, segment_eased),
                        );
                    }
                    if let (Some(s), Some(e)) = (start_kf.blur, end_kf.blur) {
                        apply_property(
                            &mut final_transform,
                            "blur",
                            Self::lerp(s as f64, e as f64, segment_eased),
                        );
                    }
                    if let (Some(s), Some(e)) = (start_kf.glitch_offset, end_kf.glitch_offset) {
                        apply_property(
                            &mut final_transform,
                            "glitch_offset",
                            Self::lerp(s as f64, e as f64, segment_eased),
                        );
                    }
                }
                _ => {}
            }
        }

        final_transform
    }

    /// Optimized version of compute_transform for RenderTransform (dense struct)
    pub fn apply_to_render_transform<T: std::borrow::Borrow<Effect>>(
        current_time: f64,
        mut transform: RenderTransform,
        effects: &[T],
        trigger_context: &TriggerContext,
    ) -> RenderTransform {
        for effect_wrapper in effects {
            let effect = effect_wrapper.borrow();
            if !Self::should_trigger(effect, trigger_context) {
                continue;
            }

            let progress = Self::calculate_progress(current_time, effect, trigger_context);
            if !(0.0..=1.0).contains(&progress) {
                continue;
            }

            let eased_progress = Self::ease(progress, &effect.easing);

            match effect.effect_type {
                EffectType::Transition => {
                    let eval_ctx = EvaluationContext {
                        t: current_time,
                        progress: eased_progress,
                        index: trigger_context.char_index,
                        count: trigger_context.char_count,
                        ..Default::default()
                    };

                    for (prop, value) in &effect.properties {
                        let val = match value {
                            AnimatedValue::Range { from, to } => {
                                Self::lerp(*from, *to, eased_progress)
                            }
                            AnimatedValue::Expression(expr) => {
                                match ExpressionEvaluator::evaluate(expr, &eval_ctx) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        log::warn!("Expression error in {}: {}", prop, e);
                                        continue;
                                    }
                                }
                            }
                        };
                        apply_property_to_render(&mut transform, prop, val);
                    }
                }
                EffectType::Typewriter => {
                    if let (Some(idx), Some(_count)) =
                        (trigger_context.char_index, trigger_context.char_count)
                    {
                        let total_chars = trigger_context.char_count.unwrap_or(1) as f64;
                        let visible_limit = eased_progress * total_chars;

                        if (idx as f64) < visible_limit {
                            apply_property_to_render(&mut transform, "opacity", 1.0);
                        } else {
                            apply_property_to_render(&mut transform, "opacity", 0.0);
                        }
                    }
                }
                EffectType::Keyframe => {
                    if effect.keyframes.is_empty() {
                        continue;
                    }
                    let mut start_kf = &effect.keyframes[0];
                    let mut end_kf = &effect.keyframes[0];
                    let mut found = false;

                    for kf in &effect.keyframes {
                        if kf.time >= eased_progress {
                            end_kf = kf;
                            found = true;
                            break;
                        }
                        start_kf = kf;
                    }

                    if !found {
                        start_kf = effect.keyframes.last().unwrap();
                        end_kf = start_kf;
                    }

                    let segment_duration = end_kf.time - start_kf.time;
                    let t = if segment_duration <= 0.0 {
                        if eased_progress >= end_kf.time {
                            1.0
                        } else {
                            0.0
                        }
                    } else {
                        (eased_progress - start_kf.time) / segment_duration
                    };

                    let segment_eased = if let Some(e) = &start_kf.easing {
                        Self::ease(t, e)
                    } else {
                        t
                    };

                    if let (Some(s), Some(e)) = (start_kf.opacity, end_kf.opacity) {
                        apply_property_to_render(
                            &mut transform,
                            "opacity",
                            Self::lerp(s as f64, e as f64, segment_eased),
                        );
                    }
                    if let (Some(s), Some(e)) = (start_kf.scale, end_kf.scale) {
                        apply_property_to_render(
                            &mut transform,
                            "scale",
                            Self::lerp(s as f64, e as f64, segment_eased),
                        );
                    }
                    if let (Some(s), Some(e)) = (start_kf.scale_x, end_kf.scale_x) {
                        transform.scale_x = Self::lerp(s as f64, e as f64, segment_eased) as f32;
                    }
                    if let (Some(s), Some(e)) = (start_kf.scale_y, end_kf.scale_y) {
                        transform.scale_y = Self::lerp(s as f64, e as f64, segment_eased) as f32;
                    }
                    if let (Some(s), Some(e)) = (start_kf.rotation, end_kf.rotation) {
                        apply_property_to_render(
                            &mut transform,
                            "rotation",
                            Self::lerp(s as f64, e as f64, segment_eased),
                        );
                    }
                    if let (Some(s), Some(e)) = (start_kf.x, end_kf.x) {
                        apply_property_to_render(
                            &mut transform,
                            "x",
                            Self::lerp(s as f64, e as f64, segment_eased),
                        );
                    }
                    if let (Some(s), Some(e)) = (start_kf.y, end_kf.y) {
                        apply_property_to_render(
                            &mut transform,
                            "y",
                            Self::lerp(s as f64, e as f64, segment_eased),
                        );
                    }
                    if let (Some(s), Some(e)) = (start_kf.blur, end_kf.blur) {
                        apply_property_to_render(
                            &mut transform,
                            "blur",
                            Self::lerp(s as f64, e as f64, segment_eased),
                        );
                    }
                    if let (Some(s), Some(e)) = (start_kf.glitch_offset, end_kf.glitch_offset) {
                        apply_property_to_render(
                            &mut transform,
                            "glitch_offset",
                            Self::lerp(s as f64, e as f64, segment_eased),
                        );
                    }
                }
                _ => {}
            }
        }
        transform
    }

    /// Check if an effect should trigger based on context
    pub fn should_trigger(_effect: &Effect, _ctx: &TriggerContext) -> bool {
        // Basic trigger logic
        true // simplified for now
    }

    /// Calculate progress of an effect (0.0 to 1.0)
    pub fn calculate_progress(current_time: f64, effect: &Effect, ctx: &TriggerContext) -> f64 {
        let duration = effect.duration.unwrap_or(ctx.end_time - ctx.start_time);
        let start = ctx.start_time + effect.delay;

        if current_time < start {
            return -1.0;
        }

        let elapsed = current_time - start;
        (elapsed / duration).clamp(0.0, 1.0)
    }

    /// Process modifier layers
    pub fn apply_layers(
        current_time: f64,
        base_transform: &Transform,
        layers: &[EffectLayer],
        ctx: &TriggerContext,
        // We might need particle system access here later, but for now just Transform
    ) -> Transform {
        let mut final_transform = base_transform.clone();

        for layer in layers {
            if Self::matches_selector(&layer.selector, ctx) {
                for modifier in &layer.modifiers {
                    Self::apply_modifier(&mut final_transform, modifier, current_time, ctx);
                }
            }
        }
        
        final_transform
    }

    /// Process modifier layers for RenderTransform
    pub fn apply_layers_to_render(
        current_time: f64,
        base_transform: RenderTransform,
        layers: &[EffectLayer],
        ctx: &TriggerContext,
    ) -> RenderTransform {
        let mut final_transform = base_transform;

        for layer in layers {
            if Self::matches_selector(&layer.selector, ctx) {
                for modifier in &layer.modifiers {
                    Self::apply_modifier_to_render(&mut final_transform, modifier, current_time, ctx);
                }
            }
        }

        final_transform
    }

    fn matches_selector(selector: &Selector, ctx: &TriggerContext) -> bool {
        match selector {
            Selector::All => true,
            Selector::Scope(scope) => {
                match scope {
                    ScopeType::Document => true, // Usually always true if reached here
                    ScopeType::Line => true, // Context is usually within a line
                    ScopeType::Word => true, // Need word index in context? For now assume per-char acts as word if grouped
                    ScopeType::Char => ctx.char_index.is_some(),
                    ScopeType::Syllable => true, // Similar to word/char
                }
            },
            Selector::Pattern { n, offset } => {
                if let Some(idx) = ctx.char_index {
                    (idx % n) == *offset
                } else {
                    false
                }
            },
            Selector::TimeRange { start, end } => {
                let _t = ctx.current_time; // This might need to be absolute or relative
                // Spec says TimeRange. Let's assume relative to line?
                // Or maybe drivers handle time. Selector might toggle.
                // Let's assume local time relative to context start
                 let local_t = ctx.current_time - ctx.start_time;
                 local_t >= *start as f64 && local_t <= *end as f64
            },
            Selector::Text { .. } => true, // Not implemented fully without text access
            Selector::Tag(_) => true, // Tagging system not passed in context
        }
    }

    fn apply_modifier(transform: &mut Transform, modifier: &Modifier, time: f64, _ctx: &TriggerContext) {
        // Time usage: ValueDrivers usually take absolute time or relative?
        // Let's use `time` (which is usually `current_time` from render loop).
        // If ValueDrivers need normalized 0-1, we might need context. 
        // But `ValueDriver::Sine` needs continuous time.
        
        match modifier {
            Modifier::Move(p) => {
                let x = DriverManager::evaluate(&p.x, time);
                let y = DriverManager::evaluate(&p.y, time);
                transform.x = Some(transform.x.unwrap_or(0.0) + x);
                transform.y = Some(transform.y.unwrap_or(0.0) + y);
            },
            Modifier::Scale(p) => {
                let sx = DriverManager::evaluate(&p.x, time);
                let sy = DriverManager::evaluate(&p.y, time);
                // Multiplicative or additive? Usually multiplicative for scale.
                // But Driver returns a value. 
                // E.g. Sine(1.0, 0.1) -> varies 0.9 to 1.1.
                // So we multiply.
                transform.scale_x = Some(transform.scale_x.unwrap_or(1.0) * sx);
                transform.scale_y = Some(transform.scale_y.unwrap_or(1.0) * sy);
            },
            Modifier::Rotate(p) => {
                let angle = DriverManager::evaluate(&p.angle, time);
                transform.rotation = Some(transform.rotation.unwrap_or(0.0) + angle);
                // pivot not handled in Transform struct yet
            },
            Modifier::Color(p) => {
                if let Some(_c) = &p.fill {
                    // For now, override fill using simple string
                    // ValueDriver not used for string yet
                    // But maybe we want color blending?
                    // Spec says `fill: Option<String>`. 
                    // This creates a fixed color override for now.
                    // Future: ValueDriver for Color components?
                }
            },
            Modifier::Fade(p) => {
                let alpha = DriverManager::evaluate(&p.value, time);
                transform.opacity = Some(transform.opacity.unwrap_or(1.0) * alpha);
            },
            Modifier::Blur(sigma) => {
                transform.blur = Some(*sigma);
            },
            Modifier::Jitter(p) => {
                let amt = DriverManager::evaluate(&p.amount, time);
                let speed = DriverManager::evaluate(&p.speed, time);
                // Pseudo random based on time * speed
                let off_x = (time as f32 * speed).sin() * amt;
                let off_y = (time as f32 * speed * 1.5).cos() * amt;
                
                transform.x = Some(transform.x.unwrap_or(0.0) + off_x);
                transform.y = Some(transform.y.unwrap_or(0.0) + off_y);
            },
            Modifier::Wave(p) => {
                let freq = DriverManager::evaluate(&p.freq, time);
                let amp = DriverManager::evaluate(&p.amp, time);
                 // Wave usually needs position index (spatial).
                 // But Modifier receives Context?
                 // Wait, Modifier applies to Transform. Transform is for a single char?
                 // If so, `time` + `char_index` * offset?
                 // But `process_layers` iterates CHARS.
                 // We need to bake `char_index` into the evaluation if "Wave" means "Standard Text Wave".
                 // BUT `ValueDriver` takes `time`.
                 // If the Agent defines `y: Sine`, it moves the whole object up/down.
                 // To achieve "Wave", the Agent must use a Selector or the Modifier must be "Spatial".
                 // `Modifier::Wave` is explicitly explicit for this.
                 
                 // If `ctx` has index, we use it.
                 let idx = _ctx.char_index.unwrap_or(0) as f32;
                 // speed?
                 let speed = DriverManager::evaluate(&p.speed, time);
                 
                 let phase = idx * 0.5; // arbitary spatial freq
                 let y = (time as f32 * speed + phase * freq).sin() * amp;
                 
                 transform.y = Some(transform.y.unwrap_or(0.0) + y);
            },
            Modifier::Appear(p) => {
                 // Logic likely similar to Typewriter but driven by `progress`.
                 let progress = DriverManager::evaluate(&p.progress, time);
                 match p.mode {
                     AppearMode::Fade => {
                         transform.opacity = Some(progress.clamp(0.0, 1.0));
                     },
                     _ => {}
                 }
            },
            _ => {}
        }
    }

    fn apply_modifier_to_render(transform: &mut RenderTransform, modifier: &Modifier, time: f64, _ctx: &TriggerContext) {
        match modifier {
            Modifier::Move(p) => {
                let x = DriverManager::evaluate(&p.x, time);
                let y = DriverManager::evaluate(&p.y, time);
                transform.x += x;
                transform.y += y;
            },
            Modifier::Scale(p) => {
                let sx = DriverManager::evaluate(&p.x, time);
                let sy = DriverManager::evaluate(&p.y, time);
                transform.scale_x *= sx;
                transform.scale_y *= sy;
            },
            Modifier::Rotate(p) => {
                let angle = DriverManager::evaluate(&p.angle, time);
                transform.rotation += angle;
            },
            Modifier::Color(_p) => {
                // Not supported in Transform/RenderTransform for now
            },
            Modifier::Fade(p) => {
                let alpha = DriverManager::evaluate(&p.value, time);
                transform.opacity *= alpha;
            },
            Modifier::Blur(sigma) => {
                transform.blur = *sigma;
            },
            Modifier::Jitter(p) => {
                let amt = DriverManager::evaluate(&p.amount, time);
                let speed = DriverManager::evaluate(&p.speed, time);
                let off_x = (time as f32 * speed).sin() * amt;
                let off_y = (time as f32 * speed * 1.5).cos() * amt;
                transform.x += off_x;
                transform.y += off_y;
            },
            Modifier::Wave(p) => {
                let freq = DriverManager::evaluate(&p.freq, time);
                let amp = DriverManager::evaluate(&p.amp, time);
                 let idx = _ctx.char_index.unwrap_or(0) as f32;
                 let speed = DriverManager::evaluate(&p.speed, time);
                 let phase = idx * 0.5;
                 let y = (time as f32 * speed + phase * freq).sin() * amp;
                 transform.y += y;
            },
            Modifier::Appear(p) => {
                 let progress = DriverManager::evaluate(&p.progress, time);
                 match p.mode {
                     AppearMode::Fade => {
                         transform.opacity = progress.clamp(0.0, 1.0);
                     },
                     _ => {}
                 }
            },
            _ => {}
        }
    }
}

fn text_cos(val: f64) -> f64 {
    val.cos()
}

#[derive(Clone)]
pub struct TriggerContext {
    pub start_time: f64,
    pub end_time: f64,
    pub current_time: f64,
    pub active: bool,
    pub char_index: Option<usize>,
    pub char_count: Option<usize>,
}

impl Default for TriggerContext {
    fn default() -> Self {
        Self {
            start_time: 0.0,
            end_time: 1.0,
            current_time: 0.0,
            active: false,
            char_index: None,
            char_count: None,
        }
    }
}

fn apply_property(transform: &mut Transform, prop: &str, value: f64) {
    match prop {
        "opacity" => transform.opacity = Some(value as f32),
        "scale" => {
            transform.scale = Some(value as f32);
            transform.scale_x = Some(value as f32);
            transform.scale_y = Some(value as f32);
        }
        "x" => transform.x = Some(value as f32),
        "y" => transform.y = Some(value as f32),
        "rotation" => transform.rotation = Some(value as f32),
        "blur" => transform.blur = Some(value as f32),
        "glitch_offset" | "glitch" => transform.glitch_offset = Some(value as f32),
        _ => {}
    }
}

fn apply_property_to_render(transform: &mut RenderTransform, prop: &str, value: f64) {
    match prop {
        "opacity" => transform.opacity = value as f32,
        "scale" => transform.scale = value as f32,
        "scale_x" => transform.scale_x = value as f32,
        "scale_y" => transform.scale_y = value as f32,
        "x" => transform.x = value as f32,
        "y" => transform.y = value as f32,
        "rotation" => transform.rotation = value as f32,
        "blur" => transform.blur = value as f32,
        "glitch_offset" | "glitch" => transform.glitch_offset = value as f32,
        "anchor_x" => transform.anchor_x = value as f32,
        "anchor_y" => transform.anchor_y = value as f32,
        "hue_shift" => transform.hue_shift = value as f32,
        _ => {}
    }
}

fn apply_property_enum(transform: &mut RenderTransform, prop: RenderProperty, value: f32) {
    match prop {
        RenderProperty::Opacity => transform.opacity = value,
        RenderProperty::Scale => transform.scale = value,
        RenderProperty::ScaleX => transform.scale_x = value,
        RenderProperty::ScaleY => transform.scale_y = value,
        RenderProperty::X => transform.x = value,
        RenderProperty::Y => transform.y = value,
        RenderProperty::Rotation => transform.rotation = value,
        RenderProperty::Blur => transform.blur = value,
        RenderProperty::GlitchOffset => transform.glitch_offset = value,
        RenderProperty::AnchorX => transform.anchor_x = value,
        RenderProperty::AnchorY => transform.anchor_y = value,
        RenderProperty::HueShift => transform.hue_shift = value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AnimatedValue, Effect, EffectTrigger, EffectType};
    use std::collections::HashMap;

    // Helper to wrap effect in ResolvedEffect
    fn resolve(effect: Effect) -> ResolvedEffect {
        // Compile any expressions
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
            name_hash: 0, // Default for tests
        }
    }

    /// Helper for float comparison with tolerance
    fn approx_eq(a: f64, b: f64, tolerance: f64) -> bool {
        (a - b).abs() < tolerance
    }

    // ============================================================================
    // EASING FUNCTION TESTS (16 tests)
    // ============================================================================

    #[test]
    fn test_ease_linear() {
        // Linear easing: f(t) = t
        assert!(approx_eq(
            EffectEngine::ease(0.0, &Easing::Linear),
            0.0,
            1e-9
        ));
        assert!(approx_eq(
            EffectEngine::ease(0.5, &Easing::Linear),
            0.5,
            1e-9
        ));
        assert!(approx_eq(
            EffectEngine::ease(1.0, &Easing::Linear),
            1.0,
            1e-9
        ));
    }

    #[test]
    fn test_ease_in_quad() {
        // EaseInQuad: f(t) = t^2
        assert!(approx_eq(
            EffectEngine::ease(0.0, &Easing::EaseInQuad),
            0.0,
            1e-9
        ));
        assert!(approx_eq(
            EffectEngine::ease(0.5, &Easing::EaseInQuad),
            0.25,
            1e-9
        ));
        assert!(approx_eq(
            EffectEngine::ease(1.0, &Easing::EaseInQuad),
            1.0,
            1e-9
        ));
        // Also test alias EaseIn
        assert!(approx_eq(
            EffectEngine::ease(0.5, &Easing::EaseIn),
            0.25,
            1e-9
        ));
    }

    #[test]
    fn test_ease_out_quad() {
        // EaseOutQuad: f(t) = t * (2 - t)
        assert!(approx_eq(
            EffectEngine::ease(0.0, &Easing::EaseOutQuad),
            0.0,
            1e-9
        ));
        assert!(approx_eq(
            EffectEngine::ease(0.5, &Easing::EaseOutQuad),
            0.75,
            1e-9
        ));
        assert!(approx_eq(
            EffectEngine::ease(1.0, &Easing::EaseOutQuad),
            1.0,
            1e-9
        ));
        // Also test alias EaseOut
        assert!(approx_eq(
            EffectEngine::ease(0.5, &Easing::EaseOut),
            0.75,
            1e-9
        ));
    }

    #[test]
    fn test_ease_in_out_quad() {
        // EaseInOutQuad: split function at t=0.5
        assert!(approx_eq(
            EffectEngine::ease(0.0, &Easing::EaseInOutQuad),
            0.0,
            1e-9
        ));
        assert!(approx_eq(
            EffectEngine::ease(0.25, &Easing::EaseInOutQuad),
            0.125,
            1e-9
        ));
        assert!(approx_eq(
            EffectEngine::ease(0.5, &Easing::EaseInOutQuad),
            0.5,
            1e-9
        ));
        assert!(approx_eq(
            EffectEngine::ease(1.0, &Easing::EaseInOutQuad),
            1.0,
            1e-9
        ));
        // Also test alias EaseInOut
        assert!(approx_eq(
            EffectEngine::ease(0.5, &Easing::EaseInOut),
            0.5,
            1e-9
        ));
    }

    #[test]
    fn test_ease_in_cubic() {
        // EaseInCubic: f(t) = t^3
        assert!(approx_eq(
            EffectEngine::ease(0.0, &Easing::EaseInCubic),
            0.0,
            1e-9
        ));
        assert!(approx_eq(
            EffectEngine::ease(0.5, &Easing::EaseInCubic),
            0.125,
            1e-9
        ));
        assert!(approx_eq(
            EffectEngine::ease(1.0, &Easing::EaseInCubic),
            1.0,
            1e-9
        ));
    }

    #[test]
    fn test_ease_out_cubic() {
        // EaseOutCubic: f(t) = (t-1)^3 + 1
        assert!(approx_eq(
            EffectEngine::ease(0.0, &Easing::EaseOutCubic),
            0.0,
            1e-9
        ));
        assert!(approx_eq(
            EffectEngine::ease(0.5, &Easing::EaseOutCubic),
            0.875,
            1e-9
        ));
        assert!(approx_eq(
            EffectEngine::ease(1.0, &Easing::EaseOutCubic),
            1.0,
            1e-9
        ));
    }

    #[test]
    fn test_ease_in_out_cubic() {
        // EaseInOutCubic: split function at t=0.5
        assert!(approx_eq(
            EffectEngine::ease(0.0, &Easing::EaseInOutCubic),
            0.0,
            1e-9
        ));
        assert!(approx_eq(
            EffectEngine::ease(0.5, &Easing::EaseInOutCubic),
            0.5,
            1e-9
        ));
        assert!(approx_eq(
            EffectEngine::ease(1.0, &Easing::EaseInOutCubic),
            1.0,
            1e-9
        ));
    }

    #[test]
    fn test_ease_in_sine() {
        // EaseInSine: f(t) = 1 - cos(t * PI / 2)
        assert!(approx_eq(
            EffectEngine::ease(0.0, &Easing::EaseInSine),
            0.0,
            1e-9
        ));
        assert!(approx_eq(
            EffectEngine::ease(1.0, &Easing::EaseInSine),
            1.0,
            1e-9
        ));
        // At t=0.5, should be ~0.293
        let mid = EffectEngine::ease(0.5, &Easing::EaseInSine);
        assert!(mid > 0.0 && mid < 0.5);
    }

    #[test]
    fn test_ease_out_sine() {
        // EaseOutSine: f(t) = sin(t * PI / 2)
        assert!(approx_eq(
            EffectEngine::ease(0.0, &Easing::EaseOutSine),
            0.0,
            1e-9
        ));
        assert!(approx_eq(
            EffectEngine::ease(1.0, &Easing::EaseOutSine),
            1.0,
            1e-9
        ));
        // At t=0.5, should be ~0.707
        let mid = EffectEngine::ease(0.5, &Easing::EaseOutSine);
        assert!(mid > 0.5 && mid < 1.0);
    }

    #[test]
    fn test_ease_in_out_sine() {
        // EaseInOutSine: f(t) = -(cos(PI * t) - 1) / 2
        assert!(approx_eq(
            EffectEngine::ease(0.0, &Easing::EaseInOutSine),
            0.0,
            1e-9
        ));
        assert!(approx_eq(
            EffectEngine::ease(0.5, &Easing::EaseInOutSine),
            0.5,
            1e-9
        ));
        assert!(approx_eq(
            EffectEngine::ease(1.0, &Easing::EaseInOutSine),
            1.0,
            1e-9
        ));
    }

    #[test]
    fn test_ease_in_expo() {
        // EaseInExpo: f(0) = 0, otherwise f(t) = 2^(10(t-1))
        assert!(approx_eq(
            EffectEngine::ease(0.0, &Easing::EaseInExpo),
            0.0,
            1e-9
        ));
        assert!(approx_eq(
            EffectEngine::ease(1.0, &Easing::EaseInExpo),
            1.0,
            1e-9
        ));
        // Exponential start: very small value at t=0.5
        let mid = EffectEngine::ease(0.5, &Easing::EaseInExpo);
        assert!(mid > 0.0 && mid < 0.1);
    }

    #[test]
    fn test_ease_out_expo() {
        // EaseOutExpo: f(1) = 1, otherwise f(t) = 1 - 2^(-10t)
        assert!(approx_eq(
            EffectEngine::ease(0.0, &Easing::EaseOutExpo),
            0.0,
            1e-9
        ));
        assert!(approx_eq(
            EffectEngine::ease(1.0, &Easing::EaseOutExpo),
            1.0,
            1e-9
        ));
        // Exponential end: very high value at t=0.5
        let mid = EffectEngine::ease(0.5, &Easing::EaseOutExpo);
        assert!(mid > 0.9 && mid < 1.0);
    }

    #[test]
    fn test_ease_out_elastic() {
        // EaseOutElastic: special case for 0 and 1
        assert!(approx_eq(
            EffectEngine::ease(0.0, &Easing::EaseOutElastic),
            0.0,
            1e-9
        ));
        assert!(approx_eq(
            EffectEngine::ease(1.0, &Easing::EaseOutElastic),
            1.0,
            1e-9
        ));
        // Elastic can overshoot, at t=0.5 it should be around 1.0
        let mid = EffectEngine::ease(0.5, &Easing::EaseOutElastic);
        assert!(mid > 0.9);
    }

    #[test]
    fn test_ease_out_bounce() {
        // EaseOutBounce: defined in 4 sections
        assert!(approx_eq(
            EffectEngine::ease(0.0, &Easing::EaseOutBounce),
            0.0,
            1e-9
        ));
        assert!(approx_eq(
            EffectEngine::ease(1.0, &Easing::EaseOutBounce),
            1.0,
            1e-9
        ));
        // Test a point in the "bounce" region
        let mid = EffectEngine::ease(0.5, &Easing::EaseOutBounce);
        assert!(mid > 0.5 && mid < 1.0);
    }

    #[test]
    fn test_ease_fallback() {
        // Unimplemented easing functions should fall back to linear
        // EaseInQuart is defined in enum but not implemented, so falls back to t
        assert!(approx_eq(
            EffectEngine::ease(0.5, &Easing::EaseInQuart),
            0.5,
            1e-9
        ));
    }

    #[test]
    fn test_ease_boundary_consistency() {
        // All easing functions should have f(0) close to 0 and f(1) close to 1
        let easings = vec![
            Easing::Linear,
            Easing::EaseIn,
            Easing::EaseOut,
            Easing::EaseInOut,
            Easing::EaseInQuad,
            Easing::EaseOutQuad,
            Easing::EaseInOutQuad,
            Easing::EaseInCubic,
            Easing::EaseOutCubic,
            Easing::EaseInOutCubic,
            Easing::EaseInSine,
            Easing::EaseOutSine,
            Easing::EaseInOutSine,
            Easing::EaseInExpo,
            Easing::EaseOutExpo,
            Easing::EaseOutElastic,
            Easing::EaseOutBounce,
        ];

        for easing in easings {
            let at_zero = EffectEngine::ease(0.0, &easing);
            let at_one = EffectEngine::ease(1.0, &easing);
            assert!(
                approx_eq(at_zero, 0.0, 1e-6),
                "Easing {:?} at 0.0 should be 0",
                easing
            );
            assert!(
                approx_eq(at_one, 1.0, 1e-6),
                "Easing {:?} at 1.0 should be 1",
                easing
            );
        }
    }

    // ============================================================================
    // LERP TESTS (3 tests)
    // ============================================================================

    #[test]
    fn test_lerp_basic() {
        // lerp(0, 100, 0.5) = 50
        assert!(approx_eq(EffectEngine::lerp(0.0, 100.0, 0.0), 0.0, 1e-9));
        assert!(approx_eq(EffectEngine::lerp(0.0, 100.0, 0.5), 50.0, 1e-9));
        assert!(approx_eq(EffectEngine::lerp(0.0, 100.0, 1.0), 100.0, 1e-9));
    }

    #[test]
    fn test_lerp_negative() {
        // lerp with negative values
        assert!(approx_eq(EffectEngine::lerp(-100.0, 100.0, 0.5), 0.0, 1e-9));
        assert!(approx_eq(
            EffectEngine::lerp(-50.0, -10.0, 0.5),
            -30.0,
            1e-9
        ));
    }

    #[test]
    fn test_lerp_same() {
        // lerp when start == end
        assert!(approx_eq(EffectEngine::lerp(42.0, 42.0, 0.0), 42.0, 1e-9));
        assert!(approx_eq(EffectEngine::lerp(42.0, 42.0, 0.5), 42.0, 1e-9));
        assert!(approx_eq(EffectEngine::lerp(42.0, 42.0, 1.0), 42.0, 1e-9));
    }

    // ============================================================================
    // PROGRESS CALCULATION TESTS (3 tests)
    // ============================================================================

    fn make_effect(duration: Option<f64>, delay: f64) -> Effect {
        Effect {
            effect_type: EffectType::Transition,
            trigger: EffectTrigger::Enter,
            duration,
            delay,
            easing: Easing::Linear,
            properties: HashMap::new(),
            mode: None,
            direction: None,
            keyframes: Vec::new(),
            preset: None,
            particle_config: None,
            iterations: 1,
            particle_override: None,
        }
    }

    fn make_context(start: f64, end: f64) -> TriggerContext {
        TriggerContext {
            start_time: start,
            end_time: end,
            current_time: start,
            active: true,
            char_index: None,
            char_count: None,
        }
    }

    #[test]
    fn test_calculate_progress_before() {
        // Before effect starts, progress should be -1.0
        let effect = make_effect(Some(1.0), 0.0);
        let ctx = make_context(5.0, 10.0);

        // Current time 4.0 is before start_time 5.0
        let progress = EffectEngine::calculate_progress(4.0, &effect, &ctx);
        assert!(progress < 0.0, "Progress before start should be negative");
    }

    #[test]
    fn test_calculate_progress_during() {
        // During effect, progress should be 0.0 to 1.0
        let effect = make_effect(Some(2.0), 0.0);
        let ctx = make_context(0.0, 10.0);

        // At start
        let at_start = EffectEngine::calculate_progress(0.0, &effect, &ctx);
        assert!(approx_eq(at_start, 0.0, 1e-9));

        // At middle (1s into 2s duration)
        let at_mid = EffectEngine::calculate_progress(1.0, &effect, &ctx);
        assert!(approx_eq(at_mid, 0.5, 1e-9));

        // At end
        let at_end = EffectEngine::calculate_progress(2.0, &effect, &ctx);
        assert!(approx_eq(at_end, 1.0, 1e-9));
    }

    #[test]
    fn test_calculate_progress_after() {
        // After effect ends, progress should be clamped to 1.0
        let effect = make_effect(Some(1.0), 0.0);
        let ctx = make_context(0.0, 10.0);

        // Way past the end
        let progress = EffectEngine::calculate_progress(100.0, &effect, &ctx);
        assert!(approx_eq(progress, 1.0, 1e-9));
    }

    // ============================================================================
    // TRANSFORM APPLICATION TESTS (3 tests)
    // ============================================================================

    #[test]
    fn test_apply_property_opacity() {
        let mut transform = Transform::default();
        apply_property(&mut transform, "opacity", 0.5);
        assert!(approx_eq(transform.opacity.unwrap() as f64, 0.5, 1e-6));
    }

    #[test]
    fn test_apply_property_scale() {
        let mut transform = Transform::default();
        apply_property(&mut transform, "scale", 2.0);
        assert!(approx_eq(transform.scale.unwrap() as f64, 2.0, 1e-6));
        assert!(approx_eq(transform.scale_x.unwrap() as f64, 2.0, 1e-6));
        assert!(approx_eq(transform.scale_y.unwrap() as f64, 2.0, 1e-6));
    }

    #[test]
    fn test_compute_transform_no_effects() {
        // compute_transform with empty effects should return base transform unchanged
        let base = Transform {
            x: Some(10.0),
            y: Some(20.0),
            opacity: Some(0.8),
            ..Default::default()
        };
        let ctx = make_context(0.0, 10.0);

        // Explicitly type empty slice to satisfy generics if needed, or let inference work
        let effects: &[&Effect] = &[];
        let result = EffectEngine::compute_transform(5.0, base, effects, &ctx);

        assert!(approx_eq(result.x.unwrap() as f64, 10.0, 1e-6));
        assert!(approx_eq(result.y.unwrap() as f64, 20.0, 1e-6));
        assert!(approx_eq(result.opacity.unwrap() as f64, 0.8, 1e-6));
    }

    #[test]
    fn test_compute_transform_with_transition() {
        // Test transition effect applying opacity change
        let base = Transform::default();
        let ctx = make_context(0.0, 10.0);

        let mut props = HashMap::new();
        props.insert(
            "opacity".to_string(),
            AnimatedValue::Range { from: 0.0, to: 1.0 },
        );

        let effect = Effect {
            effect_type: EffectType::Transition,
            trigger: EffectTrigger::Enter,
            duration: Some(2.0),
            delay: 0.0,
            easing: Easing::Linear,
            properties: props,
            mode: None,
            direction: None,
            keyframes: Vec::new(),
            preset: None,
            particle_config: None,
            iterations: 1,
            particle_override: None,
        };

        // At t=1.0, progress is 0.5, so opacity should be 0.5
        let result = EffectEngine::compute_transform(1.0, base, &[&effect], &ctx);
        assert!(approx_eq(result.opacity.unwrap() as f64, 0.5, 1e-6));
    }

    #[test]
    fn test_apply_property_all_types() {
        let mut transform = Transform::default();

        apply_property(&mut transform, "x", 100.0);
        apply_property(&mut transform, "y", 200.0);
        apply_property(&mut transform, "rotation", 45.0);
        apply_property(&mut transform, "blur", 5.0);
        apply_property(&mut transform, "glitch_offset", 10.0);

        assert!(approx_eq(transform.x.unwrap() as f64, 100.0, 1e-6));
        assert!(approx_eq(transform.y.unwrap() as f64, 200.0, 1e-6));
        assert!(approx_eq(transform.rotation.unwrap() as f64, 45.0, 1e-6));
        assert!(approx_eq(transform.blur.unwrap() as f64, 5.0, 1e-6));
        assert!(approx_eq(
            transform.glitch_offset.unwrap() as f64,
            10.0,
            1e-6
        ));
    }

    #[test]
    fn test_apply_property_glitch_alias() {
        // Test that "glitch" is an alias for "glitch_offset"
        let mut transform = Transform::default();
        apply_property(&mut transform, "glitch", 15.0);
        assert!(approx_eq(
            transform.glitch_offset.unwrap() as f64,
            15.0,
            1e-6
        ));
    }

    #[test]
    fn test_apply_property_unknown() {
        // Unknown property should be ignored
        let mut transform = Transform::default();
        apply_property(&mut transform, "unknown_property", 999.0);
        // Transform should remain in default state
        assert!(transform.x.is_none());
        assert!(transform.opacity.is_none());
    }

    #[test]
    fn test_compile_ops_transition() {
        let ctx = make_context(0.0, 10.0);
        let mut props = HashMap::new();
        props.insert(
            "opacity".to_string(),
            AnimatedValue::Range { from: 0.0, to: 1.0 },
        );
        let effect = Effect {
            effect_type: EffectType::Transition,
            trigger: EffectTrigger::Enter,
            duration: Some(2.0),
            delay: 0.0,
            easing: Easing::Linear,
            properties: props,
            mode: None,
            direction: None,
            keyframes: Vec::new(),
            preset: None,
            particle_config: None,
            iterations: 1,
            particle_override: None,
        };

        // Use resolve helper
        let resolved = resolve(effect);

        // progress = 0.5
        let effects_source = vec![resolved];
        let active_indices = vec![(0, 0.5)];
        let mut ops = Vec::new();
        EffectEngine::compile_active_effects(&effects_source, &active_indices, &ctx, &mut ops);
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].prop, RenderProperty::Opacity);
        match &ops[0].value {
            RenderValueOp::Constant(v) => assert!(approx_eq(*v as f64, 0.5, 1e-6)),
            _ => panic!("Expected Constant"),
        }
    }

    #[test]
    fn test_compile_ops_typewriter() {
        let mut ctx = make_context(0.0, 10.0);
        ctx.char_count = Some(10);

        let effect = Effect {
            effect_type: EffectType::Typewriter,
            trigger: EffectTrigger::Enter,
            duration: Some(2.0),
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

        let resolved = resolve(effect);

        // progress = 0.5 -> 5 chars visible
        let effects_source = vec![resolved];
        let active_indices = vec![(0, 0.5)];
        let mut ops = Vec::new();
        EffectEngine::compile_active_effects(&effects_source, &active_indices, &ctx, &mut ops);
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].prop, RenderProperty::Opacity);
        match &ops[0].value {
            RenderValueOp::TypewriterLimit(lim) => assert!(approx_eq(*lim, 5.0, 1e-6)),
            _ => panic!("Expected TypewriterLimit"),
        }
    }

    #[test]
    fn test_compile_ops_expression_with_progress() {
        let ctx = make_context(0.0, 10.0);
        let mut props = HashMap::new();
        props.insert(
            "x".to_string(),
            AnimatedValue::Expression("progress * 100.0".to_string()),
        );
        let effect = Effect {
            effect_type: EffectType::Transition,
            trigger: EffectTrigger::Enter,
            duration: Some(2.0),
            delay: 0.0,
            easing: Easing::Linear,
            properties: props,
            mode: None,
            direction: None,
            keyframes: Vec::new(),
            preset: None,
            particle_config: None,
            iterations: 1,
            particle_override: None,
        };

        let resolved = resolve(effect);

        // progress = 0.5
        let effects_source = vec![resolved];
        let active_indices = vec![(0, 0.5)];
        let mut ops = Vec::new();
        EffectEngine::compile_active_effects(&effects_source, &active_indices, &ctx, &mut ops);
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].prop, RenderProperty::X);

        if let RenderValueOp::Expression(_node, p) = &ops[0].value {
            // RenderValueOp now holds &Node
            assert!(approx_eq(*p, 0.5, 1e-6));

            // Now apply it
            let mut transform = RenderTransform::default();
            let eval_ctx = EvaluationContext {
                t: 0.0,
                progress: 0.0, // Should be overridden
                ..Default::default()
            };
            let mut fast_ctx = crate::expressions::FastEvaluationContext::new(&eval_ctx);

            transform = EffectEngine::apply_compiled_ops(transform, &ops, &mut fast_ctx);
            assert!(approx_eq(transform.x as f64, 50.0, 1e-6));
        } else {
            panic!("Expected Expression");
        }
    }
}

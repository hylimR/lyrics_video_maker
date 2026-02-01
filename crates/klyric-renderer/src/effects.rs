use super::model::{AnimatedValue, Easing, Effect, EffectType, RenderTransform, Transform};
use crate::expressions::{EvaluationContext, ExpressionEvaluator};
use std::f64::consts::PI;

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
pub enum RenderValueOp<'a> {
    Constant(f32),
    Expression(&'a str, f64), // Store (expression, progress)
    TypewriterLimit(f64),
}

pub struct CompiledRenderOp<'a> {
    pub prop: RenderProperty,
    pub value: RenderValueOp<'a>,
}

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
    pub fn compile_active_effects<'a>(
        active_effects: &'a [(&'a Effect, f64)],
        ctx: &TriggerContext,
    ) -> Vec<CompiledRenderOp<'a>> {
        let mut ops = Vec::with_capacity(active_effects.len() * 2);

        for (effect, eased_progress) in active_effects {
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
                                AnimatedValue::Expression(expr) => {
                                    ops.push(CompiledRenderOp {
                                        prop,
                                        value: RenderValueOp::Expression(expr, *eased_progress),
                                    });
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
        ops
    }

    /// Apply compiled operations to a RenderTransform.
    /// This is optimized to run inside tight loops (per-glyph).
    pub fn apply_compiled_ops(
        mut transform: RenderTransform,
        ops: &[CompiledRenderOp],
        eval_ctx: &EvaluationContext,
    ) -> RenderTransform {
        for op in ops {
            match op.value {
                RenderValueOp::Constant(v) => apply_property_enum(&mut transform, op.prop, v),
                RenderValueOp::Expression(expr, progress) => {
                    // Create a local context with the effect's progress
                    let mut local_ctx = eval_ctx.clone();
                    local_ctx.progress = progress;

                    match ExpressionEvaluator::evaluate(expr, &local_ctx) {
                        Ok(v) => apply_property_enum(&mut transform, op.prop, v as f32),
                        Err(e) => {
                            log::trace!("Expr error: {}", e);
                        }
                    }
                }
                RenderValueOp::TypewriterLimit(visible_limit) => {
                    if let Some(idx) = eval_ctx.index {
                        if (idx as f64) < visible_limit {
                            apply_property_enum(&mut transform, op.prop, 1.0);
                        } else {
                            apply_property_enum(&mut transform, op.prop, 0.0);
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
                    // Hardcoded typewriter effect:
                    // Characters reveal one by one based on `delay` and `duration`
                    // total_duration is effect.duration
                    // char_delay = duration / count
                    if let (Some(idx), Some(_count)) =
                        (trigger_context.char_index, trigger_context.char_count)
                    {
                        // Use eased progress (0 to 1) to determine how many chars to show
                        // If progress = 0.5, show first 50%
                        // Actually, typewriter usually means discrete steps
                        // We map progress 0..1 to index 0..count

                        // We use the raw progress (linear) for the "cursor" position mostly,
                        // but allowing easing is nice too.
                        let _visible_ratio = eased_progress;

                        // If we are "in" the typewriter sequence
                        // Let's assume we reveal from 0 to count
                        // Threshold for this char = index / count
                        // If visible_opacity < 1.0, we might fade in?
                        // For classic typewriter, it's instant.

                        // However, let's allow a small fade window if we wanted, but for now simple check:
                        // Simple logic:
                        // total chars = count
                        // current visible count = progress * count
                        // if index < visible_count, show.

                        // But we also need to handle total duration.
                        let total_chars = trigger_context.char_count.unwrap_or(1) as f64;
                        let visible_limit = eased_progress * total_chars;

                        // If index is 5, and visible_limit is 5.1 -> show
                        // If index is 5, and visible_limit is 4.9 -> hide

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

                    // 1. Sort keyframes by time (assume sorted or sort here if needed, but usually pre-sorted)
                    // We assume sorted for performance.

                    // 2. Find current interval
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
                        // Past last keyframe
                        start_kf = effect.keyframes.last().unwrap();
                        end_kf = start_kf;
                    }

                    // 3. Interpolate between start_kf and end_kf
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

                    // Optional per-keyframe easing
                    let segment_eased = if let Some(e) = &start_kf.easing {
                        Self::ease(t, e)
                    } else {
                        t
                    };

                    // Apply properties
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

    /// Apply pre-calculated effects to a RenderTransform
    pub fn apply_active_effects(
        current_time: f64,
        mut transform: RenderTransform,
        active_effects: &[(&Effect, f64)],
        trigger_context: &TriggerContext,
    ) -> RenderTransform {
        for (effect, eased_progress) in active_effects {
            match effect.effect_type {
                EffectType::Transition => {
                    let eval_ctx = EvaluationContext {
                        t: current_time,
                        progress: *eased_progress,
                        index: trigger_context.char_index,
                        count: trigger_context.char_count,
                        ..Default::default()
                    };

                    for (prop, value) in &effect.properties {
                        let val = match value {
                            AnimatedValue::Range { from, to } => {
                                Self::lerp(*from, *to, *eased_progress)
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
                        let visible_limit = *eased_progress * total_chars;

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

        // progress = 0.5
        let ops = EffectEngine::compile_active_effects(&[(&effect, 0.5)], &ctx);
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].prop, RenderProperty::Opacity);
        match ops[0].value {
            RenderValueOp::Constant(v) => assert!(approx_eq(v as f64, 0.5, 1e-6)),
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

        // progress = 0.5 -> 5 chars visible
        let ops = EffectEngine::compile_active_effects(&[(&effect, 0.5)], &ctx);
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].prop, RenderProperty::Opacity);
        match ops[0].value {
            RenderValueOp::TypewriterLimit(lim) => assert!(approx_eq(lim, 5.0, 1e-6)),
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

        // progress = 0.5
        let ops = EffectEngine::compile_active_effects(&[(&effect, 0.5)], &ctx);
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].prop, RenderProperty::X);

        if let RenderValueOp::Expression(expr, p) = ops[0].value {
            assert_eq!(expr, "progress * 100.0");
            assert!(approx_eq(p, 0.5, 1e-6));

            // Now apply it
            let mut transform = RenderTransform::default();
            let eval_ctx = EvaluationContext {
                t: 0.0,
                progress: 0.0, // Should be overridden
                ..Default::default()
            };

            transform = EffectEngine::apply_compiled_ops(transform, &ops, &eval_ctx);
            assert!(approx_eq(transform.x as f64, 50.0, 1e-6));
        } else {
            panic!("Expected Expression");
        }
    }
}

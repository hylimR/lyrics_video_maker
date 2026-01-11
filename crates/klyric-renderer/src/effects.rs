use super::model::{Easing, Effect, EffectType, Transform};
use std::f64::consts::PI;

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
                if t < 0.5 { 2.0 * t * t } else { -1.0 + (4.0 - 2.0 * t) * t }
            },
            
            // Cubic
            Easing::EaseInCubic => t * t * t,
            Easing::EaseOutCubic => {
                let t = t - 1.0;
                t * t * t + 1.0
            },
            Easing::EaseInOutCubic => {
                if t < 0.5 { 4.0 * t * t * t } else { (t - 1.0) * (2.0 * t - 2.0).powi(2) + 1.0 }
            },
            
            // Sine
            Easing::EaseInSine => 1.0 - (t * PI / 2.0).cos(),
            Easing::EaseOutSine => (t * PI / 2.0).sin(),
            Easing::EaseInOutSine => -(text_cos(PI * t) - 1.0) / 2.0,
            
            // Exponential
            Easing::EaseInExpo => if t == 0.0 { 0.0 } else { 2.0f64.powf(10.0 * (t - 1.0)) },
            Easing::EaseOutExpo => if t == 1.0 { 1.0 } else { 1.0 - 2.0f64.powf(-10.0 * t) },
            
            // Elastic
            Easing::EaseOutElastic => {
                let c4 = (2.0 * PI) / 3.0;
                if t == 0.0 { 0.0 }
                else if t == 1.0 { 1.0 }
                else { 2.0f64.powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0 }
            },
            
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
            },
            
            // Fallback for others
            _ => t, 
        }
    }
    
    /// Interpolate between two values
    pub fn lerp(start: f64, end: f64, t: f64) -> f64 {
        start + (end - start) * t
    }
    
    /// Calculate current transform based on active effects
    pub fn compute_transform(
        current_time: f64,
        base_transform: &Transform,
        effects: &[Effect],
        trigger_context: TriggerContext
    ) -> Transform {
        let mut final_transform = base_transform.clone();
        
        for effect in effects {
            if !Self::should_trigger(effect, &trigger_context) {
                continue;
            }
            
            let progress = Self::calculate_progress(current_time, effect, &trigger_context);
            if progress < 0.0 || progress > 1.0 {
                continue;
            }
            
            let eased_progress = Self::ease(progress, &effect.easing);
            
            match effect.effect_type {
                EffectType::Transition => {
                    for (prop, value) in &effect.properties {
                        let current_val = Self::lerp(value.from, value.to, eased_progress);
                        apply_property(&mut final_transform, prop, current_val);
                    }
                },
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
                        if eased_progress >= end_kf.time { 1.0 } else { 0.0 }
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
                        apply_property(&mut final_transform, "opacity", Self::lerp(s as f64, e as f64, segment_eased));
                    }
                    if let (Some(s), Some(e)) = (start_kf.scale, end_kf.scale) {
                         apply_property(&mut final_transform, "scale", Self::lerp(s as f64, e as f64, segment_eased));
                    }
                    if let (Some(s), Some(e)) = (start_kf.scale_x, end_kf.scale_x) {
                        final_transform.scale_x = Self::lerp(s as f64, e as f64, segment_eased) as f32;
                    }
                    if let (Some(s), Some(e)) = (start_kf.scale_y, end_kf.scale_y) {
                        final_transform.scale_y = Self::lerp(s as f64, e as f64, segment_eased) as f32;
                    }
                    if let (Some(s), Some(e)) = (start_kf.rotation, end_kf.rotation) {
                         apply_property(&mut final_transform, "rotation", Self::lerp(s as f64, e as f64, segment_eased));
                    }
                    if let (Some(s), Some(e)) = (start_kf.x, end_kf.x) {
                         apply_property(&mut final_transform, "x", Self::lerp(s as f64, e as f64, segment_eased));
                    }
                    if let (Some(s), Some(e)) = (start_kf.y, end_kf.y) {
                         apply_property(&mut final_transform, "y", Self::lerp(s as f64, e as f64, segment_eased));
                    }
                },
                _ => {}
            }
        }
        
        final_transform
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
        (elapsed / duration).min(1.0).max(0.0)
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
}

fn apply_property(transform: &mut Transform, prop: &str, value: f64) {
    match prop {
        "opacity" => transform.opacity = value as f32,
        "scale" => {
            transform.scale = value as f32;
            transform.scale_x = value as f32;
            transform.scale_y = value as f32;
        },
        "x" => transform.x = value as f32,
        "y" => transform.y = value as f32,
        "rotation" => transform.rotation = value as f32,
        _ => {}
    }
}

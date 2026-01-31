use crate::model::modifiers::{ValueDriver, DriverEasing};

// We might need to map DriverEasing to effect::Easing if possible, 
// or implement a separate ease function. For now, let's implement basic ease.

pub struct DriverManager;

impl DriverManager {
    pub fn evaluate(driver: &ValueDriver, time: f64) -> f32 {
        match driver {
            ValueDriver::Fixed { val } => *val,
            ValueDriver::Linear { start, end, ease } => {
                // Currently ValueDriver doesn't know "absolute time duration" from itself alone
                // It usually implies time is normalized 0.0-1.0.
                // If time is seconds, we need to know the duration.
                // BUT, looking at the Spec, the ValueDriver often assumes the context provides normalized time?
                // Or maybe Time is passed in.
                
                // For now, let's assume `time` is normalized progress (0.0 to 1.0)
                let t = Self::apply_easing(time, ease);
                Self::lerp(*start, *end, t)
            },
            ValueDriver::Sine { base, amp, freq, phase } => {
                // Sine is usually time based (seconds)
                // If time is normalized progress, freq should be high.
                // Let's assume time is seconds.
                let val = (time as f32 * freq * 2.0 * std::f32::consts::PI + phase).sin();
                base + val * amp
            },
            ValueDriver::Noise { base, amp, speed } => {
                // Placeholder for noise. In real impl use a noise crate or simple pseudo-random
                // For now, use sin for determinism
                let val = (time as f32 * speed * 123.45).sin() * (time as f32 * speed * 67.89).cos();
                base + val * amp
            },
            ValueDriver::Step { values, interval } => {
                if values.is_empty() { return 0.0; }
                let idx = (time as f32 / interval).floor() as usize;
                values[idx % values.len()]
            },
            ValueDriver::Default => 0.0,
        }
    }
    
    fn lerp(start: f32, end: f32, t: f32) -> f32 {
        start + (end - start) * t
    }
    
    fn apply_easing(t: f64, ease: &DriverEasing) -> f32 {
        // Map DriverEasing to our Easing impl
        // Or re-implement.
        let t = t as f32;
        match ease {
            DriverEasing::Linear => t,
            DriverEasing::QuadIn => t * t,
            DriverEasing::QuadOut => t * (2.0 - t),
            DriverEasing::QuadInOut => if t < 0.5 { 2.0 * t * t } else { -1.0 + (4.0 - 2.0 * t) * t },
            
            // ... implement others as needed
            _ => t, // Fallback
        }
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "params")]
pub enum Modifier {
    // --- Transform Modifiers ---
    /// Basic affine transforms
    Move(MoveParams),
    Scale(ScaleParams),
    Rotate(RotateParams),

    // --- Visual Modifiers ---
    Color(ColorParams),
    Fade(FadeParams),
    Blur(f32),

    // --- Deformers ---
    Wave(WaveParams),
    Jitter(JitterParams),
    Perspect(PerspectParams),

    // --- Text Modifiers ---
    Appear(AppearParams),
    Spacing(f32),

    // --- Particle Emitters ---
    Emit(EmitParams),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveParams {
    pub x: ValueDriver,
    pub y: ValueDriver,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleParams {
    pub x: ValueDriver,
    pub y: ValueDriver,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotateParams {
    pub angle: ValueDriver,
    pub pivot_x: f32,
    pub pivot_y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorParams {
    // Simplified for now, can be expanded
    pub fill: Option<String>,
    pub stroke: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FadeParams {
    pub value: ValueDriver,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveParams {
    pub freq: ValueDriver,
    pub amp: ValueDriver,
    pub speed: ValueDriver,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JitterParams {
    pub amount: ValueDriver,
    pub speed: ValueDriver,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerspectParams {
    pub depth: ValueDriver,
    pub rotate_x: ValueDriver,
    pub rotate_y: ValueDriver,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearParams {
    pub mode: AppearMode,
    pub progress: ValueDriver,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppearMode {
    Typewriter,
    Fade,
    Random,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmitParams {
    pub preset: String, // Reference to particle preset
    pub rate: ValueDriver,
}

// --- Value Drivers ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mode")]
pub enum ValueDriver {
    Fixed {
        val: f32,
    },
    Linear {
        start: f32,
        end: f32,
        ease: DriverEasing,
    },
    Sine {
        base: f32,
        amp: f32,
        freq: f32,
        phase: f32,
    },
    Noise {
        base: f32,
        amp: f32,
        speed: f32,
    },
    Step {
        values: Vec<f32>,
        interval: f32,
    },
    // Defaults for easy JSON
    Default,
}

impl Default for ValueDriver {
    fn default() -> Self {
        ValueDriver::Fixed { val: 0.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DriverEasing {
    Linear,
    QuadIn,
    QuadOut,
    QuadInOut,
    CubicIn,
    CubicOut,
    CubicInOut,
    BackIn,
    BackOut,
    BackInOut,
    BounceIn,
    BounceOut,
    BounceInOut,
    ElasticIn,
    ElasticOut,
    ElasticInOut,
}

// --- Selectors (Targeting) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectLayer {
    pub selector: Selector,
    pub modifiers: Vec<Modifier>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mode", content = "args")]
pub enum Selector {
    All,
    Scope(ScopeType),
    Pattern { n: usize, offset: usize },
    TimeRange { start: f32, end: f32 },
    Text { contains: String },
    Tag(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScopeType {
    Document,
    Line,
    Word,
    Char,
    Syllable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modifier_deserialization() {
        let json = r#"{
            "type": "Move",
            "params": {
                "x": { "mode": "Fixed", "val": 10.0 },
                "y": { "mode": "Fixed", "val": 20.0 }
            }
        }"#;

        let modifier: Modifier = serde_json::from_str(json).unwrap();
        match modifier {
            Modifier::Move(params) => {
                match params.x {
                    ValueDriver::Fixed { val } => assert_eq!(val, 10.0),
                    _ => panic!("Expected Fixed"),
                }
                match params.y {
                    ValueDriver::Fixed { val } => assert_eq!(val, 20.0),
                    _ => panic!("Expected Fixed"),
                }
            }
            _ => panic!("Expected Move"),
        }
    }

    #[test]
    fn test_value_driver_deserialization() {
        let json_linear = r#"{
            "mode": "Linear",
            "start": 0.0,
            "end": 100.0,
            "ease": "QuadIn"
        }"#;

        let driver: ValueDriver = serde_json::from_str(json_linear).unwrap();
        match driver {
            ValueDriver::Linear { start, end, ease } => {
                assert_eq!(start, 0.0);
                assert_eq!(end, 100.0);
                match ease {
                    DriverEasing::QuadIn => {}
                    _ => panic!("Expected QuadIn"),
                }
            }
            _ => panic!("Expected Linear"),
        }
    }

    #[test]
    fn test_selector_deserialization() {
        let json_scope = r#"{
            "mode": "Scope",
            "args": "Line"
        }"#;

        let selector: Selector = serde_json::from_str(json_scope).unwrap();
        match selector {
            Selector::Scope(ScopeType::Line) => {}
            _ => panic!("Expected Scope(Line)"),
        }

        let json_pattern = r#"{
            "mode": "Pattern",
            "args": { "n": 2, "offset": 1 }
        }"#;

        let selector_pattern: Selector = serde_json::from_str(json_pattern).unwrap();
        match selector_pattern {
            Selector::Pattern { n, offset } => {
                assert_eq!(n, 2);
                assert_eq!(offset, 1);
            }
            _ => panic!("Expected Pattern"),
        }
    }
}

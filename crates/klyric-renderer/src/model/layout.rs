use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    /// X position (pixels or percentage string)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<PositionValue>,

    /// Y position (pixels or percentage string)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<PositionValue>,

    /// Anchor point for positioning
    #[serde(default)]
    pub anchor: Anchor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PositionValue {
    Pixels(f32),
    Percentage(String),
}

impl Default for PositionValue {
    fn default() -> Self {
        PositionValue::Percentage("50%".to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Anchor {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    #[default]
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Transform {
    /// X offset in pixels
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<f32>,

    /// Y offset in pixels
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<f32>,

    /// Rotation in degrees
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<f32>,

    /// Uniform scale factor
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<f32>,

    /// Horizontal scale factor
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale_x: Option<f32>,

    /// Vertical scale factor
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale_y: Option<f32>,

    /// Opacity (0-1)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f32>,

    /// Transform anchor X (0-1, 0.5 = center)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor_x: Option<f32>,

    /// Transform anchor Y (0-1, 0.5 = center)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor_y: Option<f32>,

    /// Blur sigma
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blur: Option<f32>,

    /// Glitch offset (pixels to shift channels)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub glitch_offset: Option<f32>,

    /// Hue shift in degrees
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hue_shift: Option<f32>,
}

pub fn default_scale() -> f32 {
    1.0
}
pub fn default_opacity() -> f32 {
    1.0
}
pub fn default_anchor() -> f32 {
    0.5
}

impl Transform {
    pub fn x_val(&self) -> f32 {
        self.x.unwrap_or(0.0)
    }
    pub fn y_val(&self) -> f32 {
        self.y.unwrap_or(0.0)
    }
    pub fn rotation_val(&self) -> f32 {
        self.rotation.unwrap_or(0.0)
    }
    pub fn scale_val(&self) -> f32 {
        self.scale.unwrap_or_else(default_scale)
    }
    pub fn scale_x_val(&self) -> f32 {
        self.scale_x.unwrap_or_else(default_scale)
    }
    pub fn scale_y_val(&self) -> f32 {
        self.scale_y.unwrap_or_else(default_scale)
    }
    pub fn opacity_val(&self) -> f32 {
        self.opacity.unwrap_or_else(default_opacity)
    }
    pub fn anchor_x_val(&self) -> f32 {
        self.anchor_x.unwrap_or_else(default_anchor)
    }
    pub fn anchor_y_val(&self) -> f32 {
        self.anchor_y.unwrap_or_else(default_anchor)
    }
    pub fn blur_val(&self) -> f32 {
        self.blur.unwrap_or(0.0)
    }
    pub fn glitch_offset_val(&self) -> f32 {
        self.glitch_offset.unwrap_or(0.0)
    }
    pub fn hue_shift_val(&self) -> f32 {
        self.hue_shift.unwrap_or(0.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Layout {
    /// Text layout mode
    #[serde(default)]
    pub mode: LayoutMode,

    /// Horizontal alignment
    #[serde(default)]
    pub align: Align,

    /// Vertical alignment
    #[serde(default)]
    pub justify: Justify,

    /// Gap between characters in pixels
    #[serde(default)]
    pub gap: f32,

    /// Whether to wrap text
    #[serde(default)]
    pub wrap: bool,

    /// Maximum width before wrapping
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_width: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LayoutMode {
    #[default]
    Horizontal,
    Vertical,
    Path,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Align {
    Left,
    #[default]
    Center,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Justify {
    Top,
    #[default]
    #[serde(alias = "center")]
    Middle,
    Bottom,
}

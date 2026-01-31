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

/// A dense version of Transform optimized for rendering.
/// Contains direct f32 values instead of Options.
#[derive(Debug, Clone, Copy)]
pub struct RenderTransform {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub scale: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub opacity: f32,
    pub anchor_x: f32,
    pub anchor_y: f32,
    pub blur: f32,
    pub glitch_offset: f32,
    pub hue_shift: f32,
}

impl Default for RenderTransform {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            scale: 1.0,
            scale_x: 1.0,
            scale_y: 1.0,
            opacity: 1.0,
            anchor_x: 0.5,
            anchor_y: 0.5,
            blur: 0.0,
            glitch_offset: 0.0,
            hue_shift: 0.0,
        }
    }
}

impl RenderTransform {
    /// Create a RenderTransform by combining line and char transforms.
    /// Logic mirrors existing additive/multiplicative combination in LineRenderer.
    pub fn new(line: &Transform, char_t: &Transform) -> Self {
        Self {
            x: line.x_val() + char_t.x_val(),
            y: line.y_val() + char_t.y_val(),
            rotation: line.rotation_val() + char_t.rotation_val(),
            scale: line.scale_val() * char_t.scale_val(),
            scale_x: line.scale_x_val() * char_t.scale_x_val(),
            scale_y: line.scale_y_val() * char_t.scale_y_val(),
            opacity: line.opacity_val() * char_t.opacity_val(),
            // Anchor is usually not additive, char overrides line if present, but here we just follow line renderer which was taking char_val.
            // Wait, LineRenderer was: anchor_x: Some(char_transform_ref.anchor_x_val())
            // It completely ignored line_transform for anchor!
            anchor_x: char_t.anchor_x_val(),
            anchor_y: char_t.anchor_y_val(),
            blur: line.blur_val() + char_t.blur_val(),
            glitch_offset: line.glitch_offset_val() + char_t.glitch_offset_val(),
            hue_shift: line.hue_shift_val() + char_t.hue_shift_val(),
        }
    }

    /// Apply a sparse delta transform (e.g. from prefix optimization)
    pub fn apply_delta(&mut self, delta: &Transform) {
        if let Some(v) = delta.x {
            self.x = v;
        }
        if let Some(v) = delta.y {
            self.y = v;
        }
        if let Some(v) = delta.rotation {
            self.rotation = v;
        }
        if let Some(v) = delta.scale {
            self.scale = v;
        }
        if let Some(v) = delta.scale_x {
            self.scale_x = v;
        }
        if let Some(v) = delta.scale_y {
            self.scale_y = v;
        }
        if let Some(v) = delta.opacity {
            self.opacity = v;
        }
        if let Some(v) = delta.anchor_x {
            self.anchor_x = v;
        }
        if let Some(v) = delta.anchor_y {
            self.anchor_y = v;
        }
        if let Some(v) = delta.blur {
            self.blur = v;
        }
        if let Some(v) = delta.glitch_offset {
            self.glitch_offset = v;
        }
        if let Some(v) = delta.hue_shift {
            self.hue_shift = v;
        }
    }
}

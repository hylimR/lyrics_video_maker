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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transform {
    /// X offset in pixels
    #[serde(default)]
    pub x: f32,
    
    /// Y offset in pixels
    #[serde(default)]
    pub y: f32,
    
    /// Rotation in degrees
    #[serde(default)]
    pub rotation: f32,
    
    /// Uniform scale factor
    #[serde(default = "default_scale")]
    pub scale: f32,
    
    /// Horizontal scale factor
    #[serde(default = "default_scale")]
    pub scale_x: f32,
    
    /// Vertical scale factor
    #[serde(default = "default_scale")]
    pub scale_y: f32,
    
    /// Opacity (0-1)
    #[serde(default = "default_opacity")]
    pub opacity: f32,
    
    /// Transform anchor X (0-1, 0.5 = center)
    #[serde(default = "default_anchor")]
    pub anchor_x: f32,
    
    /// Transform anchor Y (0-1, 0.5 = center)
    #[serde(default = "default_anchor")]
    pub anchor_y: f32,
}

impl Default for Transform {
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
        }
    }
}

fn default_scale() -> f32 { 1.0 }
fn default_opacity() -> f32 { 1.0 }
fn default_anchor() -> f32 { 0.5 }

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

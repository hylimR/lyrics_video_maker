use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Theme {
    /// Background configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<Background>,
    
    /// Name of default style for all lines
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_style: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Background {
    /// Background type
    #[serde(rename = "type", default)]
    pub bg_type: BackgroundType,
    
    /// Solid color (hex or rgba)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    
    /// Gradient definition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gradient: Option<Gradient>,
    
    /// Path to background image
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    
    /// Path to background video
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<String>,
    
    /// Background opacity (0-1)
    #[serde(default = "default_opacity")]
    pub opacity: f32,
}

fn default_opacity() -> f32 { 1.0 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum BackgroundType {
    #[default]
    Solid,
    Gradient,
    Image,
    Video,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Gradient {
    /// Gradient type
    #[serde(rename = "type", default)]
    pub gradient_type: GradientType,
    
    /// Array of colors in gradient
    pub colors: Vec<String>,
    
    /// Angle in degrees for linear gradients
    #[serde(default = "default_gradient_angle")]
    pub angle: f32,
    
    /// Optional stop positions (0-1)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stops: Option<Vec<f32>>,
}

fn default_gradient_angle() -> f32 { 180.0 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum GradientType {
    #[default]
    Linear,
    Radial,
}

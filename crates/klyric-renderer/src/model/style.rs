use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Style {
    /// Parent style to inherit from
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,
    
    /// Font settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font: Option<Font>,
    
    /// State-based colors
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colors: Option<StateColors>,
    
    /// Text stroke settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke: Option<Stroke>,
    
    /// Drop shadow settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shadow: Option<Shadow>,
    
    /// Glow effect settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub glow: Option<Glow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Font {
    /// Font family (comma-separated for fallbacks)
    #[serde(default = "default_font_family")]
    pub family: String,
    
    /// Font size in pixels
    #[serde(default = "default_font_size")]
    pub size: f32,
    
    /// Font weight (100-900)
    #[serde(default = "default_font_weight")]
    pub weight: u32,
    
    /// Font style
    #[serde(default)]
    pub style: FontStyle,
    
    /// Letter spacing in pixels
    #[serde(default)]
    pub letter_spacing: f32,
}

fn default_font_family() -> String { "Noto Sans SC".to_string() }
fn default_font_size() -> f32 { 72.0 }
fn default_font_weight() -> u32 { 700 }

impl Default for Font {
    fn default() -> Self {
        Self {
            family: default_font_family(),
            size: default_font_size(),
            weight: default_font_weight(),
            style: FontStyle::default(),
            letter_spacing: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
    Oblique,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StateColors {
    /// Colors before character is highlighted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inactive: Option<FillStroke>,
    
    /// Colors during character highlight
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<FillStroke>,
    
    /// Colors after highlight completes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complete: Option<FillStroke>,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FillStroke {
    /// Fill color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill: Option<String>,
    
    /// Stroke color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke: Option<String>,
}

// Custom deserializer to handle both string and object formats
impl<'de> serde::Deserialize<'de> for FillStroke {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;
        
        struct FillStrokeVisitor;
        
        impl<'de> Visitor<'de> for FillStrokeVisitor {
            type Value = FillStroke;
            
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a color string or an object with fill/stroke fields")
            }
            
            // Handle string: "rgba(255,255,255,0.4)" or "#FFFFFF"
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(FillStroke {
                    fill: Some(value.to_string()),
                    stroke: None,
                })
            }
            
            // Handle object: { fill: "...", stroke: "..." }
            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut fill: Option<String> = None;
                let mut stroke: Option<String> = None;
                
                while let Some(key) = map.next_key::<&str>()? {
                    match key {
                        "fill" => fill = map.next_value()?,
                        "stroke" => stroke = map.next_value()?,
                        _ => { let _ = map.next_value::<serde::de::IgnoredAny>()?; }
                    }
                }
                
                Ok(FillStroke { fill, stroke })
            }
        }
        
        deserializer.deserialize_any(FillStrokeVisitor)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Stroke {
    /// Stroke width in pixels
    #[serde(default)]
    pub width: f32,
    
    /// Stroke color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Shadow {
    /// Shadow color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    
    /// Horizontal offset in pixels
    #[serde(default = "default_shadow_offset")]
    pub x: f32,
    
    /// Vertical offset in pixels
    #[serde(default = "default_shadow_offset")]
    pub y: f32,
    
    /// Blur radius in pixels
    #[serde(default = "default_shadow_blur")]
    pub blur: f32,
}

fn default_shadow_offset() -> f32 { 2.0 }
fn default_shadow_blur() -> f32 { 4.0 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Glow {
    /// Glow color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    
    /// Glow blur radius
    #[serde(default = "default_glow_blur")]
    pub blur: f32,
    
    /// Glow intensity (0-1)
    #[serde(default = "default_glow_intensity")]
    pub intensity: f32,
}

fn default_glow_blur() -> f32 { 8.0 }
fn default_glow_intensity() -> f32 { 0.5 }

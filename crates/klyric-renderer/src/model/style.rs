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

    /// Global transform
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<Transform>,

    /// Global effects (applied to all lines using this style)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effects: Option<Vec<String>>,

    /// Modifier layers (New System)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layers: Option<Vec<EffectLayer>>,
}

use super::layout::Transform;
use super::modifiers::EffectLayer;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Font {
    /// Font family (comma-separated for fallbacks)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,

    /// Font size in pixels
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<f32>,

    /// Font weight (100-900)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<u32>,

    /// Font style
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<FontStyle>,

    /// Letter spacing in pixels
    #[serde(skip_serializing_if = "Option::is_none")]
    pub letter_spacing: Option<f32>,
}

pub fn default_font_family() -> String {
    "Noto Sans SC".to_string()
}
pub fn default_font_size() -> f32 {
    72.0
}
pub fn default_font_weight() -> u32 {
    700
}

impl Font {
    pub fn family_or_default(&self) -> String {
        self.family.clone().unwrap_or_else(default_font_family)
    }
    pub fn size_or_default(&self) -> f32 {
        self.size.unwrap_or_else(default_font_size)
    }
    pub fn weight_or_default(&self) -> u32 {
        self.weight.unwrap_or_else(default_font_weight)
    }
    pub fn style_or_default(&self) -> FontStyle {
        self.style.clone().unwrap_or_default()
    }
    pub fn letter_spacing_or_default(&self) -> f32 {
        self.letter_spacing.unwrap_or(0.0)
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
                        _ => {
                            let _ = map.next_value::<serde::de::IgnoredAny>()?;
                        }
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<f32>,

    /// Stroke color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

impl Stroke {
    pub fn width_or_default(&self) -> f32 {
        self.width.unwrap_or(0.0)
    }
    pub fn color_or_default(&self) -> String {
        self.color.clone().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_deserialization_defaults() {
        let json = r#"{}"#;
        let style: Style = serde_json::from_str(json).unwrap();
        assert!(style.extends.is_none());
        assert!(style.font.is_none());
        assert!(style.colors.is_none());
        assert!(style.stroke.is_none());
        assert!(style.shadow.is_none());
        assert!(style.glow.is_none());
        assert!(style.transform.is_none());
        assert!(style.effects.is_none());
        assert!(style.layers.is_none());
    }

    #[test]
    fn test_font_defaults() {
        let font = Font::default();
        assert_eq!(font.family_or_default(), "Noto Sans SC");
        assert_eq!(font.size_or_default(), 72.0);
        assert_eq!(font.weight_or_default(), 700);
        match font.style_or_default() {
            FontStyle::Normal => {}
            _ => panic!("Expected Normal font style"),
        }
        assert_eq!(font.letter_spacing_or_default(), 0.0);
    }

    #[test]
    fn test_stroke_defaults() {
        let stroke = Stroke::default();
        assert_eq!(stroke.width_or_default(), 0.0);
        assert_eq!(stroke.color_or_default(), "");
    }

    #[test]
    fn test_shadow_defaults() {
        let shadow = Shadow::default();
        assert_eq!(shadow.x_or_default(), 2.0);
        assert_eq!(shadow.y_or_default(), 2.0);
        assert_eq!(shadow.blur_or_default(), 4.0);
        assert_eq!(shadow.color_or_default(), "");
    }

    #[test]
    fn test_glow_defaults() {
        let glow = Glow::default();
        assert_eq!(glow.blur_or_default(), 8.0);
        assert_eq!(glow.intensity_or_default(), 0.5);
        assert_eq!(glow.color_or_default(), "");
    }

    #[test]
    fn test_fill_stroke_custom_deserialization_string() {
        let json = r#""#FFFFFF""#;
        let fs: FillStroke = serde_json::from_str(json).unwrap();
        assert_eq!(fs.fill.as_deref(), Some("#FFFFFF"));
        assert!(fs.stroke.is_none());
    }

    #[test]
    fn test_fill_stroke_custom_deserialization_object() {
        let json = r#"{
            "fill": "#FF0000",
            "stroke": "#00FF00",
            "ignored_field": "test"
        }"#;
        let fs: FillStroke = serde_json::from_str(json).unwrap();
        assert_eq!(fs.fill.as_deref(), Some("#FF0000"));
        assert_eq!(fs.stroke.as_deref(), Some("#00FF00"));
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Shadow {
    /// Shadow color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,

    /// Horizontal offset in pixels
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<f32>,

    /// Vertical offset in pixels
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<f32>,

    /// Blur radius in pixels
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blur: Option<f32>,
}

pub fn default_shadow_offset() -> f32 {
    2.0
}
pub fn default_shadow_blur() -> f32 {
    4.0
}

impl Shadow {
    pub fn x_or_default(&self) -> f32 {
        self.x.unwrap_or_else(default_shadow_offset)
    }
    pub fn y_or_default(&self) -> f32 {
        self.y.unwrap_or_else(default_shadow_offset)
    }
    pub fn blur_or_default(&self) -> f32 {
        self.blur.unwrap_or_else(default_shadow_blur)
    }
    pub fn color_or_default(&self) -> String {
        self.color.clone().unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Glow {
    /// Glow color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,

    /// Glow blur radius
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blur: Option<f32>,

    /// Glow intensity (0-1)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intensity: Option<f32>,
}

pub fn default_glow_blur() -> f32 {
    8.0
}
pub fn default_glow_intensity() -> f32 {
    0.5
}

impl Glow {
    pub fn blur_or_default(&self) -> f32 {
        self.blur.unwrap_or_else(default_glow_blur)
    }
    pub fn intensity_or_default(&self) -> f32 {
        self.intensity.unwrap_or_else(default_glow_intensity)
    }
    pub fn color_or_default(&self) -> String {
        self.color.clone().unwrap_or_default()
    }
}

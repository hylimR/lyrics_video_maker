use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::effect::Effect;
use super::line::Line;
use super::project::Project;
use super::style::Style;
use super::theme::Theme;

/// Root KLyric v2.0 document structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KLyricDocumentV2 {
    /// JSON Schema reference
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,

    /// Format version (must be "2.0")
    pub version: String,

    /// Project metadata
    pub project: Project,

    /// Theme and background settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<Theme>,

    /// Named style definitions
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub styles: HashMap<String, Style>,

    /// Named effect definitions
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub effects: HashMap<String, Effect>,

    /// Lyric lines with timing and characters
    pub lines: Vec<Line>,
}

impl KLyricDocumentV2 {
    /// Parse a KLyric v2.0 document from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serialize to JSON
    pub fn to_json(&self, pretty: bool) -> Result<String, serde_json::Error> {
        if pretty {
            serde_json::to_string_pretty(self)
        } else {
            serde_json::to_string(self)
        }
    }

    /// Get the line that should be displayed at a given time
    pub fn get_active_line(&self, time: f64) -> Option<&Line> {
        self.lines
            .iter()
            .find(|line| time >= line.start && time <= line.end)
    }

    /// Resolve a style by name, handling inheritance
    pub fn resolve_style(&self, name: &str) -> Style {
        let mut resolved = Style::default();

        if let Some(style) = self.styles.get(name) {
            // Handle inheritance
            if let Some(ref extends) = style.extends {
                resolved = self.resolve_style(extends);
            }
            // Merge current style
            merge_style(&mut resolved, style);
        }

        resolved
    }
}

/// Merge style properties (source overrides target)
fn merge_style(target: &mut Style, source: &Style) {
    if source.font.is_some() {
        target.font = source.font.clone();
    }
    if source.colors.is_some() {
        target.colors = source.colors.clone();
    }
    if source.stroke.is_some() {
        target.stroke = source.stroke.clone();
    }
    if source.shadow.is_some() {
        target.shadow = source.shadow.clone();
    }
    if source.glow.is_some() {
        target.glow = source.glow.clone();
    }
    if let Some(effects) = &source.effects {
        // Effects override or append?
        // Usually style inheritance overrides completely if present
        target.effects = Some(effects.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::project::Resolution;

    #[test]
    fn test_klyric_document_v2_from_json() {
        let json = r#"{
            "version": "2.0",
            "project": {
                "title": "Test Title",
                "duration": 120.0,
                "resolution": { "width": 1920, "height": 1080 }
            },
            "lines": []
        }"#;

        let doc = KLyricDocumentV2::from_json(json).unwrap();
        assert_eq!(doc.version, "2.0");
        assert_eq!(doc.project.title, "Test Title");
        assert_eq!(doc.project.duration, 120.0);
        assert_eq!(doc.project.resolution.width, 1920);
        assert_eq!(doc.project.resolution.height, 1080);
    }

    #[test]
    fn test_klyric_document_v2_to_json() {
        let doc = KLyricDocumentV2 {
            schema: None,
            version: "2.0".to_string(),
            project: Project {
                title: "Test".to_string(),
                artist: None,
                album: None,
                duration: 60.0,
                resolution: Resolution {
                    width: 1280,
                    height: 720,
                },
                fps: 30,
                audio: None,
                created: None,
                modified: None,
            },
            theme: None,
            styles: HashMap::new(),
            effects: HashMap::new(),
            lines: vec![],
        };

        let json = doc.to_json(false).unwrap();
        assert!(json.contains("\"version\":\"2.0\""));
        assert!(json.contains("\"title\":\"Test\""));

        let pretty_json = doc.to_json(true).unwrap();
        assert!(pretty_json.contains("\"version\": \"2.0\""));
    }

    #[test]
    fn test_get_active_line() {
        let mut doc = KLyricDocumentV2 {
            schema: None,
            version: "2.0".to_string(),
            project: Project {
                title: "Test".to_string(),
                artist: None,
                album: None,
                duration: 60.0,
                resolution: Resolution { width: 1920, height: 1080 },
                fps: 30,
                audio: None,
                created: None,
                modified: None,
            },
            theme: None,
            styles: HashMap::new(),
            effects: HashMap::new(),
            lines: vec![],
        };

        let line1 = Line {
            start: 1.0,
            end: 3.0,
            text: Some("First".to_string()),
            ..Default::default()
        };
        let line2 = Line {
            start: 4.0,
            end: 6.0,
            text: Some("Second".to_string()),
            ..Default::default()
        };
        doc.lines.push(line1);
        doc.lines.push(line2);

        assert!(doc.get_active_line(0.5).is_none());
        assert_eq!(doc.get_active_line(2.0).unwrap().text.as_deref(), Some("First"));
        assert!(doc.get_active_line(3.5).is_none());
        assert_eq!(doc.get_active_line(5.0).unwrap().text.as_deref(), Some("Second"));
        assert!(doc.get_active_line(7.0).is_none());
    }

    #[test]
    fn test_resolve_style() {
        let mut doc = KLyricDocumentV2 {
            schema: None,
            version: "2.0".to_string(),
            project: Project {
                title: "Test".to_string(),
                artist: None,
                album: None,
                duration: 60.0,
                resolution: Resolution { width: 1920, height: 1080 },
                fps: 30,
                audio: None,
                created: None,
                modified: None,
            },
            theme: None,
            styles: HashMap::new(),
            effects: HashMap::new(),
            lines: vec![],
        };

        let mut base_style = Style::default();
        base_style.font = Some(crate::model::style::Font {
            family: Some("Arial".to_string()),
            size: Some(24.0),
            weight: None,
            style: None,
            letter_spacing: None,
        });

        let mut inherited_style = Style::default();
        inherited_style.extends = Some("base".to_string());
        inherited_style.font = Some(crate::model::style::Font {
            family: None,
            size: Some(36.0),
            weight: None,
            style: None,
            letter_spacing: None,
        });

        doc.styles.insert("base".to_string(), base_style);
        doc.styles.insert("inherited".to_string(), inherited_style);

        let resolved = doc.resolve_style("inherited");
        assert_eq!(resolved.font.as_ref().unwrap().family.as_deref(), None); // Overridden logic: merge_style completely replaces font if target has font. Wait, `merge_style` says `if source.font.is_some() { target.font = source.font.clone(); }`. So the target's font is replaced entirely by the source font, fields are NOT merged recursively. Thus family will be None, and size will be 36.0.
        assert_eq!(resolved.font.as_ref().unwrap().size, Some(36.0));
    }
}

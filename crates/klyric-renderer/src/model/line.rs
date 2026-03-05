use serde::{Deserialize, Serialize};

use super::layout::{Layout, Position, Transform};
use super::style::{Font, Shadow, Stroke};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Line {
    /// Unique identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Line start time in seconds
    pub start: f64,

    /// Line end time in seconds
    pub end: f64,

    /// Full text (optional, can derive from chars)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    /// Style name to apply
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,

    /// Font override for this line
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font: Option<Font>,

    /// Stroke override for this line
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke: Option<Stroke>,

    /// Shadow override for this line
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shadow: Option<Shadow>,

    /// Effect names to apply
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub effects: Vec<String>,

    /// Line position
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,

    /// Line transform
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<Transform>,

    /// Text layout settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<Layout>,

    /// Characters with individual timing
    pub chars: Vec<Char>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Char {
    /// The character(s) to display
    pub char: String,

    /// Highlight start time in seconds
    pub start: f64,

    /// Highlight end time in seconds
    pub end: f64,

    /// Override style for this character
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,

    /// Font override for this character
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font: Option<Font>,

    /// Stroke override for this character
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke: Option<Stroke>,

    /// Shadow override for this character
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shadow: Option<Shadow>,

    /// Additional effects for this character
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub effects: Vec<String>,

    /// Character-specific transform
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<Transform>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_deserialization_defaults() {
        let json = r#"{
            "start": 0.0,
            "end": 5.0,
            "chars": []
        }"#;

        let line: Line = serde_json::from_str(json).unwrap();

        assert_eq!(line.start, 0.0);
        assert_eq!(line.end, 5.0);
        assert!(line.chars.is_empty());
        assert!(line.id.is_none());
        assert!(line.text.is_none());
        assert!(line.style.is_none());
        assert!(line.font.is_none());
        assert!(line.stroke.is_none());
        assert!(line.shadow.is_none());
        assert!(line.effects.is_empty());
        assert!(line.position.is_none());
        assert!(line.transform.is_none());
        assert!(line.layout.is_none());
    }

    #[test]
    fn test_char_deserialization_defaults() {
        let json = r#"{
            "char": "A",
            "start": 1.0,
            "end": 2.0
        }"#;

        let ch: Char = serde_json::from_str(json).unwrap();

        assert_eq!(ch.char, "A");
        assert_eq!(ch.start, 1.0);
        assert_eq!(ch.end, 2.0);
        assert!(ch.style.is_none());
        assert!(ch.font.is_none());
        assert!(ch.stroke.is_none());
        assert!(ch.shadow.is_none());
        assert!(ch.effects.is_empty());
        assert!(ch.transform.is_none());
    }

    #[test]
    fn test_line_with_chars() {
        let json = r#"{
            "start": 0.0,
            "end": 2.0,
            "text": "Hi",
            "chars": [
                { "char": "H", "start": 0.0, "end": 1.0 },
                { "char": "i", "start": 1.0, "end": 2.0 }
            ]
        }"#;

        let line: Line = serde_json::from_str(json).unwrap();
        assert_eq!(line.text.as_deref(), Some("Hi"));
        assert_eq!(line.chars.len(), 2);
        assert_eq!(line.chars[0].char, "H");
        assert_eq!(line.chars[1].char, "i");
    }
}

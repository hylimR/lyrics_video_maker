pub mod document;
pub mod project;
pub mod theme;
pub mod style;
pub mod effect;
pub mod layout;
pub mod line;

pub use document::KLyricDocumentV2;
pub use project::{Project, Resolution};
pub use theme::{Theme, Background, BackgroundType, Gradient, GradientType};
pub use style::{Style, Font, FontStyle, StateColors, FillStroke, Stroke, Shadow, Glow};
pub use effect::{Effect, EffectType, EffectTrigger, AnimatedValue, Keyframe, Easing, KaraokeMode, Direction};
pub use layout::{Position, PositionValue, Anchor, Transform, Layout, LayoutMode, Align, Justify};
pub use line::{Line, Char};

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_v2_document() {
        let json = r#"{
            "version": "2.0",
            "project": {
                "title": "Test Song",
                "duration": 180.0,
                "resolution": { "width": 1920, "height": 1080 }
            },
            "lines": [
                {
                    "start": 5.0,
                    "end": 10.0,
                    "chars": [
                        { "char": "H", "start": 5.0, "end": 5.5 },
                        { "char": "i", "start": 5.5, "end": 6.0 }
                    ]
                }
            ]
        }"#;
        
        let doc = KLyricDocumentV2::from_json(json).unwrap();
        assert_eq!(doc.version, "2.0");
        assert_eq!(doc.project.title, "Test Song");
        assert_eq!(doc.lines.len(), 1);
        assert_eq!(doc.lines[0].chars.len(), 2);
    }
}

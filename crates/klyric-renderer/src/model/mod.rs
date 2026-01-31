pub mod document;
pub mod effect;
pub mod layout;
pub mod line;
pub mod project;
pub mod style;
pub mod theme;

pub use document::KLyricDocumentV2;
pub use effect::{
    AnimatedValue, Direction, Easing, Effect, EffectTrigger, EffectType, KaraokeMode, Keyframe,
};
pub use layout::{Align, Anchor, Justify, Layout, LayoutMode, Position, PositionValue, Transform};
pub use line::{Char, Line};
pub use project::{Project, Resolution};
pub use style::{FillStroke, Font, FontStyle, Glow, Shadow, StateColors, Stroke, Style};
pub use theme::{Background, BackgroundType, Gradient, GradientType, Theme};

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

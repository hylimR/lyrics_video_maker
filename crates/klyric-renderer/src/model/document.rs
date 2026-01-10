use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::project::Project;
use super::theme::Theme;
use super::style::Style;
use super::effect::Effect;
use super::line::Line;

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
        self.lines.iter().find(|line| time >= line.start && time <= line.end)
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
}

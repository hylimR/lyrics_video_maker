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

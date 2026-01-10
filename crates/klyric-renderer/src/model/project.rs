use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    /// Song or project title
    pub title: String,
    
    /// Artist name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,
    
    /// Album name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album: Option<String>,
    
    /// Total duration in seconds
    pub duration: f64,
    
    /// Video resolution
    pub resolution: Resolution,
    
    /// Frames per second for export
    #[serde(default = "default_fps")]
    pub fps: u32,
    
    /// Path to audio file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<String>,
    
    /// Creation timestamp (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    
    /// Last modified timestamp (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<String>,
}

fn default_fps() -> u32 { 30 }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resolution {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
}

impl Default for Resolution {
    fn default() -> Self {
        Self { width: 1920, height: 1080 }
    }
}

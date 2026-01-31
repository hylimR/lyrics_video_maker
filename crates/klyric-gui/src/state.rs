//! Application State

use klyric_renderer::model::KLyricDocumentV2;
use std::path::PathBuf;

use iced::widget::image;

/// Global application state
#[derive(Default)]
pub struct AppState {
    /// Currently loaded document
    pub document: Option<KLyricDocumentV2>,
    
    /// Path to the currently loaded file
    pub file_path: Option<PathBuf>,
    
    /// Whether document has unsaved changes
    pub is_dirty: bool,
    
    /// Currently selected line index
    pub selected_line: Option<usize>,
    
    /// Currently selected character index (within line)
    pub selected_char: Option<usize>,
    
    /// Playback state
    pub playback: PlaybackState,
    
    /// Preview panel visibility
    pub show_preview: bool,
    
    /// Export panel visibility
    pub show_export: bool,
    
    /// Export progress (0.0 - 1.0)
    pub export_progress: Option<f32>,
    
    /// Window dimensions
    pub window_width: u32,
    pub window_height: u32,

    /// Rendered preview frame
    pub preview_handle: Option<image::Handle>,
    
    /// Currently selected effect name
    pub selected_effect: Option<String>,
    
    /// Connection to render worker
    pub worker_connection: Option<crate::worker::WorkerConnection>,

    /// Audio Manager for playback
    pub audio_manager: Option<crate::audio::AudioManager>,


    /// Show debug overlay
    pub show_debug: bool,

    /// App Configuration
    pub config: crate::config::AppConfig,
    
    /// Available system fonts
    pub available_fonts: Vec<crate::utils::font_loader::FontInfo>,
    
    /// Whether settings modal is open
    pub show_settings: bool,
    
    /// Whether initial font scan is done
    pub font_scan_complete: bool,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("document", &self.document)
            .field("file_path", &self.file_path)
            .field("is_dirty", &self.is_dirty)
            .field("show_debug", &self.show_debug)
            .finish()
    }
}

/// Playback state
#[derive(Debug, Default)]
pub struct PlaybackState {
    /// Current playback time in seconds
    pub current_time: f64,
    
    /// Whether playback is active
    pub is_playing: bool,
    
    /// Total duration in seconds
    pub duration: f64,
}

impl AppState {
    pub fn new() -> Self {
        let mut document = None;
        let mut file_path = None;
        let mut selected_char = None;
        let mut selected_line = None;

        // Try to load sample project
        // Check locations: ./public, ../public, ../../public
        let candidates = vec![
            std::path::PathBuf::from("samples/sample.klyric"),
            std::path::PathBuf::from("../samples/sample.klyric"),
            std::path::PathBuf::from("../../samples/sample.klyric"),
        ];

        for path in candidates {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(mut doc) = serde_json::from_str::<KLyricDocumentV2>(&content) {
                        log::info!("Loaded sample project from {:?}", path);
                        
                        // Check for sample.wav in the same directory
                        if let Some(parent) = path.parent() {
                            let wav_path = parent.join("sample.wav");
                            if wav_path.exists() {
                                if let Ok(abs_wav) = std::fs::canonicalize(&wav_path) {
                                     doc.project.audio = Some(abs_wav.to_string_lossy().to_string());
                                }
                            }
                        }

                        // Set initial selection
                        if !doc.lines.is_empty() {
                            selected_line = Some(0);
                            if !doc.lines[0].chars.is_empty() {
                                selected_char = Some(0);
                            }
                        }

                        if let Ok(abs_path) = std::fs::canonicalize(&path) {
                            file_path = Some(abs_path);
                        } else {
                            file_path = Some(path);
                        }
                        
                        document = Some(doc);
                        break;
                    }
                }
            }
        }

        // Create Audio Manager
        let mut audio_manager = crate::audio::AudioManager::new();

        // Load audio if sample project has it
        if let Some(doc) = &document {
            if let Some(audio_path) = &doc.project.audio {
                if let Some(am) = &mut audio_manager {
                    if let Err(e) = am.load(audio_path) {
                        log::error!("Failed to load sample audio: {}", e);
                    }
                }
            }
        }

        let playback = if let Some(doc) = &document {
            crate::state::PlaybackState {
                duration: doc.project.duration,
                ..Default::default()
            }
        } else {
            Default::default()
        };

        Self {
            document,
            file_path,
            selected_line,
            selected_char,
            show_preview: true,
            window_width: 1400,
            window_height: 900,
            worker_connection: Some(crate::worker::spawn()),
            audio_manager,
            playback,
            show_debug: false,
            config: crate::config::AppConfig::load(),
            show_settings: false,
            available_fonts: Vec::new(),
            ..Default::default()
        }
    }
    
    /// Get the currently selected line
    pub fn current_line(&self) -> Option<&klyric_renderer::model::Line> {
        let doc = self.document.as_ref()?;
        let idx = self.selected_line?;
        doc.lines.get(idx)
    }
    
    /// Get the currently selected character
    pub fn current_char(&self) -> Option<&klyric_renderer::model::Char> {
        let line = self.current_line()?;
        let idx = self.selected_char?;
        line.chars.get(idx)
    }
    
    /// Get mutable reference to current line
    pub fn current_line_mut(&mut self) -> Option<&mut klyric_renderer::model::Line> {
        let idx = self.selected_line?;
        self.document.as_mut()?.lines.get_mut(idx)
    }
    
    /// Get mutable reference to current char
    pub fn current_char_mut(&mut self) -> Option<&mut klyric_renderer::model::Char> {
        let idx = self.selected_char?;
        self.current_line_mut()?.chars.get_mut(idx)
    }
}





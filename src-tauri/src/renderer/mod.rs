//! Renderer Module
//!
//! Contains text rendering utilities.
//! The main KLyric v2 renderer is provided by the `klyric_renderer` crate.

pub mod text;

// Re-export the shared klyric-renderer crate types for convenience
pub use klyric_renderer::model::{KLyricDocumentV2, Line, Char};
pub use klyric_renderer::Renderer;

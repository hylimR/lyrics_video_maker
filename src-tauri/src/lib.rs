//! Lyric Video Maker - Rust Backend Library
//!
//! This library provides high-performance video rendering capabilities
//! for the Lyric Video Maker application using Tauri.

pub mod commands;
pub mod renderer;
pub mod video;

// Re-export commonly used types from the shared klyric-renderer crate
pub use klyric_renderer::model::{KLyricDocumentV2, Line, Char};
pub use klyric_renderer::Renderer;

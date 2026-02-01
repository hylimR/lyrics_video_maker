//! KLyric Renderer - Pure Rust rendering library for KLyric v2.0 format
//!
//! This crate provides a high-performance native renderer using skia-safe for
//! GPU-accelerated lyric video rendering.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────┐
//! │  KLyric v2.0    │
//! │  JSON Document  │
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │     Parser      │ → model.rs
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │ Style Resolver  │ → style.rs
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │ Layout Engine   │ → layout.rs
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │ Effect Engine   │ → effects.rs
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │   Renderer      │ → renderer.rs (skia-safe)
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │  RGBA Pixels    │
//! └─────────────────┘
//! ```


pub mod model;
pub mod parser;
pub mod importer;
pub mod style;
pub mod effects;
pub mod layout;
pub mod renderer;
pub mod text;
pub mod particle;
pub mod presets;
pub mod utils;

pub mod expressions;

// Re-exports for convenience
pub use model::*;
pub use parser::parse_document;
pub use renderer::Renderer;
pub use text::TextRenderer;

pub use particle::{Particle, ParticleEmitter, ParticleConfig};
pub use presets::{EffectPreset, PresetFactory};


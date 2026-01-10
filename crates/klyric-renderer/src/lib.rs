//! KLyric Renderer - Pure Rust rendering library for KLyric v2.0 format
//!
//! This crate provides a unified renderer that can be compiled to:
//! - Native Rust for high-performance video export
//! - WebAssembly for browser-based preview
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
//! │   Rasterizer    │ → renderer.rs
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │  RGBA Pixels    │
//! └─────────────────┘
//! ```

pub mod model;
pub mod parser;
pub mod style;
pub mod effects;
pub mod layout;
pub mod renderer;
pub mod text;
pub mod particle;
pub mod presets;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

// Re-exports for convenience
pub use tiny_skia;
pub use model::*;
pub use parser::parse_document;
pub use renderer::Renderer;
pub use particle::{Particle, ParticleEmitter, ParticleConfig};
pub use presets::{EffectPreset, PresetFactory};
pub use text::TextRenderer;


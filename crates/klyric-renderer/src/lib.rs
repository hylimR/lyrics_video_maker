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

pub mod effects;
pub mod importer;
pub mod layout;
pub mod model;
pub mod parser;
pub mod particle;
pub mod presets;
#[cfg(not(target_arch = "wasm32"))]
pub mod renderer;
pub mod style;
#[cfg(not(target_arch = "wasm32"))]
pub mod text;

#[cfg(target_arch = "wasm32")]
pub mod wasm_renderer;

pub mod expressions;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(target_arch = "wasm32")]
pub use wasm_renderer as renderer;
#[cfg(target_arch = "wasm32")]
pub use wasm_renderer as text;

// Re-exports for convenience
pub use model::*;
pub use parser::parse_document;
#[cfg(not(target_arch = "wasm32"))]
pub use renderer::Renderer;
#[cfg(not(target_arch = "wasm32"))]
pub use text::TextRenderer;

#[cfg(target_arch = "wasm32")]
pub use wasm_renderer::Renderer;
#[cfg(target_arch = "wasm32")]
pub use wasm_renderer::TextRenderer;

pub use particle::{Particle, ParticleConfig, ParticleEmitter};
pub use presets::{EffectPreset, PresetFactory};

//! Particle System for KLyric Effects
//!
//! Provides particle emission, physics simulation, and lifecycle management
//! for visual effects like rain, sparkle, disintegrate, etc.
//!
//! # Module Structure
//! - `types` - Core Particle struct and color utilities
//! - `physics` - Physics simulation parameters
//! - `config` - Configuration, spawn patterns, and range values
//! - `emitter` - ParticleEmitter for spawning/managing particles
//! - `rng` - Deterministic random number generator

pub mod types;
pub mod physics;
pub mod config;
pub mod emitter;
pub mod rng;

// Re-exports for convenience
pub use types::{Particle, ParticleShape, BlendMode, color_to_rgba, parse_hex_color};
pub use physics::ParticlePhysics;
pub use config::{ParticleConfig, RangeValue, SpawnPattern};
pub use emitter::ParticleEmitter;
pub use rng::Rng;

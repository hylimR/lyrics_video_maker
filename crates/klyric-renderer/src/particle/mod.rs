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

pub mod config;
pub mod emitter;
pub mod physics;
pub mod rng;
pub mod types;

// Re-exports for convenience
pub use config::{ParticleConfig, RangeValue, SpawnPattern};
pub use emitter::ParticleEmitter;
pub use physics::ParticlePhysics;
pub use rng::Rng;
pub use types::{color_to_rgba, parse_hex_color, BlendMode, Particle, ParticleShape};

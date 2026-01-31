use super::types::CharBounds;
use crate::particle::ParticleEmitter;

/// Trait for defining particle effect presets
///
/// Implementing this trait allows creating custom particle effects
/// that can be registered with the PresetFactory.
pub trait ParticlePreset: Send + Sync {
    /// Create a new emitter instance for this preset
    fn create_emitter(&self, bounds: &CharBounds, seed: u64) -> ParticleEmitter;
}

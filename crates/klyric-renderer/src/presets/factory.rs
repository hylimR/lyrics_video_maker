use std::collections::HashMap;
use crate::particle::ParticleEmitter;
use super::types::{CharBounds, EffectPreset};
use super::traits::ParticlePreset;
use super::particles::*;

/// Factory for creating preset particle effects
/// 
/// Maintains a registry of available presets allowing for extension.
pub struct PresetFactory {
    registry: HashMap<String, Box<dyn ParticlePreset>>,
}

impl PresetFactory {
    /// Create a new factory with default presets registered
    pub fn new() -> Self {
        let mut factory = Self {
            registry: HashMap::new(),
        };
        
        // Register default presets
        factory.register("rain", Box::new(rain::RainPreset));
        factory.register("sparkle", Box::new(sparkle::SparklePreset));
        factory.register("hearts", Box::new(hearts::HeartsPreset));
        factory.register("confetti", Box::new(confetti::ConfettiPreset));
        factory.register("disintegrate", Box::new(disintegrate::DisintegratePreset));
        factory.register("fire", Box::new(fire::FirePreset));
        factory.register("glow", Box::new(glow::GlowPulsePreset));
        factory.register("glowpulse", Box::new(glow::GlowPulsePreset));
        
        factory
    }

    /// Register a new preset implementation
    pub fn register(&mut self, name: &str, preset: Box<dyn ParticlePreset>) {
        self.registry.insert(name.to_lowercase(), preset);
    }

    /// Create a particle emitter for the given preset enum (legacy support)
    pub fn create_from_enum(
        &self,
        preset: EffectPreset,
        bounds: &CharBounds,
        seed: u64,
    ) -> ParticleEmitter {
        let key = match preset {
            EffectPreset::Rain => "rain",
            EffectPreset::Sparkle => "sparkle",
            EffectPreset::Hearts => "hearts",
            EffectPreset::Confetti => "confetti",
            EffectPreset::Disintegrate => "disintegrate",
            EffectPreset::Fire => "fire",
            EffectPreset::GlowPulse => "glow",
        };
        
        self.create(key, bounds, seed)
            .unwrap_or_else(|| {
                // Fallback if registry is somehow broken for built-ins
                // This shouldn't happen if initialized correctly
                rain::RainPreset.create_emitter(bounds, seed)
            })
    }
    
    /// Create a particle emitter by name
    pub fn create(
        &self,
        name: &str,
        bounds: &CharBounds,
        seed: u64,
    ) -> Option<ParticleEmitter> {
        self.registry.get(&name.to_lowercase())
            .map(|p| p.create_emitter(bounds, seed))
    }
}

impl Default for PresetFactory {
    fn default() -> Self {
        Self::new()
    }
}

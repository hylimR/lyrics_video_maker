pub mod types;
pub mod impls;
pub mod factory;
pub mod traits;

pub use types::{EffectPreset, CharBounds};
pub use factory::PresetFactory;
pub use traits::ParticlePreset;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_from_str() {
        assert_eq!(EffectPreset::from_str("rain"), Some(EffectPreset::Rain));
        assert_eq!(EffectPreset::from_str("SPARKLE"), Some(EffectPreset::Sparkle));
        assert_eq!(EffectPreset::from_str("unknown"), None);
    }

    #[test]
    fn test_preset_factory() {
        let bounds = CharBounds {
            x: 100.0,
            y: 100.0,
            width: 50.0,
            height: 72.0,
        };

        let factory = PresetFactory::new();
        let mut emitter = factory.create_from_enum(EffectPreset::Rain, &bounds, 42);
        assert!(emitter.particles.is_empty());
        
        // Update to trigger spawn
        emitter.update(0.5);
        assert!(!emitter.particles.is_empty());
    }

    #[test]
    fn test_burst_effects() {
        let bounds = CharBounds {
            x: 0.0, y: 0.0, width: 100.0, height: 100.0,
        };

        let factory = PresetFactory::new();

        let mut sparkle = factory.create_from_enum(EffectPreset::Sparkle, &bounds, 42);
        sparkle.burst();
        assert_eq!(sparkle.particles.len(), 12);

        let mut disintegrate = factory.create_from_enum(EffectPreset::Disintegrate, &bounds, 42);
        disintegrate.burst();
        assert_eq!(disintegrate.particles.len(), 30);
    }
}

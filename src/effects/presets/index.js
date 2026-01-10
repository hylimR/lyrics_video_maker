/**
 * Effect Presets Index
 * 
 * Central export for all effect presets.
 */
export * from './basicEffects';
export * from './advancedEffects';
export * from './utils';

// Import all effects for the PRESETS object
import {
    blurEffect,
    wobblyEffect,
    scalePopEffect,
    colorShiftEffect,
    pulseGlowEffect,
} from './basicEffects';

import {
    particleExplosionEffect,
    sparkleTrailEffect,
    flip3DEffect,
    wave3DEffect,
    typewriterEffect,
    neonGlowEffect,
    shatterEffect,
} from './advancedEffects';

/**
 * All effect presets as a map
 */
export const PRESETS = {
    blur: blurEffect,
    wobbly: wobblyEffect,
    scalePop: scalePopEffect,
    colorShift: colorShiftEffect,
    pulseGlow: pulseGlowEffect,
    particleExplosion: particleExplosionEffect,
    sparkleTrail: sparkleTrailEffect,
    flip3D: flip3DEffect,
    wave3D: wave3DEffect,
    typewriter: typewriterEffect,
    neonGlow: neonGlowEffect,
    shatter: shatterEffect,
};

export default PRESETS;

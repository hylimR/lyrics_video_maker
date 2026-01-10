/**
 * EffectManager.js - Per-Character Timeline-Based Effect System
 * 
 * KEY CONCEPT: All effects are per-glyph and timeline-driven.
 * As the karaoke progress (0→1) sweeps across the text, each glyph
 * receives its animation when the progress reaches its position.
 * 
 * Glyph Animation Lifecycle:
 * ┌─────────────────────────────────────────────────────────────┐
 * │  Progress: 0%        25%        50%        75%       100%  │
 * │            ↓          ↓          ↓          ↓          ↓   │
 * │           [H]       [e]        [l]        [l]        [o]   │
 * │            ↑          ↑          ↑          ↑          ↑   │
 * │         animate    animate    animate    animate   animate │
 * └─────────────────────────────────────────────────────────────┘
 */
import { ParticleSystem } from './ParticleSystem';
import { PRESETS } from './presets';

/**
 * EffectManager class - Manages visual effects for lyric lines
 */
export class EffectManager {
    constructor(lyricRenderer) {
        this.lyricRenderer = lyricRenderer;
        this.presets = { ...PRESETS };
        this.activeEffects = new Map();
        this.defaultPreset = 'blur';
        this.globalOptions = {};

        // Initialize particle system
        this.particleSystem = new ParticleSystem(lyricRenderer.app);

        // Context object passed to effects
        this.context = {
            particleSystem: this.particleSystem,
        };
    }

    /**
     * Register a custom effect preset
     * @param {string} name - Unique name for the preset
     * @param {Object} preset - Effect preset configuration
     */
    registerPreset(name, preset) {
        this.presets[name] = {
            name,
            options: preset.options || {},
            onEnter: preset.onEnter || (() => { }),
            onExit: preset.onExit || (() => { }),
            onUpdate: preset.onUpdate || (() => { }),
        };
        console.log(`✨ Effect preset registered: ${name}`);
    }

    /**
     * Set the default effect preset
     * @param {string} name - Preset name
     * @param {Object} options - Override options
     */
    setDefaultPreset(name, options = {}) {
        if (!this.presets[name]) {
            console.warn(`Effect preset "${name}" not found`);
            return;
        }
        this.defaultPreset = name;
        this.globalOptions = options;
        console.log(`🎨 Default effect set to: ${name}`);
    }

    /**
     * Apply a preset to a specific line
     * @param {string} presetName - Name of the preset
     * @param {number} lineIndex - Line index
     * @param {Object} options - Override options
     * @returns {string} Effect key
     */
    applyPreset(presetName, lineIndex, options = {}) {
        const line = this.lyricRenderer.getLine(lineIndex);
        if (!line) return;

        const preset = this.presets[presetName];
        if (!preset) {
            console.warn(`Effect preset "${presetName}" not found`);
            return;
        }

        const mergedOptions = { ...preset.options, ...options };
        const effectKey = `${lineIndex}-${presetName}`;

        this.activeEffects.set(effectKey, {
            preset,
            options: mergedOptions,
            state: {},
        });

        return effectKey;
    }

    /**
     * Called when a line becomes active
     * @param {Object} line - Lyric line object
     * @param {number} lineIndex - Line index
     */
    onLineEnter(line, lineIndex) {
        const effectKey = `${lineIndex}-${this.defaultPreset}`;

        if (!this.activeEffects.has(effectKey)) {
            this.applyPreset(this.defaultPreset, lineIndex, this.globalOptions);
        }

        const effect = this.activeEffects.get(effectKey);
        if (effect) {
            effect.state = effect.preset.onEnter(line, effect.options, this.context) || {};
        }
    }

    /**
     * Called when a line exits (becomes inactive)
     * @param {Object} line - Lyric line object
     * @param {number} lineIndex - Line index
     */
    onLineExit(line, lineIndex) {
        const effectKey = `${lineIndex}-${this.defaultPreset}`;
        const effect = this.activeEffects.get(effectKey);

        if (effect) {
            effect.preset.onExit(line, effect.options, effect.state, this.context);
        }
    }

    /**
     * Called every frame for active lines
     * @param {Object} line - Lyric line object
     * @param {number} lineIndex - Line index
     * @param {number} progress - Progress 0-1 through the line
     * @param {number} currentTime - Current playback time
     */
    onUpdate(line, lineIndex, progress, currentTime) {
        const effectKey = `${lineIndex}-${this.defaultPreset}`;
        const effect = this.activeEffects.get(effectKey);

        if (effect) {
            effect.preset.onUpdate(line, effect.options, effect.state, progress, currentTime, this.context);
        }
    }

    /**
     * Clear effects for a specific line
     * @param {number} lineIndex - Line index
     */
    clearEffects(lineIndex) {
        for (const [key] of this.activeEffects) {
            if (key.startsWith(`${lineIndex}-`)) {
                this.activeEffects.delete(key);
            }
        }
    }

    /**
     * Get list of available preset names
     * @returns {string[]} Array of preset names
     */
    getPresetNames() {
        return Object.keys(this.presets);
    }

    /**
     * Cleanup resources
     */
    destroy() {
        this.activeEffects.clear();
        if (this.particleSystem) {
            this.particleSystem.destroy();
        }
    }
}

export default EffectManager;

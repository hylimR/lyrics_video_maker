/**
 * Application Constants
 * 
 * Centralized configuration for the Lyric Video Maker application.
 * Import these constants instead of hardcoding values in components.
 */

// ==================== VISUAL EFFECTS ====================

/**
 * Available effect presets organized by category
 */
export const EFFECT_PRESETS = [
    // Basic Effects
    { value: 'blur', label: '✨ Blur Fade', category: 'Basic' },
    { value: 'wobbly', label: '🌊 Wobbly Text', category: 'Basic' },
    { value: 'scalePop', label: '💥 Scale Pop', category: 'Basic' },
    { value: 'colorShift', label: '🌈 Color Shift', category: 'Basic' },
    { value: 'pulseGlow', label: '💫 Pulse Glow', category: 'Basic' },
    // Advanced Effects
    { value: 'particleExplosion', label: '🎆 Particle Explosion', category: 'Advanced' },
    { value: 'sparkleTrail', label: '✨ Sparkle Trail', category: 'Advanced' },
    { value: 'flip3D', label: '🔄 3D Flip', category: 'Advanced' },
    { value: 'wave3D', label: '🌀 3D Wave', category: 'Advanced' },
    { value: 'typewriter', label: '⌨️ Typewriter', category: 'Advanced' },
    { value: 'neonGlow', label: '💜 Neon Glow', category: 'Advanced' },
    { value: 'shatter', label: '💔 Shatter', category: 'Advanced' },
];

/**
 * Effect categories for filtering
 */
export const EFFECT_CATEGORIES = {
    BASIC: 'Basic',
    ADVANCED: 'Advanced',
};

/**
 * Get effects by category
 */
export const getEffectsByCategory = (category) =>
    EFFECT_PRESETS.filter((p) => p.category === category);

// ==================== RESOLUTION PRESETS ====================

/**
 * Available resolution presets
 */
export const RESOLUTION_PRESETS = {
    // Landscape
    '4K': { width: 3840, height: 2160, label: '4K UHD (3840×2160)', category: 'Landscape' },
    'FHD': { width: 1920, height: 1080, label: 'Full HD (1920×1080)', category: 'Landscape' },
    'HD': { width: 1280, height: 720, label: 'HD (1280×720)', category: 'Landscape' },
    'SD': { width: 854, height: 480, label: 'SD (854×480)', category: 'Landscape' },
    // Vertical / Square
    'VERTICAL': { width: 1080, height: 1920, label: 'Vertical (1080×1920)', category: 'Portrait' },
    'SQUARE': { width: 1080, height: 1080, label: 'Square (1080×1080)', category: 'Portrait' },
};

/**
 * Get resolution preset by key string "WIDTHxHEIGHT"
 */
export const getResolutionPreset = (key) => {
    const [width, height] = key.split('x').map(Number);
    return Object.values(RESOLUTION_PRESETS).find(
        (r) => r.width === width && r.height === height
    ) || RESOLUTION_PRESETS.FHD;
};

/**
 * Format resolution as string key
 */
export const formatResolutionKey = (resolution) =>
    `${resolution.width}x${resolution.height}`;

// ==================== FILE TYPES ====================

/**
 * Supported lyric file extensions
 */
export const LYRIC_EXTENSIONS = ['.lrc', '.srt', '.ass', '.ssa', '.klyric', '.json'];

/**
 * KLyric format version
 */
export const KLYRIC_VERSION = '1.0';

/**
 * Supported audio file extensions
 */
export const AUDIO_EXTENSIONS = ['.mp3', '.wav', '.ogg', '.m4a', '.flac', '.aac'];

/**
 * Check if file extension is a lyric file
 */
export const isLyricFile = (filename) => {
    const ext = '.' + filename.split('.').pop().toLowerCase();
    return LYRIC_EXTENSIONS.includes(ext);
};

/**
 * Check if file extension is an audio file
 */
export const isAudioFile = (filename) => {
    const ext = '.' + filename.split('.').pop().toLowerCase();
    return AUDIO_EXTENSIONS.includes(ext);
};

// ==================== SYNC & TIMING ====================

/**
 * BroadcastChannel configuration
 */
export const SYNC_CONFIG = {
    CHANNEL_NAME: 'lyric-video-state-sync',
    HEARTBEAT_INTERVAL: 2000, // ms
    MASTER_TIMEOUT: 5000, // ms
    PLAYBACK_SYNC_FPS: 10, // frames per second for playback sync
};

/**
 * History configuration
 */
export const HISTORY_CONFIG = {
    MAX_UNDO_STEPS: 50,
};

// ==================== TYPOGRAPHY ====================

/**
 * Chinese-compatible font stacks
 */
export const FONT_FAMILIES = {
    PRIMARY: '"Noto Sans SC", "Microsoft YaHei", "PingFang SC", "Hiragino Sans GB", Inter, Arial, sans-serif',
    MONOSPACE: '"JetBrains Mono", "Fira Code", Consolas, monospace',
};

/**
 * Default text styles for rendering
 */
export const TEXT_STYLES = {
    DEFAULT_FONT_SIZE: 48,
    INACTIVE_COLOR: '#ffffff',
    ACTIVE_COLOR: '#00ffff',
};

// ==================== KEYBOARD SHORTCUTS ====================

/**
 * Global keyboard shortcuts
 */
export const KEYBOARD_SHORTCUTS = {
    UNDO: { key: 'z', ctrl: true, shift: false },
    REDO: { key: 'z', ctrl: true, shift: true },
    REDO_ALT: { key: 'y', ctrl: true, shift: false },
    PLAY_PAUSE: { key: 'p', ctrl: false, shift: false },
    SEEK_BACK: { key: 'ArrowLeft', ctrl: false, shift: false },
    SEEK_FORWARD: { key: 'ArrowRight', ctrl: false, shift: false },
    SEEK_BACK_FAST: { key: 'ArrowLeft', ctrl: false, shift: true },
    SEEK_FORWARD_FAST: { key: 'ArrowRight', ctrl: false, shift: true },
};

/**
 * K-Timing Editor specific shortcuts
 */
export const KTIMING_SHORTCUTS = {
    MARK: ['Space', 'k', 'K'],
    UNDO_MARK: 'Backspace',
    APPLY: 'Enter',
    CLOSE: 'Escape',
    AUTO_SPLIT: ['a', 'A'],
    RESTART: ['r', 'R'],
    TOGGLE_LOOP: ['l', 'L'],
};

// ==================== DEFAULTS ====================

/**
 * Default application state values
 */
export const DEFAULTS = {
    RESOLUTION: { width: 1920, height: 1080 },
    EFFECT: 'blur',
    DURATION: 28,
    DEMO_AUDIO_FILES: ['/wav_泡沫.wav', '/sample.wav'],
    DEMO_ASS_FILE: '/sample_karaoke.ass',
};

/**
 * Effect Utilities
 * 
 * Helper functions for calculating glyph progress and timing.
 */

/**
 * Calculate which glyph index should be active based on progress
 * @param {number} progress - Overall progress 0-1
 * @param {number} totalGlyphs - Total number of characters
 * @returns {number} Active glyph index
 */
export function getActiveGlyphIndex(progress, totalGlyphs) {
    return Math.floor(progress * totalGlyphs);
}

/**
 * Calculate per-glyph progress (0-1 for each character)
 * @param {number} progress - Overall progress 0-1
 * @param {number} glyphIndex - Index of the glyph
 * @param {number} totalGlyphs - Total number of characters
 * @returns {number} Glyph-specific progress 0-1
 */
export function getGlyphProgress(progress, glyphIndex, totalGlyphs) {
    const glyphStart = glyphIndex / totalGlyphs;
    const glyphEnd = (glyphIndex + 1) / totalGlyphs;

    if (progress < glyphStart) return 0;
    if (progress >= glyphEnd) return 1;

    return (progress - glyphStart) / (glyphEnd - glyphStart);
}

/**
 * Clamp a value between min and max
 * @param {number} value - Value to clamp
 * @param {number} min - Minimum value
 * @param {number} max - Maximum value
 * @returns {number} Clamped value
 */
export function clamp(value, min, max) {
    return Math.min(Math.max(value, min), max);
}

/**
 * Linear interpolation
 * @param {number} a - Start value
 * @param {number} b - End value
 * @param {number} t - Interpolation factor 0-1
 * @returns {number} Interpolated value
 */
export function lerp(a, b, t) {
    return a + (b - a) * t;
}

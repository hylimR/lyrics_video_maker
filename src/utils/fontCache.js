/**
 * Global cache for font data (Uint8Array) to avoid repeated disk reads/IPC calls.
 * Implements an LRU (Least Recently Used) eviction policy to manage memory usage.
 *
 * Key: Font Name (string) or Path (string)
 * Value: Uint8Array
 */

const MAX_CACHE_SIZE = 10; // Limit to 10 fonts to prevent excessive memory usage (e.g. ~200MB)
const fontCache = new Map();

/**
 * Get cached font data
 * Marks the accessed item as most recently used.
 * @param {string} key - Font name or path
 * @returns {Uint8Array|undefined}
 */
export const getCachedFont = (key) => {
    if (fontCache.has(key)) {
        const value = fontCache.get(key);
        // Delete and re-set to move to end (MRU position)
        fontCache.delete(key);
        fontCache.set(key, value);
        return value;
    }
    return undefined;
};

/**
 * Cache font data
 * Evicts the least recently used item if cache is full.
 * @param {string} key - Font name or path
 * @param {Uint8Array} data - Font data
 */
export const cacheFont = (key, data) => {
    if (!(data instanceof Uint8Array)) {
        console.warn('cacheFont: data must be Uint8Array');
        return;
    }

    // If updating existing, remove first
    if (fontCache.has(key)) {
        fontCache.delete(key);
    } else if (fontCache.size >= MAX_CACHE_SIZE) {
        // Evict LRU (first item in Map)
        const firstKey = fontCache.keys().next().value;
        if (firstKey) {
            console.log(`🧹 Evicting font from cache: ${firstKey}`);
            fontCache.delete(firstKey);
        }
    }

    fontCache.set(key, data);
};

/**
 * Check if font is cached
 * Does NOT update LRU status.
 * @param {string} key - Font name or path
 * @returns {boolean}
 */
export const hasCachedFont = (key) => {
    return fontCache.has(key);
};

/**
 * Clear the font cache
 */
export const clearFontCache = () => {
    fontCache.clear();
};

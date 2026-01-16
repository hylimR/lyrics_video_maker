/**
 * KLyricFormat.js - KLyric Format v2.0 Parser and Converter
 * 
 * This module provides utilities for:
 * - Converting ASS/SRT/LRC to KLyric format
 * - Parsing KLyric JSON files
 * - Serializing KLyric data to JSON
 * - Converting KLyric back to ASS for export
 * 
 * The KLyric format is a rich, JSON-based lyric format that supports:
 * - Per-character transforms (position, rotation, scale, opacity)
 * - CSS-like styling with inheritance
 * - Keyframe animations
 * - Individual character timing
 * - Parent-child hierarchies
 * - Layout modes (horizontal, vertical, path)
 */

import { parseLRC, parseSRT, parseASS } from './lyricParsers.js';

// --- Constants ---

const KLYRIC_VERSION = '2.0';

// v2.0 uses FillStroke objects for colors
const DEFAULT_STYLE = {
    font: {
        family: 'Noto Sans SC',
        size: 72,
        weight: 700,
        style: 'normal',
        letterSpacing: 0
    },
    colors: {
        inactive: { fill: '#888888' },
        active: { fill: '#FFFF00' },
        complete: { fill: '#FFFFFF' }
    },
    stroke: {
        width: 3,
        color: '#000000'
    },
    shadow: {
        color: 'rgba(0,0,0,0.5)',
        x: 2,
        y: 2,
        blur: 4
    }
};

const DEFAULT_EFFECT = {
    fadeIn: {
        type: 'transition',
        trigger: 'enter',
        duration: 0.5,
        easing: 'easeOutCubic',
        properties: {
            opacity: { from: 0.0, to: 1.0 },
            y: { from: 50.0, to: 0.0 }
        }
    }
};

// --- ID Generation ---

let idCounter = 0;

/**
 * Generate a unique ID for KLyric elements
 * @param {string} prefix - Prefix for the ID
 * @returns {string} Unique ID
 */
function generateId(prefix = 'kl') {
    idCounter++;
    return `${prefix}_${Date.now().toString(36)}_${idCounter.toString(36)}`;
}

/**
 * Reset ID counter (useful for tests)
 */
function resetIdCounter() {
    idCounter = 0;
}

// --- Conversion Functions ---

/**
 * Convert parsed lyrics (from ASS/SRT/LRC) to KLyric v2.0 format
 * 
 * @param {Array} lyrics - Parsed lyrics array from lyricParsers
 * @param {Object} metadata - Optional metadata from parser
 * @param {Object} options - Conversion options
 * @returns {Object} KLyric v2.0 document
 */
export function lyricsToKLyric(lyrics, metadata = {}, options = {}) {
    const {
        resolution = { width: 1920, height: 1080 },
        duration: optDuration,
        preserveRaw = true,
        sourceFormat = 'unknown'
    } = options;

    // Calculate total duration
    const duration = optDuration || (lyrics.length > 0
        ? Math.max(...lyrics.map(l => l.endTime)) + 2
        : 0);

    // Create the KLyric v2.0 document
    const doc = {
        version: KLYRIC_VERSION,
        project: {
            title: metadata.title || metadata.ti || 'Untitled',
            artist: metadata.artist || metadata.ar || '',
            duration,
            resolution,
            fps: 30,
            created: new Date().toISOString(),
            modified: new Date().toISOString(),
            source: sourceFormat
        },
        theme: {
            background: {
                type: 'solid',
                color: '#000000',
                opacity: 1.0
            }
        },
        styles: {
            base: { ...DEFAULT_STYLE }
        },
        effects: { ...DEFAULT_EFFECT },
        lines: lyrics.map((lyric, lineIndex) => convertLineToKLyricV2(lyric, lineIndex))
    };

    return doc;
}

/**
 * Convert a single lyric line to KLyric v2.0 line format
 * 
 * @param {Object} lyric - Single lyric line from parser
 * @param {number} lineIndex - Index of this line
 * @returns {Object} KLyric v2.0 line object
 */
function convertLineToKLyricV2(lyric, lineIndex) {
    const { text, startTime, endTime, syllables } = lyric;

    // Generate character timing
    const textChars = [...text]; // Handle multi-byte characters correctly
    let charData = [];

    if (syllables && syllables.length > 0) {
        // Use syllable-based timing
        for (const syllable of syllables) {
            const syllableChars = [...syllable.text];
            const charDuration = syllable.duration / syllableChars.length;

            for (let i = 0; i < syllableChars.length; i++) {
                const char = syllableChars[i];
                const charStart = startTime + syllable.startOffset + (i * charDuration);
                const charEnd = charStart + charDuration;

                charData.push({
                    char,
                    start: Math.round(charStart * 1000) / 1000,
                    end: Math.round(charEnd * 1000) / 1000
                });
            }
        }
    } else {
        // Estimated timing (even distribution)
        const totalDuration = endTime - startTime;
        const charDuration = totalDuration / textChars.length;

        charData = textChars.map((char, i) => ({
            char,
            start: Math.round((startTime + i * charDuration) * 1000) / 1000,
            end: Math.round((startTime + (i + 1) * charDuration) * 1000) / 1000
        }));
    }

    // Apply any existing charCustomizations if present
    if (lyric.charCustomizations) {
        Object.entries(lyric.charCustomizations).forEach(([charIndexStr, customization]) => {
            const charIdx = parseInt(charIndexStr, 10);
            if (charData[charIdx]) {
                charData[charIdx].transform = {
                    x: customization.offsetX || 0,
                    y: customization.offsetY || 0,
                    rotation: customization.rotation || 0,
                    scale: customization.scale || 1,
                    scaleX: 1,
                    scaleY: 1,
                    opacity: 1
                };
                if (customization.effect) {
                    charData[charIdx].effects = [customization.effect];
                }
            }
        });
    }

    return {
        id: `line-${lineIndex}`,
        start: Math.round(startTime * 1000) / 1000,
        end: Math.round(endTime * 1000) / 1000,
        text,
        style: 'base',
        effects: ['fadeIn'],
        position: {
            x: 960,  // Center of 1920
            y: 540,  // Center of 1080
            anchor: 'center'
        },
        chars: charData
    };
}

/**
 * Convert KLyric document back to the legacy lyrics format
 * (For compatibility with existing LyricRenderer)
 * 
 * @param {Object} klyricDoc - KLyric document
 * @returns {Array} Legacy lyrics array
 */
export function klyricToLegacy(klyricDoc) {
    if (!klyricDoc || !klyricDoc.lines) {
        return [];
    }

    return klyricDoc.lines.map(line => {
        // Reconstruct text from characters
        const text = line.chars.map(c => c.char).join('');

        // Reconstruct syllables from character timing
        const syllables = reconstructSyllables(line.chars, line.startTime);

        // Extract char customizations
        const charCustomizations = {};
        line.chars.forEach((char, index) => {
            if (char.transform || char.effects || char.animations) {
                charCustomizations[index] = {
                    offsetX: char.transform?.offsetX || 0,
                    offsetY: char.transform?.offsetY || 0,
                    scale: char.transform?.scale || 1,
                    rotation: char.transform?.rotation || 0,
                    effect: char.effects?.[0]?.ref || null
                };
            }
        });

        return {
            text,
            startTime: line.startTime,
            endTime: line.endTime,
            syllables,
            charCustomizations: Object.keys(charCustomizations).length > 0
                ? charCustomizations
                : undefined
        };
    });
}

/**
 * Reconstruct syllables from KLyric character data
 * Groups consecutive characters into syllables when they share timing patterns
 * 
 * @param {Array} chars - KLyric character array
 * @param {number} lineStartTime - Line start time
 * @returns {Array} Syllables array
 */
function reconstructSyllables(chars, lineStartTime) {
    if (!chars || chars.length === 0) return [];

    // Group characters by their _syllableIndex if present, or create individual syllables
    const syllables = [];
    let currentSyllable = null;
    let lastSyllableIndex = null;

    for (const char of chars) {
        const syllableIndex = char._syllableIndex;

        if (syllableIndex !== undefined && syllableIndex === lastSyllableIndex && currentSyllable) {
            // Same syllable, append character
            currentSyllable.text += char.char;
            currentSyllable.charEnd++;
            // Update duration to include this character
            const charEnd = char.end;
            currentSyllable.duration = charEnd - lineStartTime - currentSyllable.startOffset;
        } else {
            // New syllable
            if (currentSyllable) {
                syllables.push(currentSyllable);
            }

            currentSyllable = {
                text: char.char,
                startOffset: char.start - lineStartTime,
                duration: char.end - char.start,
                charStart: syllables.reduce((sum, s) => sum + s.text.length, 0),
                charEnd: syllables.reduce((sum, s) => sum + s.text.length, 0) + 1
            };

            lastSyllableIndex = syllableIndex;
        }
    }

    // Push the last syllable
    if (currentSyllable) {
        syllables.push(currentSyllable);
    }

    return syllables;
}

/**
 * Parse a KLyric JSON string into a document object
 * 
 * @param {string} jsonString - JSON string
 * @returns {Object} KLyric document
 */
export function parseKLyric(jsonString) {
    try {
        const doc = JSON.parse(jsonString);

        // Validate version
        if (!doc.version || !doc.version.startsWith('1.')) {
            console.warn('KLyric: Unknown version, may have compatibility issues');
        }

        // Ensure required fields exist
        if (!doc.lines) {
            doc.lines = [];
        }

        if (!doc.styles) {
            doc.styles = { base: { ...DEFAULT_STYLE } };
        }

        if (!doc.animations) {
            doc.animations = { ...DEFAULT_ANIMATION };
        }

        return doc;
    } catch (error) {
        throw new Error(`Failed to parse KLyric JSON: ${error.message}`);
    }
}

/**
 * Serialize KLyric document to JSON string
 * 
 * @param {Object} doc - KLyric document
 * @param {boolean} pretty - Whether to pretty-print
 * @returns {string} JSON string
 */
export function serializeKLyric(doc, pretty = true) {
    // Update modified timestamp
    if (doc.meta) {
        doc.meta.modified = new Date().toISOString();
    }

    return JSON.stringify(doc, null, pretty ? 2 : 0);
}

/**
 * Convert KLyric document to ASS format for export
 * 
 * @param {Object} klyricDoc - KLyric document
 * @returns {string} ASS file content
 */
export function klyricToASS(klyricDoc) {
    const meta = klyricDoc.meta || {};
    const resolution = meta.resolution || { width: 1920, height: 1080 };

    // ASS Header
    let ass = `[Script Info]
; Generated by KLyric Format Converter
Title: ${meta.title || 'Untitled'}
ScriptType: v4.00+
WrapStyle: 0
ScaledBorderAndShadow: yes
PlayResX: ${resolution.width}
PlayResY: ${resolution.height}

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Noto Sans SC,72,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,3,2,2,50,50,20,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
`;

    // Convert each line to ASS dialogue
    for (const line of klyricDoc.lines) {
        const startStr = formatASSTime(line.startTime);
        const endStr = formatASSTime(line.endTime);

        // Build karaoke text from characters
        let karaokeText = '';
        let lastEnd = line.startTime;

        for (const char of line.chars) {
            const gap = char.start - lastEnd;

            // Add gap if there's a pause between characters
            if (gap > 0.01) {
                const gapCs = Math.round(gap * 100);
                karaokeText += `{\\kf${gapCs}}`;
            }

            // Add character timing
            const durationCs = Math.round((char.end - char.start) * 100);
            karaokeText += `{\\kf${durationCs}}${char.char}`;

            lastEnd = char.end;
        }

        ass += `Dialogue: 0,${startStr},${endStr},Default,,0,0,0,karaoke,${karaokeText}\n`;
    }

    return ass;
}

/**
 * Format time for ASS (h:mm:ss.cc format)
 */
function formatASSTime(seconds) {
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = Math.floor(seconds % 60);
    const cs = Math.round((seconds % 1) * 100);

    return `${h}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}.${String(cs).padStart(2, '0')}`;
}

// --- All-in-One Import Function ---

/**
 * Format handler interface for detecting and parsing subtitle formats
 * Each handler knows how to detect and parse a specific format
 */
const FORMAT_HANDLERS = [
    {
        name: 'lrc',
        canHandle: (extension, content) => {
            return extension === 'lrc' ||
                   content.includes('[00:') ||
                   content.includes('[01:');
        },
        parse: (content) => parseLRC(content)
    },
    {
        name: 'srt',
        canHandle: (extension, content) => {
            return extension === 'srt' ||
                   /^\d+\s*\n\d{2}:\d{2}:\d{2}/.test(content);
        },
        parse: (content) => parseSRT(content)
    },
    {
        name: 'ass',
        canHandle: (extension, content) => {
            return extension === 'ass' ||
                   extension === 'ssa' ||
                   content.includes('[Script Info]');
        },
        parse: (content) => parseASS(content)
    }
];

/**
 * Try to parse content as KLyric format
 * @param {string} content - File content
 * @returns {Object|null} KLyric document or null if not valid KLyric
 */
function tryParseAsKLyric(content) {
    try {
        return parseKLyric(content);
    } catch {
        return null;
    }
}

/**
 * Detect format and parse using appropriate handler
 * @param {string} extension - File extension
 * @param {string} content - File content
 * @returns {Object} { lyrics, metadata, format }
 */
function detectAndParse(extension, content) {
    // Try each format handler in order
    for (const handler of FORMAT_HANDLERS) {
        if (handler.canHandle(extension, content)) {
            const result = handler.parse(content);
            return {
                lyrics: result.lyrics,
                metadata: result.metadata,
                format: handler.name
            };
        }
    }

    // Default to LRC if no handler matches
    const result = parseLRC(content);
    return {
        lyrics: result.lyrics,
        metadata: result.metadata,
        format: 'lrc'
    };
}

/**
 * Import any supported subtitle file and convert to KLyric format
 *
 * @param {string} content - File content
 * @param {string} filename - Original filename
 * @param {Object} options - Import options
 * @returns {Object} { klyric: KLyric document, legacy: legacy format, format: source format }
 */
export function importSubtitleToKLyric(content, filename = '', options = {}) {
    const extension = filename.split('.').pop()?.toLowerCase() || '';

    // Check if already KLyric format (special case - no conversion needed)
    if (extension === 'klyric' || extension === 'json') {
        const klyric = tryParseAsKLyric(content);
        if (klyric) {
            return {
                klyric,
                legacy: klyricToLegacy(klyric),
                format: 'klyric'
            };
        }
        // If parsing as KLyric failed, fall through to other format detection
    }

    // Detect format and parse
    const { lyrics, metadata, format } = detectAndParse(extension, content);

    // Convert to KLyric
    const klyric = lyricsToKLyric(lyrics, metadata, {
        ...options,
        sourceFormat: format
    });

    return {
        klyric,
        legacy: klyricToLegacy(klyric),
        format
    };
}

// --- Update Functions ---

/**
 * Update a character's timing in a KLyric document
 * 
 * @param {Object} klyricDoc - KLyric document
 * @param {number} lineIndex - Line index
 * @param {number} charIndex - Character index in line
 * @param {Object} timing - New timing { start, end }
 * @returns {Object} Updated KLyric document
 */
export function updateCharTiming(klyricDoc, lineIndex, charIndex, timing) {
    const doc = { ...klyricDoc };
    doc.lines = [...doc.lines];
    doc.lines[lineIndex] = { ...doc.lines[lineIndex] };
    doc.lines[lineIndex].chars = [...doc.lines[lineIndex].chars];
    doc.lines[lineIndex].chars[charIndex] = {
        ...doc.lines[lineIndex].chars[charIndex],
        start: timing.start,
        end: timing.end
    };

    return doc;
}

/**
 * Update a character's transform in a KLyric document
 * 
 * @param {Object} klyricDoc - KLyric document
 * @param {number} lineIndex - Line index
 * @param {number} charIndex - Character index
 * @param {Object} transform - Transform properties
 * @returns {Object} Updated KLyric document
 */
export function updateCharTransform(klyricDoc, lineIndex, charIndex, transform) {
    const doc = { ...klyricDoc };
    doc.lines = [...doc.lines];
    doc.lines[lineIndex] = { ...doc.lines[lineIndex] };
    doc.lines[lineIndex].chars = [...doc.lines[lineIndex].chars];
    doc.lines[lineIndex].chars[charIndex] = {
        ...doc.lines[lineIndex].chars[charIndex],
        transform: {
            ...(doc.lines[lineIndex].chars[charIndex].transform || {}),
            ...transform
        }
    };

    return doc;
}

/**
 * Update a line's layout in a KLyric document
 * 
 * @param {Object} klyricDoc - KLyric document
 * @param {number} lineIndex - Line index
 * @param {Object} layout - Layout properties
 * @returns {Object} Updated KLyric document
 */
export function updateLineLayout(klyricDoc, lineIndex, layout) {
    const doc = { ...klyricDoc };
    doc.lines = [...doc.lines];
    doc.lines[lineIndex] = {
        ...doc.lines[lineIndex],
        layout: {
            ...(doc.lines[lineIndex].layout || {}),
            ...layout
        }
    };

    return doc;
}

/**
 * Add an animation to a character
 * 
 * @param {Object} klyricDoc - KLyric document
 * @param {number} lineIndex - Line index
 * @param {number} charIndex - Character index
 * @param {Object} animation - Animation reference
 * @returns {Object} Updated KLyric document
 */
export function addCharAnimation(klyricDoc, lineIndex, charIndex, animation) {
    const doc = { ...klyricDoc };
    doc.lines = [...doc.lines];
    doc.lines[lineIndex] = { ...doc.lines[lineIndex] };
    doc.lines[lineIndex].chars = [...doc.lines[lineIndex].chars];

    const char = { ...doc.lines[lineIndex].chars[charIndex] };
    char.animations = [...(char.animations || []), animation];
    doc.lines[lineIndex].chars[charIndex] = char;

    return doc;
}

/**
 * Update syllable timing in bulk (for drag operations in timeline)
 * 
 * @param {Object} klyricDoc - KLyric document
 * @param {number} lineIndex - Line index
 * @param {number} syllableIndex - Syllable index (group of chars)
 * @param {number} newStartTime - New start time for syllable
 * @param {number} newDuration - New duration for syllable
 * @returns {Object} Updated KLyric document
 */
export function updateSyllableTiming(klyricDoc, lineIndex, syllableIndex, newStartTime, newDuration) {
    const doc = { ...klyricDoc };
    doc.lines = [...doc.lines];
    doc.lines[lineIndex] = { ...doc.lines[lineIndex] };
    doc.lines[lineIndex].chars = [...doc.lines[lineIndex].chars];

    // Find all characters belonging to this syllable
    const charsInSyllable = doc.lines[lineIndex].chars.filter(
        (c) => c._syllableIndex === syllableIndex
    );

    if (charsInSyllable.length === 0) return doc;

    const charDuration = newDuration / charsInSyllable.length;

    let currentOffset = 0;
    for (const char of doc.lines[lineIndex].chars) {
        if (char._syllableIndex === syllableIndex) {
            const charIdx = doc.lines[lineIndex].chars.indexOf(char);
            doc.lines[lineIndex].chars[charIdx] = {
                ...char,
                start: Math.round((newStartTime + currentOffset) * 1000) / 1000,
                end: Math.round((newStartTime + currentOffset + charDuration) * 1000) / 1000
            };
            currentOffset += charDuration;
        }
    }

    return doc;
}

// --- Exports ---

export default {
    // Core conversion
    lyricsToKLyric,
    klyricToLegacy,

    // Import/Export
    importSubtitleToKLyric,
    parseKLyric,
    serializeKLyric,
    klyricToASS,

    // Manipulation
    updateCharTiming,
    updateCharTransform,
    updateLineLayout,
    addCharAnimation,
    updateSyllableTiming,

    // Helpers
    generateId,
    resetIdCounter,

    // Constants
    KLYRIC_VERSION,
    DEFAULT_STYLE,
    DEFAULT_EFFECT
};

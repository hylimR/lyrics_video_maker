/**
 * KaraokeUtils.js - Per-Character Timing Utilities
 * 
 * Handles Aegisub-style karaoke timing with {\k##} tags.
 * 
 * Format variations:
 * - {\k50}Text = 50 centiseconds (0.5s) for "Text"
 * - {\kf50} or {\K50} = same, different fill modes in Aegisub
 * - {\kf130}{\kf10}阳 = 130cs pause + 10cs for "阳" (KARASS format)
 * 
 * Data Structure:
 * {
 *   text: "Hello",
 *   startTime: 2.0,
 *   endTime: 5.0,
 *   syllables: [
 *     { text: "Hel", duration: 0.2, startOffset: 0 },
 *     { text: "lo", duration: 0.15, startOffset: 0.2 },
 *   ]
 * }
 */

/**
 * Parse karaoke tags from ASS/SSA formatted text
 * 
 * Handles multiple formats:
 * - Standard: {\k50}Text{\k30}More
 * - KARASS style: {\kf130}{\kf10}阳{\kf29}{\kf12}光 (pause + syllable pairs)
 * 
 * @param {string} text - Text with {\k##} tags
 * @param {number} lineStartTime - Line start time in seconds
 * @returns {Object} { cleanText, syllables }
 */
export function parseKaraokeTags(text) {
    const syllables = [];
    let cleanText = '';
    let currentOffset = 0;

    // First, tokenize all the k-tags and text segments
    // Match either a k-tag or a text segment
    const tokens = [];
    const tokenRegex = /(\{\\[kK]f?\s*\d+\})|([^{]+)/g;
    let match;

    while ((match = tokenRegex.exec(text)) !== null) {
        if (match[1]) {
            // K-tag
            const durationMatch = match[1].match(/(\d+)/);
            const duration = durationMatch ? parseInt(durationMatch[1], 10) / 100 : 0;
            tokens.push({ type: 'k', duration });
        } else if (match[2]) {
            // Text (might contain other tags like {\1c&H...} which we should strip)
            let textContent = match[2].replace(/\{[^}]*\}/g, '').trim();
            if (textContent) {
                tokens.push({ type: 'text', content: textContent });
            }
        }
    }

    // Now process tokens: each k-tag applies to the following text
    // If multiple k-tags appear before text, sum their durations (gap + syllable)
    let pendingDuration = 0;
    let pendingStartOffset = currentOffset;

    for (let i = 0; i < tokens.length; i++) {
        const token = tokens[i];

        if (token.type === 'k') {
            // Look ahead to see if next token is text or another k-tag
            const nextToken = tokens[i + 1];

            if (!nextToken || nextToken.type === 'k') {
                // This is a gap/pause - add to pending duration but start new offset calculation
                if (pendingDuration === 0) {
                    pendingStartOffset = currentOffset;
                }
                // This is a pause before the next syllable
                currentOffset += token.duration;
                pendingDuration = 0; // Don't include gap in syllable duration
                pendingStartOffset = currentOffset; // Syllable starts after the gap
            } else {
                // Next token is text, this k-tag's duration applies to the text
                pendingDuration = token.duration;
            }
        } else if (token.type === 'text') {
            // This text uses the pending duration
            const syllableDuration = pendingDuration > 0 ? pendingDuration : 0.1; // Default to 0.1s if no timing

            syllables.push({
                text: token.content,
                duration: syllableDuration,
                startOffset: pendingStartOffset,
                charStart: cleanText.length,
                charEnd: cleanText.length + token.content.length,
            });

            cleanText += token.content;
            currentOffset = pendingStartOffset + syllableDuration;
            pendingDuration = 0;
            pendingStartOffset = currentOffset;
        }
    }

    // If no karaoke tags found, return null syllables (use estimated)
    if (syllables.length === 0) {
        return { cleanText: text.replace(/\{[^}]*\}/g, ''), syllables: null };
    }

    return { cleanText, syllables };
}

/**
 * Generate estimated syllables for text without timing
 * Each character gets equal duration
 * 
 * @param {string} text - Clean text
 * @param {number} startTime - Line start time
 * @param {number} endTime - Line end time
 * @returns {Array} syllables array
 */
export function generateEstimatedSyllables(text, startTime, endTime) {
    const duration = endTime - startTime;
    const chars = [...text]; // Handle multi-byte chars correctly
    const charDuration = duration / chars.length;

    return chars.map((char, index) => ({
        text: char,
        duration: charDuration,
        startOffset: index * charDuration,
        charStart: index,
        charEnd: index + 1,
    }));
}

/**
 * Calculate character timing from syllables
 * Maps each character to its timing based on syllable data
 * 
 * @param {string} text - Full text
 * @param {Array} syllables - Syllable timing data
 * @param {number} startTime - Line start time
 * @param {number} endTime - Line end time
 * @returns {Array} Per-character timing array
 */
export function calculateCharacterTiming(text, syllables, startTime, endTime) {
    const chars = [...text];
    const charTiming = [];

    // Use syllables if available
    if (syllables && syllables.length > 0) {
        let syllableIndex = 0;
        let charInSyllable = 0;

        for (let i = 0; i < chars.length; i++) {
            const syllable = syllables[syllableIndex];

            if (!syllable) {
                // Beyond defined syllables, use remaining time
                const remainingDuration = endTime - startTime - (syllables[syllables.length - 1]?.startOffset || 0);
                charTiming.push({
                    char: chars[i],
                    startTime: startTime + (syllables[syllables.length - 1]?.startOffset || 0),
                    endTime: endTime,
                    duration: remainingDuration / (chars.length - i),
                });
                continue;
            }

            // Calculate timing within syllable
            const syllableChars = [...syllable.text];
            const charDurationInSyllable = syllable.duration / syllableChars.length;

            charTiming.push({
                char: chars[i],
                startTime: startTime + syllable.startOffset + charInSyllable * charDurationInSyllable,
                endTime: startTime + syllable.startOffset + (charInSyllable + 1) * charDurationInSyllable,
                duration: charDurationInSyllable,
                syllableIndex,
            });

            charInSyllable++;

            // Move to next syllable if we've processed all chars in current
            if (charInSyllable >= syllableChars.length) {
                syllableIndex++;
                charInSyllable = 0;
            }
        }
    } else {
        // Fallback: estimated even distribution
        const duration = endTime - startTime;
        const charDuration = duration / chars.length;

        for (let i = 0; i < chars.length; i++) {
            charTiming.push({
                char: chars[i],
                startTime: startTime + i * charDuration,
                endTime: startTime + (i + 1) * charDuration,
                duration: charDuration,
            });
        }
    }

    return charTiming;
}

/**
 * Get the active character index based on current time
 * 
 * @param {Array} charTiming - Per-character timing array
 * @param {number} currentTime - Current playback time
 * @returns {number} Active character index (-1 if none)
 */
export function getActiveCharIndex(charTiming, currentTime) {
    for (let i = 0; i < charTiming.length; i++) {
        if (currentTime >= charTiming[i].startTime && currentTime < charTiming[i].endTime) {
            return i;
        }
    }
    return -1;
}

/**
 * Get progress within a specific character
 * 
 * @param {Array} charTiming - Per-character timing array
 * @param {number} charIndex - Character index
 * @param {number} currentTime - Current playback time
 * @returns {number} Progress 0-1 within the character
 */
export function getCharProgress(charTiming, charIndex, currentTime) {
    const char = charTiming[charIndex];
    if (!char) return 0;

    if (currentTime < char.startTime) return 0;
    if (currentTime >= char.endTime) return 1;

    return (currentTime - char.startTime) / char.duration;
}

/**
 * Get overall line progress based on character timing
 * 
 * @param {Array} charTiming - Per-character timing array
 * @param {number} currentTime - Current playback time
 * @returns {number} Progress 0-1 across entire line
 */
export function getLineProgress(charTiming, currentTime) {
    if (!charTiming || charTiming.length === 0) return 0;

    const firstStart = charTiming[0].startTime;
    const lastEnd = charTiming[charTiming.length - 1].endTime;
    const totalDuration = lastEnd - firstStart;

    if (currentTime < firstStart) return 0;
    if (currentTime >= lastEnd) return 1;

    return (currentTime - firstStart) / totalDuration;
}

/**
 * Convert syllables back to ASS-style karaoke tags
 * 
 * @param {Array} syllables - Syllable timing data
 * @returns {string} ASS-formatted string with {\k##} tags
 */
export function syllablesToASS(syllables) {
    if (!syllables || syllables.length === 0) return '';

    return syllables.map(syl => {
        const centiseconds = Math.round(syl.duration * 100);
        return `{\\k${centiseconds}}${syl.text}`;
    }).join('');
}

/**
 * Merge adjacent syllables (for editing)
 * 
 * @param {Array} syllables - Syllable timing data
 * @param {number} startIndex - First syllable to merge
 * @param {number} endIndex - Last syllable to merge
 * @returns {Array} New syllables array
 */
export function mergeSyllables(syllables, startIndex, endIndex) {
    const newSyllables = [...syllables];
    const toMerge = newSyllables.splice(startIndex, endIndex - startIndex + 1);

    const merged = {
        text: toMerge.map(s => s.text).join(''),
        duration: toMerge.reduce((sum, s) => sum + s.duration, 0),
        startOffset: toMerge[0].startOffset,
        charStart: toMerge[0].charStart,
        charEnd: toMerge[toMerge.length - 1].charEnd,
    };

    newSyllables.splice(startIndex, 0, merged);
    return newSyllables;
}

/**
 * Split a syllable at a character position
 * 
 * @param {Array} syllables - Syllable timing data
 * @param {number} syllableIndex - Syllable to split
 * @param {number} charOffset - Character position within syllable to split at
 * @returns {Array} New syllables array
 */
export function splitSyllable(syllables, syllableIndex, charOffset) {
    const newSyllables = [...syllables];
    const syllable = newSyllables[syllableIndex];

    const text1 = syllable.text.slice(0, charOffset);
    const text2 = syllable.text.slice(charOffset);

    // Split duration proportionally
    const ratio = charOffset / syllable.text.length;
    const duration1 = syllable.duration * ratio;
    const duration2 = syllable.duration * (1 - ratio);

    const syl1 = {
        text: text1,
        duration: duration1,
        startOffset: syllable.startOffset,
        charStart: syllable.charStart,
        charEnd: syllable.charStart + charOffset,
    };

    const syl2 = {
        text: text2,
        duration: duration2,
        startOffset: syllable.startOffset + duration1,
        charStart: syllable.charStart + charOffset,
        charEnd: syllable.charEnd,
    };

    newSyllables.splice(syllableIndex, 1, syl1, syl2);
    return newSyllables;
}

export default {
    parseKaraokeTags,
    generateEstimatedSyllables,
    calculateCharacterTiming,
    getActiveCharIndex,
    getCharProgress,
    getLineProgress,
    syllablesToASS,
    mergeSyllables,
    splitSyllable,
};

/**
 * LyricParsers.js - LRC and SRT File Parsers
 * 
 * Supports:
 * - LRC format (standard karaoke format)
 * - SRT format (SubRip subtitle format)
 * - Enhanced LRC with word-level timing
 * 
 * Output format: Array of { text, startTime, endTime }
 */

import { parseKaraokeTags } from './karaokeUtils.js';

/**
 * Parse LRC (Lyric) file format
 * 
 * Standard LRC format:
 * [mm:ss.xx]Lyric line text
 * 
 * Enhanced LRC (with word timing):
 * [00:12.00]<00:12.00>Word1 <00:12.50>Word2 <00:13.00>Word3
 * 
 * @param {string} content - Raw LRC file content
 * @returns {Array} Parsed lyrics array
 */
export function parseLRC(content) {
    const lines = content.split('\n');
    const lyrics = [];

    // Regex for timestamp: [mm:ss.xx] or [mm:ss:xx]
    const timestampRegex = /\[(\d{2}):(\d{2})[.:](\d{2,3})\]/g;

    // Regex for metadata tags
    const metadataRegex = /\[(ti|ar|al|au|length|by|offset|re|ve):([^\]]*)\]/i;

    const metadata = {};
    let offset = 0; // Offset in ms

    for (const line of lines) {
        const trimmedLine = line.trim();
        if (!trimmedLine) continue;

        // Check for metadata
        const metaMatch = trimmedLine.match(metadataRegex);
        if (metaMatch) {
            const [, key, value] = metaMatch;
            metadata[key.toLowerCase()] = value.trim();

            // Handle offset
            if (key.toLowerCase() === 'offset') {
                offset = parseInt(value, 10) || 0;
            }
            continue;
        }

        // Extract all timestamps from the line
        const timestamps = [];
        let match;
        timestampRegex.lastIndex = 0;

        while ((match = timestampRegex.exec(trimmedLine)) !== null) {
            const minutes = parseInt(match[1], 10);
            const seconds = parseInt(match[2], 10);
            const centiseconds = parseInt(match[3].padEnd(3, '0').slice(0, 3), 10);

            const timeInSeconds = minutes * 60 + seconds + centiseconds / 1000;
            timestamps.push(timeInSeconds + offset / 1000);
        }

        if (timestamps.length === 0) continue;

        // Get the text after all timestamps
        const text = trimmedLine.replace(timestampRegex, '').trim();
        if (!text) continue;

        // Each timestamp creates a lyric entry with the same text
        for (const startTime of timestamps) {
            lyrics.push({
                text,
                startTime,
                endTime: null, // Will be calculated later
            });
        }
    }

    // Sort by start time
    lyrics.sort((a, b) => a.startTime - b.startTime);

    // Calculate end times (each line ends when the next begins, or +3s for last)
    for (let i = 0; i < lyrics.length; i++) {
        if (i < lyrics.length - 1) {
            // End time is the start of next line (with small gap)
            lyrics[i].endTime = lyrics[i + 1].startTime - 0.1;
        } else {
            // Last line: default duration of 3 seconds
            lyrics[i].endTime = lyrics[i].startTime + 3;
        }

        // Ensure minimum duration
        if (lyrics[i].endTime - lyrics[i].startTime < 0.5) {
            lyrics[i].endTime = lyrics[i].startTime + 0.5;
        }
    }

    return { lyrics, metadata };
}

/**
 * Parse SRT (SubRip) subtitle format
 * 
 * Format:
 * 1
 * 00:00:20,000 --> 00:00:24,400
 * Subtitle text line 1
 * Subtitle text line 2
 * 
 * @param {string} content - Raw SRT file content
 * @returns {Array} Parsed lyrics array
 */
export function parseSRT(content) {
    const lyrics = [];

    // Split by double newline to get blocks
    const blocks = content.split(/\n\s*\n/);

    // Regex for SRT timestamp line
    const timestampRegex = /(\d{2}):(\d{2}):(\d{2})[,.](\d{3})\s*-->\s*(\d{2}):(\d{2}):(\d{2})[,.](\d{3})/;

    for (const block of blocks) {
        const lines = block.trim().split('\n');
        if (lines.length < 2) continue;

        // Find the timestamp line
        let timestampLine = null;
        let textLines = [];
        let foundTimestamp = false;

        for (const line of lines) {
            if (!foundTimestamp && timestampRegex.test(line)) {
                timestampLine = line;
                foundTimestamp = true;
            } else if (foundTimestamp) {
                // Everything after timestamp is text
                textLines.push(line.trim());
            }
        }

        if (!timestampLine || textLines.length === 0) continue;

        const match = timestampLine.match(timestampRegex);
        if (!match) continue;

        // Parse start time
        const startTime =
            parseInt(match[1], 10) * 3600 +
            parseInt(match[2], 10) * 60 +
            parseInt(match[3], 10) +
            parseInt(match[4], 10) / 1000;

        // Parse end time
        const endTime =
            parseInt(match[5], 10) * 3600 +
            parseInt(match[6], 10) * 60 +
            parseInt(match[7], 10) +
            parseInt(match[8], 10) / 1000;

        // Join text lines, remove HTML tags
        const text = textLines
            .join(' ')
            .replace(/<[^>]+>/g, '') // Remove HTML tags
            .replace(/\{[^}]+\}/g, '') // Remove SSA/ASS formatting
            .trim();

        if (text) {
            lyrics.push({
                text,
                startTime,
                endTime,
            });
        }
    }

    // Sort by start time
    lyrics.sort((a, b) => a.startTime - b.startTime);

    return { lyrics, metadata: {} };
}

/**
 * Parse ASS/SSA subtitle format with karaoke tag support
 * 
 * Supports:
 * - {\k##} - Karaoke timing in centiseconds
 * - {\kf##} - Karaoke with fill effect
 * - {\K##} - Karaoke (alternative)
 * 
 * @param {string} content - Raw ASS/SSA file content
 * @returns {Array} Parsed lyrics array with syllable data
 */
export function parseASS(content) {
    const lyrics = [];
    const metadata = {};

    // Parse [Script Info] for metadata
    const scriptInfoMatch = content.match(/\[Script Info\]([\s\S]*?)(?:\[|$)/i);
    if (scriptInfoMatch) {
        const infoLines = scriptInfoMatch[1].split('\n');
        for (const line of infoLines) {
            const match = line.match(/^(Title|Original Script|Original Translation|Script Updated By|Update Details|Artist):(.*)$/i);
            if (match) {
                metadata[match[1].toLowerCase().replace(/\s/g, '')] = match[2].trim();
            }
        }
    }

    // Find the [Events] section
    const eventsMatch = content.match(/\[Events\]([\s\S]*?)(?:\[|$)/i);
    if (!eventsMatch) {
        return { lyrics: [], metadata };
    }

    const eventsSection = eventsMatch[1];
    const lines = eventsSection.split('\n');

    // Find the Format line to get column indices
    let formatParts = [];

    for (const line of lines) {
        if (line.trim().toLowerCase().startsWith('format:')) {
            formatParts = line.substring(7).split(',').map(s => s.trim().toLowerCase());
            break;
        }
    }

    const startIndex = formatParts.indexOf('start');
    const endIndex = formatParts.indexOf('end');
    const textIndex = formatParts.indexOf('text');

    if (startIndex === -1 || endIndex === -1 || textIndex === -1) {
        return { lyrics: [], metadata };
    }

    // Parse dialogue lines
    for (const line of lines) {
        if (!line.trim().toLowerCase().startsWith('dialogue:')) continue;

        const parts = line.substring(9).split(',');
        if (parts.length <= textIndex) continue;

        // Parse timestamps (h:mm:ss.cc format)
        const parseTime = (timeStr) => {
            const match = timeStr.trim().match(/(\d+):(\d{2}):(\d{2})\.(\d{2})/);
            if (!match) return 0;
            return (
                parseInt(match[1], 10) * 3600 +
                parseInt(match[2], 10) * 60 +
                parseInt(match[3], 10) +
                parseInt(match[4], 10) / 100
            );
        };

        const startTime = parseTime(parts[startIndex]);
        const endTime = parseTime(parts[endIndex]);

        // Text is everything from textIndex onwards (may contain commas)
        let rawText = parts.slice(textIndex).join(',');

        // Handle line breaks
        rawText = rawText.replace(/\\N/g, ' ').replace(/\\n/g, ' ');

        // Check for karaoke tags
        const hasKaraokeTags = /\{\\[kK]f?\s*\d+\}/.test(rawText);

        let text;
        let syllables = null;

        if (hasKaraokeTags) {
            // Parse karaoke tags to get syllable timing
            const parsed = parseKaraokeTags(rawText, startTime);
            text = parsed.cleanText;
            syllables = parsed.syllables;
        } else {
            // Remove other ASS formatting tags
            text = rawText
                .replace(/\{[^}]*\}/g, '') // Style overrides
                .replace(/\\[nNhb]/g, ' ')  // Other escape sequences
                .trim();
        }

        if (text && startTime >= 0 && endTime > startTime) {
            lyrics.push({
                text,
                startTime,
                endTime,
                syllables, // null if no karaoke tags, array if present
                rawText: hasKaraokeTags ? rawText : undefined, // Keep original for export
            });
        }
    }

    // Sort by start time
    lyrics.sort((a, b) => a.startTime - b.startTime);

    return { lyrics, metadata };
}

/**
 * Auto-detect format and parse
 * 
 * @param {string} content - Raw file content
 * @param {string} filename - Original filename (for extension detection)
 * @returns {Object} { lyrics, metadata, format }
 */
export function parseSubtitleFile(content, filename = '') {
    const extension = filename.split('.').pop()?.toLowerCase() || '';

    let result;
    let format;

    // Try to detect format
    if (extension === 'lrc' || content.includes('[00:') || content.includes('[01:')) {
        result = parseLRC(content);
        format = 'lrc';
    } else if (extension === 'srt' || /^\d+\s*\n\d{2}:\d{2}:\d{2}/.test(content)) {
        result = parseSRT(content);
        format = 'srt';
    } else if (extension === 'ass' || extension === 'ssa' || content.includes('[Script Info]')) {
        result = parseASS(content);
        format = 'ass';
    } else {
        // Default to LRC
        result = parseLRC(content);
        format = 'lrc';
    }

    return {
        ...result,
        format,
    };
}

/**
 * Validate parsed lyrics
 * 
 * @param {Array} lyrics - Parsed lyrics array
 * @returns {Object} { valid, errors, warnings }
 */
export function validateLyrics(lyrics) {
    const errors = [];
    const warnings = [];

    if (!Array.isArray(lyrics) || lyrics.length === 0) {
        errors.push('No lyrics found in file');
        return { valid: false, errors, warnings };
    }

    lyrics.forEach((lyric, index) => {
        if (!lyric.text) {
            warnings.push(`Line ${index + 1}: Empty text`);
        }
        if (lyric.startTime < 0) {
            errors.push(`Line ${index + 1}: Invalid start time`);
        }
        if (lyric.endTime <= lyric.startTime) {
            warnings.push(`Line ${index + 1}: End time before or equal to start time`);
        }
        if (lyric.endTime - lyric.startTime > 30) {
            warnings.push(`Line ${index + 1}: Very long duration (${(lyric.endTime - lyric.startTime).toFixed(1)}s)`);
        }
    });

    return {
        valid: errors.length === 0,
        errors,
        warnings,
    };
}

export default {
    parseLRC,
    parseSRT,
    parseASS,
    parseSubtitleFile,
    validateLyrics,
};

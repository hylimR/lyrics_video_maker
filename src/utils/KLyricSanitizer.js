/**
 * KLyricSanitizer.js - Centralized sanitization for KLyric Data
 * 
 * Ensures KLyric documents strictly adhere to v2.0 schema types before export.
 * Critical for preventing Rust backend deserialization errors (e.g., "invalid type: string, expected f64").
 */

/**
 * Sanitize a full KLyric document
 * @param {Object} doc - KLyric document (potentially "dirty" with loose types)
 * @param {Object} defaults - Default values for missing fields (e.g. { duration, resolution })
 * @returns {Object} Clean KLyric document ready for serialization
 */
export function sanitizeKLyricDoc(doc, defaults = {}) {
    if (!doc) return null;

    // Deep clone to avoid mutating state
    const cleanDoc = JSON.parse(JSON.stringify(doc));

    // 1. Sanitize Project
    if (cleanDoc.project) {
        cleanDoc.project = sanitizeProject(cleanDoc.project, defaults);
    } else {
        // Create default project if missing
        cleanDoc.project = sanitizeProject({}, defaults);
    }

    // 2. Sanitize Lines
    if (Array.isArray(cleanDoc.lines)) {
        cleanDoc.lines = cleanDoc.lines.map(sanitizeLine);
    } else {
        cleanDoc.lines = [];
    }

    // 3. Sanitize Styles
    if (cleanDoc.styles && typeof cleanDoc.styles === 'object') {
        Object.keys(cleanDoc.styles).forEach(key => {
            cleanDoc.styles[key] = sanitizeStyle(cleanDoc.styles[key]);
        });
    } else {
        cleanDoc.styles = {};
    }

    return cleanDoc;
}

/**
 * Sanitize Project metadata
 */
function sanitizeProject(project, defaults = {}) {
    return {
        ...project,
        title: String(project.title || defaults.title || 'Untitled'),
        artist: String(project.artist || defaults.artist || ''),
        duration: ensureNumber(project.duration || defaults.duration, 0),
        resolution: {
            width: ensureInt(project.resolution?.width || defaults.resolution?.width, 1920),
            height: ensureInt(project.resolution?.height || defaults.resolution?.height, 1080)
        },
        fps: ensureNumber(project.fps, 30)
    };
}

/**
 * Sanitize a single Line
 */
function sanitizeLine(line) {
    const cleanLine = {
        ...line,
        id: line.id || `line_${Math.random().toString(36).substr(2, 9)}`,
        start: ensureNumber(line.start ?? line.startTime, 0),
        end: ensureNumber(line.end ?? line.endTime, 0),
        text: String(line.text || ''),
    };

    // Remove legacy fields
    delete cleanLine.startTime;
    delete cleanLine.endTime;
    delete cleanLine._originalText;
    delete cleanLine._syllables;

    // Sanitize Transform (Strict)
    if (cleanLine.transform) {
        cleanLine.transform = sanitizeTransform(cleanLine.transform);
    }

    // Sanitize Position (Flexible: allows pixels or percentage strings)
    if (cleanLine.position) {
        cleanLine.position = sanitizePosition(cleanLine.position);
    } else if (!cleanLine.transform) {
        // If neither position nor transform exists, default to center
        cleanLine.position = { x: "50%", y: "50%", anchor: "center" };
    }

    // Sanitize Layout
    if (cleanLine.layout) {
        cleanLine.layout = {
            ...cleanLine.layout,
            gap: ensureNumber(cleanLine.layout.gap, 0),
            maxWidth: cleanLine.layout.maxWidth ? ensureNumber(cleanLine.layout.maxWidth, 0) : undefined
        };
    }

    // Sanitize Characters
    if (Array.isArray(cleanLine.chars)) {
        cleanLine.chars = cleanLine.chars.map(sanitizeChar);
    } else {
        cleanLine.chars = [];
    }

    return cleanLine;
}

/**
 * Sanitize a single Character
 */
function sanitizeChar(char) {
    const cleanChar = {
        ...char,
        char: String(char.char ?? char.text ?? ''),
        start: ensureNumber(char.start ?? char.timing?.start, 0),
        end: ensureNumber(char.end ?? char.timing?.end, 0),
    };

    // Remove legacy fields
    delete cleanChar.text;
    delete cleanChar.timing;
    delete cleanChar._syllableIndex;
    delete cleanChar.id;

    // Sanitize Transform (Strict)
    if (cleanChar.transform) {
        cleanChar.transform = sanitizeTransform(cleanChar.transform);
    }

    return cleanChar;
}

/**
 * Sanitize Transform - STRICTLY ENFORCE NUMBERS
 * Rust `Transform` struct uses `f32` for all fields.
 * Strings like "50%" WILL cause a panic.
 */
function sanitizeTransform(t) {
    return {
        x: ensureNumber(t.x, 0),
        y: ensureNumber(t.y, 0),
        rotation: ensureNumber(t.rotation, 0),
        scale: ensureNumber(t.scale, 1),
        scaleX: ensureNumber(t.scaleX ?? t.scale, 1), // Fallback to uniform scale if missing
        scaleY: ensureNumber(t.scaleY ?? t.scale, 1),
        opacity: ensureNumber(t.opacity, 1),
        anchorX: ensureNumber(t.anchorX, 0.5),
        anchorY: ensureNumber(t.anchorY, 0.5)
    };
}

/**
 * Sanitize Position - Flexible
 * Rust `Position` struct uses `PositionValue` enum which accepts:
 * - Pixels(f32)
 * - Percentage(String)
 */
function sanitizePosition(p) {
    return {
        x: ensurePositionValue(p.x),
        y: ensurePositionValue(p.y),
        anchor: ensureAnchor(p.anchor)
    };
}

/**
 * Sanitize Style Object
 * Matches Rust `Style` struct
 */
function sanitizeStyle(style) {
    if (!style || typeof style !== 'object') return {};

    const cleanStyle = {};

    if (style.extends !== undefined) {
        cleanStyle.extends = String(style.extends);
    }

    if (style.font) {
        cleanStyle.font = sanitizeFont(style.font);
    }

    if (style.colors) {
        cleanStyle.colors = sanitizeStateColors(style.colors);
    }

    if (style.stroke) {
        cleanStyle.stroke = sanitizeStroke(style.stroke);
    }

    if (style.shadow) {
        cleanStyle.shadow = sanitizeShadow(style.shadow);
    }

    if (style.glow) {
        cleanStyle.glow = sanitizeGlow(style.glow);
    }

    return cleanStyle;
}

/**
 * Sanitize Font settings
 */
function sanitizeFont(font) {
    if (!font || typeof font !== 'object') return undefined;

    return {
        family: String(font.family || 'Noto Sans SC'),
        size: ensureNumber(font.size, 72.0),
        weight: ensureInt(font.weight, 700),
        style: ensureFontStyle(font.style),
        letterSpacing: ensureNumber(font.letterSpacing, 0.0)
    };
}

/**
 * Validate FontStyle enum
 */
function ensureFontStyle(val) {
    const validStyles = ['normal', 'italic', 'oblique'];
    // Handle input, converting to lowercase
    if (typeof val === 'string') {
        const lower = val.toLowerCase();
        if (validStyles.includes(lower)) return lower;
    }
    return 'normal';
}

/**
 * Sanitize StateColors
 */
function sanitizeStateColors(colors) {
    if (!colors || typeof colors !== 'object') return undefined;

    const cleanColors = {};
    if (colors.inactive) cleanColors.inactive = sanitizeFillStroke(colors.inactive);
    if (colors.active) cleanColors.active = sanitizeFillStroke(colors.active);
    if (colors.complete) cleanColors.complete = sanitizeFillStroke(colors.complete);

    return cleanColors;
}

/**
 * Sanitize FillStroke (String or Object)
 */
function sanitizeFillStroke(val) {
    if (!val) return undefined;

    if (typeof val === 'string') return val;

    if (typeof val === 'object') {
        const result = {};
        if (val.fill) result.fill = String(val.fill);
        if (val.stroke) result.stroke = String(val.stroke);
        return result;
    }

    return undefined;
}

/**
 * Sanitize Stroke settings
 */
function sanitizeStroke(stroke) {
    if (!stroke || typeof stroke !== 'object') return undefined;

    const cleanStroke = {
        width: ensureNumber(stroke.width, 0)
    };

    if (stroke.color) cleanStroke.color = String(stroke.color);

    return cleanStroke;
}

/**
 * Sanitize Shadow settings
 */
function sanitizeShadow(shadow) {
    if (!shadow || typeof shadow !== 'object') return undefined;

    const cleanShadow = {
        x: ensureNumber(shadow.x, 2.0),
        y: ensureNumber(shadow.y, 2.0),
        blur: ensureNumber(shadow.blur, 4.0)
    };

    if (shadow.color) cleanShadow.color = String(shadow.color);

    return cleanShadow;
}

/**
 * Sanitize Glow settings
 */
function sanitizeGlow(glow) {
    if (!glow || typeof glow !== 'object') return undefined;

    const cleanGlow = {
        blur: ensureNumber(glow.blur, 8.0),
        intensity: ensureNumber(glow.intensity, 0.5)
    };

    if (glow.color) cleanGlow.color = String(glow.color);

    return cleanGlow;
}


// --- Helpers ---

/**
 * Force value to be a number (float).
 * Handles strings like "50%", "1.2px" by stripping non-numeric chars if needed, 
 * or just parseFloat.
 */
function ensureNumber(val, defaultVal = 0) {
    if (typeof val === 'number') {
        return isNaN(val) ? defaultVal : val;
    }

    if (typeof val === 'string') {
        // Check for percentage (e.g., "50%" -> 0.5) - ONLY for things that *should* be ratio?
        // Actually, for Transform.opacity/anchor, 0-1 is expected. 
        // For Transform.x, pixels are expected.
        // It's ambiguous. Safer to just ParseFloat.
        // "50%" -> 50. This might be wrong for anchor (0.5), but 50 is better than NaN/Crash.
        const parsed = parseFloat(val);
        return isNaN(parsed) ? defaultVal : parsed;
    }

    return defaultVal;
}

/**
 * Force value to be an integer
 */
function ensureInt(val, defaultVal = 0) {
    const num = parseInt(val, 10);
    return isNaN(num) ? defaultVal : num;
}

/**
 * Validates PositionValue (Number or String)
 */
function ensurePositionValue(val) {
    if (val === undefined || val === null) return undefined;
    if (typeof val === 'number') return val;
    if (typeof val === 'string') return val; // Allow strings for Position
    return undefined;
}

/**
 * Validate Anchor string
 */
function ensureAnchor(val) {
    const validAnchors = [
        'top-left', 'top-center', 'top-right',
        'center-left', 'center', 'center-right',
        'bottom-left', 'bottom-center', 'bottom-right'
    ];
    return validAnchors.includes(val) ? val : 'center';
}

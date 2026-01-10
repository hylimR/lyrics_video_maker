/**
 * KLyricMigrator.js
 * 
 * Utility to migrate KLyric v1.0 documents to the v2.0 declarative format.
 * Ensures backward compatibility for existing projects.
 */

/**
 * Migrate any KLyric document to v2.0 format
 * @param {Object} doc - The input document (v1.0 or partial v2.0)
 * @returns {Object} - Valid KLyric v2.0 document
 */
export function migrateToV2(doc) {
    if (!doc) return createEmptyV2Doc();

    // Deep clone to avoid mutating input
    const v1 = JSON.parse(JSON.stringify(doc));

    // If already v2, return as is (with validation ideally, but trusting for now)
    if (v1.version === '2.0') {
        return v1;
    }

    console.log('📦 Migrating KLyric document from v1.0 to v2.0...');

    // 1. Initialize v2 structure
    const v2 = {
        $schema: "https://klyric.dev/schemas/v2.0/klyric.schema.json",
        version: "2.0",
        project: migrateProject(v1),
        theme: migrateTheme(v1),
        styles: migrateStyles(v1),
        effects: migrateEffects(v1),
        lines: []
    };

    // 2. Migrate lines
    if (Array.isArray(v1.lines)) {
        v2.lines = v1.lines.map(line => migrateLine(line, v2.project));
    }

    return v2;
}

function createEmptyV2Doc() {
    return {
        version: "2.0",
        project: {
            title: "Untitled",
            duration: 0,
            resolution: { width: 1920, height: 1080 },
            fps: 30
        },
        lines: []
    };
}

function migrateProject(v1) {
    // v1 used 'meta' or root properties
    const meta = v1.meta || {};
    const settings = v1.settings || {};

    return {
        title: meta.title || v1.title || "Untitled",
        artist: meta.artist || v1.artist || "",
        album: meta.album || "",
        duration: meta.duration || 0,
        resolution: settings.resolution || meta.resolution || { width: 1920, height: 1080 },
        fps: 30, // v1 didn't store fps usually
        audio: meta.audio || null,
        created: meta.created || new Date().toISOString(),
        modified: new Date().toISOString()
    };
}

function migrateTheme(v1) {
    const settings = v1.settings || {};
    return {
        background: settings.background || { type: "solid", color: "#000000" },
        defaultStyle: "base"
    };
}

function migrateStyles(v1) {
    // v1 might have had styles in 'defaults' or scattered
    // We'll create a standard base style
    return {
        base: {
            font: {
                family: "Noto Sans SC",
                size: 72,
                weight: 700
            },
            colors: {
                inactive: { fill: "#FFFFFF66" },
                active: { fill: "#FFD700" },
                complete: { fill: "#FFFFFF" }
            },
            stroke: {
                width: 3,
                color: "#000000"
            },
            shadow: {
                color: "#00000080",
                x: 2,
                y: 2,
                blur: 4
            }
        }
    };
}

function migrateEffects(v1) {
    // Migrate any custom effects if v1 had them, otherwise default set
    return {
        fadeIn: {
            type: "transition",
            trigger: "enter",
            duration: 0.3,
            easing: "easeOutCubic",
            properties: {
                opacity: { from: 0, to: 1 },
                y: { from: 20, to: 0 }
            }
        },
        karaokeWipe: {
            type: "karaoke",
            mode: "mask",
            direction: "ltr",
            easing: "linear"
        }
    };
}

function migrateLine(lineV1, project) {
    // 1. Timing
    const start = lineV1.start !== undefined ? lineV1.start : (lineV1.startTime || 0);
    const end = lineV1.end !== undefined ? lineV1.end : (lineV1.endTime || 0);

    // 2. Position & Transform
    // v1 transform might be relative offsets
    const transformV1 = lineV1.transform || {};
    const layoutV1 = lineV1.layout || {};

    // In v1, we often used absolute x/y in transform for the line container
    // or sometimes offsets. For v2, we prefer percentage-based positioning.

    // Check if we have specific coordinates, otherwise center
    let pos = { x: "50%", y: "50%", anchor: "center" };
    let transform = {
        scale: transformV1.scale || 1,
        rotation: transformV1.rotation || 0,
        opacity: transformV1.alpha !== undefined ? transformV1.alpha : (transformV1.opacity !== undefined ? transformV1.opacity : 1)
    };

    // If v1 had specific X/Y, try to preserve it as pixel offset or convert to position
    // For simplicity in migration, we can leave position centered and put everything in transform offset
    // BUT clean migration is better.

    // If transform.x/y exists and looks like screen coords (large numbers)
    if ((transformV1.x && Math.abs(transformV1.x) > 100) || (transformV1.y && Math.abs(transformV1.y) > 100)) {
        // Likely absolute coordinates
        pos.x = transformV1.x;
        pos.y = transformV1.y;
    } else {
        // Likely offsets
        if (transformV1.x) transform.x = transformV1.x;
        if (transformV1.y) transform.y = transformV1.y;
        if (transformV1.offsetX) transform.x = transformV1.offsetX;
        if (transformV1.offsetY) transform.y = transformV1.offsetY;
    }

    // 3. Characters
    const chars = (lineV1.chars || []).map(charV1 => migrateChar(charV1));

    return {
        id: lineV1.id || `line_${crypto.randomUUID().substr(0, 8)}`,
        start,
        end,
        text: lineV1.text || chars.map(c => c.char).join(''),
        style: "base", // Default style
        effects: ["fadeIn", "karaokeWipe"], // Default effects
        position: pos,
        transform: transform,
        chars
    };
}

function migrateChar(charV1) {
    // Timing
    let start = 0, end = 0;
    if (charV1.timing) {
        start = charV1.timing.start;
        end = charV1.timing.end;
    } else {
        start = charV1.start || 0;
        end = charV1.end || 0;
    }

    // Transform
    const transformV1 = charV1.transform || {};
    const transform = {};

    if (transformV1.scale !== undefined) transform.scale = transformV1.scale;
    if (transformV1.rotation !== undefined) transform.rotation = transformV1.rotation;
    if (transformV1.x) transform.x = transformV1.x;
    if (transformV1.y) transform.y = transformV1.y;
    if (transformV1.offsetX) transform.x = transformV1.offsetX;
    if (transformV1.offsetY) transform.y = transformV1.offsetY;

    return {
        char: charV1.char || charV1.text || "",
        start,
        end,
        transform: Object.keys(transform).length > 0 ? transform : undefined
    };
}

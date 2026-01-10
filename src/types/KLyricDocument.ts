/**
 * KLyric v2.0 TypeScript Type Definitions
 * 
 * These types define the structure of KLyric v2.0 documents.
 * They are designed to be:
 * - Fully declarative (no imperative logic)
 * - AI-friendly (semantic naming, self-documenting)
 * - Compatible with both JS/TS frontend and Rust backend
 */

// ============================================================================
// Core Types
// ============================================================================

/**
 * Root KLyric v2.0 document structure
 */
export interface KLyricDocument {
    /** JSON Schema reference */
    $schema?: string;
    /** Format version (must be "2.0") */
    version: "2.0";
    /** Project metadata */
    project: Project;
    /** Theme and background settings */
    theme?: Theme;
    /** Named style definitions */
    styles?: Record<string, Style>;
    /** Named effect definitions */
    effects?: Record<string, Effect>;
    /** Lyric lines with timing and characters */
    lines: Line[];
}

// ============================================================================
// Project & Theme
// ============================================================================

export interface Project {
    /** Song or project title */
    title: string;
    /** Artist name */
    artist?: string;
    /** Album name */
    album?: string;
    /** Total duration in seconds */
    duration: number;
    /** Video resolution */
    resolution: Resolution;
    /** Frames per second for export */
    fps?: 24 | 25 | 30 | 50 | 60;
    /** Path to audio file */
    audio?: string;
    /** Creation timestamp (ISO 8601) */
    created?: string;
    /** Last modified timestamp (ISO 8601) */
    modified?: string;
}

export interface Resolution {
    /** Width in pixels */
    width: number;
    /** Height in pixels */
    height: number;
}

export interface Theme {
    /** Background configuration */
    background?: Background;
    /** Name of default style for all lines */
    defaultStyle?: string;
}

export interface Background {
    /** Background type */
    type?: "solid" | "gradient" | "image" | "video";
    /** Solid color (hex or rgba) */
    color?: Color;
    /** Gradient definition */
    gradient?: Gradient;
    /** Path to background image */
    image?: string;
    /** Path to background video */
    video?: string;
    /** Background opacity (0-1) */
    opacity?: number;
}

export interface Gradient {
    /** Gradient type */
    type?: "linear" | "radial";
    /** Array of colors in gradient */
    colors: Color[];
    /** Angle in degrees for linear gradients */
    angle?: number;
    /** Optional stop positions (0-1) */
    stops?: number[];
}

// ============================================================================
// Styling
// ============================================================================

export interface Style {
    /** Parent style to inherit from */
    extends?: string;
    /** Font settings */
    font?: Font;
    /** State-based colors */
    colors?: StateColors;
    /** Text stroke settings */
    stroke?: Stroke;
    /** Drop shadow settings */
    shadow?: Shadow;
    /** Glow effect settings */
    glow?: Glow;
}

export interface Font {
    /** Font family (comma-separated for fallbacks) */
    family?: string;
    /** Font size in pixels */
    size?: number;
    /** Font weight (100-900 or "normal"/"bold") */
    weight?: number | "normal" | "bold";
    /** Font style */
    style?: "normal" | "italic" | "oblique";
    /** Letter spacing in pixels */
    letterSpacing?: number;
}

export interface StateColors {
    /** Colors before character is highlighted */
    inactive?: FillStroke;
    /** Colors during character highlight */
    active?: FillStroke;
    /** Colors after highlight completes */
    complete?: FillStroke;
}

export interface FillStroke {
    /** Fill color */
    fill?: Color;
    /** Stroke color */
    stroke?: Color;
}

/** Color value - hex (#RRGGBB or #RRGGBBAA) or rgba() */
export type Color = string;

export interface Stroke {
    /** Stroke width in pixels */
    width?: number;
    /** Stroke color */
    color?: Color;
}

export interface Shadow {
    /** Shadow color */
    color?: Color;
    /** Horizontal offset in pixels */
    x?: number;
    /** Vertical offset in pixels */
    y?: number;
    /** Blur radius in pixels */
    blur?: number;
}

export interface Glow {
    /** Glow color */
    color?: Color;
    /** Glow blur radius */
    blur?: number;
    /** Glow intensity (0-1) */
    intensity?: number;
}

// ============================================================================
// Effects & Animation
// ============================================================================

export interface Effect {
    /** Effect type */
    type: "transition" | "karaoke" | "keyframe" | "particle" | "custom";
    /** When the effect triggers */
    trigger?: "enter" | "exit" | "active" | "inactive" | "always";
    /** Effect duration in seconds */
    duration?: number;
    /** Delay before effect starts */
    delay?: number;
    /** Easing function */
    easing?: Easing;

    // Transition-specific
    /** Properties to animate (for transition type) */
    properties?: Record<string, AnimatedValue>;

    // Karaoke-specific
    /** Karaoke mode */
    mode?: "mask" | "color" | "wipe" | "reveal";
    /** Effect direction */
    direction?: "ltr" | "rtl" | "ttb" | "btt";

    // Keyframe-specific
    /** Keyframe array (for keyframe type) */
    keyframes?: Keyframe[];

    // Common
    /** Number of iterations (or "infinite") */
    iterations?: number | "infinite";
}

export interface AnimatedValue {
    /** Starting value */
    from: number;
    /** Ending value */
    to: number;
}

export interface Keyframe {
    /** Position in animation (0-1) */
    time: number;
    /** Opacity at this keyframe */
    opacity?: number;
    /** Scale at this keyframe */
    scale?: number;
    /** Horizontal scale */
    scaleX?: number;
    /** Vertical scale */
    scaleY?: number;
    /** Rotation in degrees */
    rotation?: number;
    /** X offset in pixels */
    x?: number;
    /** Y offset in pixels */
    y?: number;
    /** Color at this keyframe */
    color?: Color;
    /** Easing to next keyframe */
    easing?: Easing;
}

export type Easing =
    | "linear"
    | "easeIn" | "easeOut" | "easeInOut"
    | "easeInQuad" | "easeOutQuad" | "easeInOutQuad"
    | "easeInCubic" | "easeOutCubic" | "easeInOutCubic"
    | "easeInQuart" | "easeOutQuart" | "easeInOutQuart"
    | "easeInQuint" | "easeOutQuint" | "easeInOutQuint"
    | "easeInSine" | "easeOutSine" | "easeInOutSine"
    | "easeInExpo" | "easeOutExpo" | "easeInOutExpo"
    | "easeInCirc" | "easeOutCirc" | "easeInOutCirc"
    | "easeInElastic" | "easeOutElastic" | "easeInOutElastic"
    | "easeInBack" | "easeOutBack" | "easeInOutBack"
    | "easeInBounce" | "easeOutBounce" | "easeInOutBounce";

// ============================================================================
// Layout & Position
// ============================================================================

export interface Position {
    /** X position (pixels or percentage string like "50%") */
    x?: number | string;
    /** Y position (pixels or percentage string like "50%") */
    y?: number | string;
    /** Anchor point for positioning */
    anchor?: Anchor;
}

export type Anchor =
    | "top-left" | "top-center" | "top-right"
    | "center-left" | "center" | "center-right"
    | "bottom-left" | "bottom-center" | "bottom-right";

export interface Transform {
    /** X offset in pixels */
    x?: number;
    /** Y offset in pixels */
    y?: number;
    /** Rotation in degrees */
    rotation?: number;
    /** Uniform scale factor */
    scale?: number;
    /** Horizontal scale factor */
    scaleX?: number;
    /** Vertical scale factor */
    scaleY?: number;
    /** Opacity (0-1) */
    opacity?: number;
    /** Transform anchor X (0-1, 0.5 = center) */
    anchorX?: number;
    /** Transform anchor Y (0-1, 0.5 = center) */
    anchorY?: number;
}

export interface Layout {
    /** Text layout mode */
    mode?: "horizontal" | "vertical" | "path";
    /** Horizontal alignment */
    align?: "left" | "center" | "right";
    /** Vertical alignment */
    justify?: "top" | "middle" | "bottom";
    /** Gap between characters in pixels */
    gap?: number;
    /** Whether to wrap text */
    wrap?: boolean;
    /** Maximum width before wrapping */
    maxWidth?: number;
}

// ============================================================================
// Lines & Characters
// ============================================================================

export interface Line {
    /** Unique identifier */
    id?: string;
    /** Line start time in seconds */
    start: number;
    /** Line end time in seconds */
    end: number;
    /** Full text (optional, can derive from chars) */
    text?: string;
    /** Style name to apply */
    style?: string;
    /** Effect names to apply */
    effects?: string[];
    /** Line position */
    position?: Position;
    /** Line transform */
    transform?: Transform;
    /** Text layout settings */
    layout?: Layout;
    /** Characters with individual timing */
    chars: Char[];
}

export interface Char {
    /** The character(s) to display */
    char: string;
    /** Highlight start time in seconds */
    start: number;
    /** Highlight end time in seconds */
    end: number;
    /** Override style for this character */
    style?: string;
    /** Additional effects for this character */
    effects?: string[];
    /** Character-specific transform */
    transform?: Transform;
}

// ============================================================================
// Utility Types
// ============================================================================

/**
 * Create a new empty KLyric v2.0 document
 */
export function createEmptyDocument(project: Partial<Project> = {}): KLyricDocument {
    return {
        version: "2.0",
        project: {
            title: project.title || "Untitled",
            artist: project.artist || "",
            duration: project.duration || 0,
            resolution: project.resolution || { width: 1920, height: 1080 },
            fps: project.fps || 30,
            created: new Date().toISOString(),
            modified: new Date().toISOString(),
        },
        theme: {
            background: { type: "solid", color: "#000000" },
            defaultStyle: "base",
        },
        styles: {
            base: {
                font: { family: "Noto Sans SC", size: 72, weight: 700 },
                colors: {
                    inactive: { fill: "#FFFFFF66" },
                    active: { fill: "#FFD700" },
                    complete: { fill: "#FFFFFF" },
                },
                stroke: { width: 3, color: "#000000" },
            },
        },
        effects: {
            fadeIn: {
                type: "transition",
                trigger: "enter",
                duration: 0.3,
                easing: "easeOutCubic",
                properties: {
                    opacity: { from: 0, to: 1 },
                    y: { from: 20, to: 0 },
                },
            },
            karaokeWipe: {
                type: "karaoke",
                mode: "mask",
                direction: "ltr",
                easing: "linear",
            },
        },
        lines: [],
    };
}

/**
 * Validate a KLyric document structure (basic validation)
 */
export function validateDocument(doc: unknown): doc is KLyricDocument {
    if (typeof doc !== "object" || doc === null) return false;
    const d = doc as Record<string, unknown>;

    if (d.version !== "2.0") return false;
    if (typeof d.project !== "object" || d.project === null) return false;
    if (!Array.isArray(d.lines)) return false;

    return true;
}

export default KLyricDocument;

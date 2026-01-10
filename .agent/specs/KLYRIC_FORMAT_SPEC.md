# KLyric Format Specification v2.0

**Author:** Lyric Video Maker  
**Last Updated:** 2026-01-10  
**Status:** Active  
**Version:** 2.0  

---

## 📋 Overview

**KLyric** (Karaoke Lyric) is a JSON-based format designed to extend the capabilities of ASS/SSA subtitle files for advanced lyric video production. It provides:

- **Per-character transforms** (position, rotation, scale, opacity)
- **CSS-like styling** with inheritance
- **Custom keyframe animations** per character
- **Individual character timing** (start/end per character)
- **Hierarchical parent-child relationships** (Document -> Line -> Character)
- **Layout modes** (horizontal, vertical, path-based)
- **Effect definitions** and triggers
- **Multi-layer support**

---

## 🏗️ Document Structure

The root of the document is `KLyricDocumentV2`.

```jsonc
{
  "$schema": "./klyric-schema.json",
  "version": "2.0",
  
  // Project metadata
  "project": {
    "title": "Song Title",
    "artist": "Artist Name",
    "duration": 240.5,
    "resolution": {
      "width": 1920,
      "height": 1080
    },
    // Optional
    "author": "User",
    "description": "...",
    "framerate": 60
  },
  
  // Theme configuration (optional)
  "theme": {
    "background": {
        "color": "#000000",
        "image": "path/to/bg.png", // Reserved for future
        "video": "path/to/bg.mp4"  // Reserved for future
    }
  },

  // Named reusable definitions
  "styles": { /* ... */ },
  "effects": { /* ... */ },

  // The actual lyric content
  "lines": [ /* ... */ ]
}
```

---

## 📝 Lines & Characters

### Line Structure

Each line is a container that holds text and/or characters. Lines manage the high-level layout.

```jsonc
{
  "id": "line_001",           // Optional unique ID
  "start": 0.141,             // Start time in seconds
  "end": 4.490,               // End time in seconds
  "text": "Full text line",   // Optional full text representation
  
  // Layout (how children are arranged)
  "layout": {
    "mode": "horizontal",     // "horizontal" | "vertical" | "path"
    "align": "center",        // "left" | "center" | "right"
    "justify": "middle",      // "top" | "middle" (or "center") | "bottom"
    "gap": 0,                 // spacing between characters in pixels
    "wrap": false,            // allow line wrapping
    "maxWidth": 1000          // optional max width in pixels
  },
  
  // Transform (relative to canvas)
  "transform": {
    "x": 960,                 // pixels
    "y": 540,                 // pixels
    "rotation": 0,            // degrees
    "scale": 1,               // uniform scale
    "scaleX": 1,
    "scaleY": 1,
    "opacity": 1,
    "anchorX": 0.5,           // 0.0-1.0 (pivot point)
    "anchorY": 0.5
  },
  
  // Positioning (Alternative to transform.x/y, supports percentages)
  "position": {
    "x": "50%",               // string "50%" or number pixels
    "y": "50%",
    "anchor": "center"        // See Anchor Enum below
  },

  // References
  "style": "verse_style",     // Reference to named style
  "effects": ["fadeIn"],      // List of named effect references
  
  // Content
  "chars": [ /* ... */ ]
}
```

### Character Structure

Each character has **individual control** over timing, transforms, and animations.

```jsonc
{
  "char": "阳",               // The character string
  "start": 0.141,             // Highlight start time (seconds)
  "end": 0.241,               // Highlight end time (seconds)
  
  // Inline style override
  "style": "highlight_style", // Reference or direct style object (if supported, currently name)
  
  // Effects
  "effects": ["glow", "bounce"],
  
  // Transform (relative to layout position)
  "transform": {
    "x": 0,                   // Offset X
    "y": 0,                   // Offset Y
    "rotation": 0,
    "scale": 1.2,
    "opacity": 1
  }
}
```

---

## 🎨 Styles Definition

Styles support inheritance and define visual appearance.

```jsonc
"styles": {
  "base": {
    "font": {
      "family": "Noto Sans SC",
      "size": 72,
      "weight": 700,
      "style": "normal",       // "normal" | "italic" | "oblique"
      "letterSpacing": 0
    },
    "colors": {
      // Color State System
      "inactive": { "fill": "#FFFFFF66", "stroke": "#000000" }, // Before highlight
      "active": "#FFFFFF",                                      // During highlight (string shorthand)
      "complete": { "fill": "#CCCCCC" }                         // After highlight
    },
    "stroke": {
      "width": 3,
      "color": "#000000"       // Optional override
    },
    "shadow": {
      "color": "#00000080",
      "x": 2,
      "y": 2,
      "blur": 4
    },
    "glow": {
      "color": "#FFD700",
      "blur": 10,
      "intensity": 0.5
    }
  },
  
  "chorus": {
    "extends": "base",         // Inherit from "base"
    "font": { "size": 96 }     // Override specific properties
  }
}
```

### Color Formats
Colors can be defined as:
1. **String**: Hex (`"#RRGGBB"`, `"#RRGGBBAA"`) or standard web colors.
2. **Object**: `{"fill": "...", "stroke": "..."}`.

---

## 🎬 Effects Definition

Effects define dynamic behavior using various engines (Keyframes, Particles, Shaders).

```jsonc
"effects": {
  "fadeIn": {
    "type": "keyframe",
    "trigger": "enter",        // "enter" | "exit" | "active" | "inactive" | "always"
    "duration": 0.5,
    "easing": "easeOutQuad",
    "keyframes": [
      {
        "time": 0.0,
        "opacity": 0,
        "scale": 0.5
      },
      {
        "time": 1.0,
        "opacity": 1,
        "scale": 1.0
      }
    ]
  },
  
  "magicDust": {
    "type": "particle",
    "trigger": "active",
    "preset": "sparkle",       // Use a built-in particle preset
    // OR custom config
    "particleConfig": {
        "count": 50,
        "speed": 100,
        "life": 1.0,
        "colors": ["#FF0000", "#00FF00"]
    }
  },
  
  "karaokeWipe": {
    "type": "karaoke",
    "trigger": "active",
    "mode": "mask",            // "mask" | "color" | "wipe" | "reveal"
    "direction": "ltr"         // "ltr" | "rtl" | "ttb" | "btt"
  }
}
```

### Effect Types
- `transition`: Global element transitions
- `karaoke`: Syllable filling logic
- `keyframe`: CSS-style keyframe animation
- `particle`: Particle emitter systems
- `custom`: Reserved for code-driven effects

### Effect Triggers
- `enter`: When item first appears (Line start)
- `exit`: When item disappears (Line end)
- `active`: During the karaoke active timing (Char start -> end)
- `inactive`: When not active
- `always`: Loop continuously

### Easing Functions
`linear`, `easeIn`, `easeOut`, `easeInOut`, `easeInQuad`, `easeOutQuad`, `easeInOutQuad`, `easeInCubic`, `easeOutCubic`, `easeInOutCubic`, `easeInQuart`, `easeOutQuart`, `easeInOutQuart`, `easeInQuint`, `easeOutQuint`, `easeInOutQuint`, `easeInSine`, `easeOutSine`, `easeInOutSine`, `easeInExpo`, `easeOutExpo`, `easeInOutExpo`, `easeInCirc`, `easeOutCirc`, `easeInOutCirc`, `easeInElastic`, `easeOutElastic`, `easeInOutElastic`, `easeInBack`, `easeOutBack`, `easeInOutBack`, `easeInBounce`, `easeOutBounce`, `easeInOutBounce`

---

## 📐 Enums Reference

### Layout
**LayoutMode**: `horizontal`, `vertical`, `path`
**Align**: `left`, `center`, `right`
**Justify**: `top`, `middle` (alias: `center`), `bottom`

### Anchors (kebab-case)
`top-left`, `top-center`, `top-right`
`center-left`, `center`, `center-right`
`bottom-left`, `bottom-center`, `bottom-right`

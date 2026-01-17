---
name: design-lyrics
description: LLM-powered lyric video designer. Generates styled KLyric documents with animations, effects, and layouts based on mood/genre prompts. Use with /design-lyrics command.
user_invocable: true
---

# LLM Lyric Video Designer

Generate professionally styled KLyric v2.0 documents based on natural language descriptions of mood, genre, and visual style.

## When to Use

- User wants to design a lyric video style
- User provides mood/genre descriptions like "energetic pop" or "moody indie"
- User has timed lyrics and wants styling/animation suggestions
- Invoked directly with `/design-lyrics`

## Input Requirements

The skill expects:
1. **Timed lyrics** - Either:
   - Existing KLyric document with timing
   - ASS/SRT content with timing
   - Or just lyrics text (you'll need to estimate timing or ask)

2. **Style prompt** - Natural language description:
   - Genre: "J-pop", "indie rock", "EDM", "ballad", "hip-hop"
   - Mood: "energetic", "melancholic", "romantic", "aggressive", "dreamy"
   - Visual style: "neon", "minimalist", "retro", "cyberpunk", "elegant"
   - Energy curve: "builds up", "constant high energy", "calm with explosive chorus"

## Output Format

Generate a complete KLyric v2.0 document with:

```json
{
  "version": "2.0",
  "project": {
    "title": "Song Title",
    "artist": "Artist Name",
    "duration": 180.0,
    "resolution": { "width": 1920, "height": 1080 },
    "framerate": 60
  },
  "theme": {
    "background": { "color": "#000000" }
  },
  "styles": {
    // Named reusable styles
  },
  "effects": {
    // Named effect definitions
  },
  "lines": [
    // Lyric lines with timing and style assignments
  ]
}
```

## Style Generation Guidelines

### Color Palettes by Genre

| Genre | Primary | Secondary | Accent | Background |
|-------|---------|-----------|--------|------------|
| Pop | #FF6B9D | #C44DFF | #FFD93D | #1A0A2E |
| Rock | #FF4444 | #FF8C00 | #FFFF00 | #0D0D0D |
| EDM | #00FFFF | #FF00FF | #00FF00 | #0A0A1A |
| Ballad | #E8D5B7 | #B8860B | #FFF8DC | #1C1C2E |
| Hip-Hop | #FFD700 | #FF4500 | #32CD32 | #121212 |
| Indie | #DDA0DD | #87CEEB | #F0E68C | #2D2D3A |
| J-Pop | #FF69B4 | #00CED1 | #FFB6C1 | #1A1A2E |
| Lo-Fi | #8B7355 | #CD853F | #D2B48C | #2D2416 |

### Font Recommendations by Genre

| Genre | Font Family | Weight | Size Range |
|-------|-------------|--------|------------|
| Pop | "Montserrat", "Poppins" | 700-900 | 64-96px |
| Rock | "Bebas Neue", "Impact" | 700 | 72-108px |
| EDM | "Orbitron", "Audiowide" | 600-700 | 60-84px |
| Ballad | "Playfair Display", "Cormorant" | 400-500 | 56-80px |
| Hip-Hop | "Oswald", "Anton" | 700-900 | 72-120px |
| Indie | "Quicksand", "Nunito" | 500-600 | 48-72px |
| J-Pop | "Noto Sans JP", "M PLUS Rounded" | 500-700 | 64-96px |

### Effect Presets by Energy Level

#### Calm (Energy: 1-3)
```json
{
  "fadeIn": {
    "type": "keyframe",
    "trigger": "enter",
    "duration": 0.8,
    "easing": "easeOutSine",
    "keyframes": [
      { "time": 0.0, "opacity": 0, "y": 20 },
      { "time": 1.0, "opacity": 1, "y": 0 }
    ]
  },
  "gentleFloat": {
    "type": "keyframe",
    "trigger": "always",
    "duration": 3.0,
    "easing": "easeInOutSine",
    "keyframes": [
      { "time": 0.0, "y": 0 },
      { "time": 0.5, "y": -5 },
      { "time": 1.0, "y": 0 }
    ]
  }
}
```

#### Medium (Energy: 4-6)
```json
{
  "scalePop": {
    "type": "keyframe",
    "trigger": "active",
    "duration": 0.3,
    "easing": "easeOutBack",
    "keyframes": [
      { "time": 0.0, "scale": 1.0 },
      { "time": 0.5, "scale": 1.15 },
      { "time": 1.0, "scale": 1.0 }
    ]
  },
  "colorShift": {
    "type": "keyframe",
    "trigger": "active",
    "duration": 0.5,
    "easing": "linear",
    "keyframes": [
      { "time": 0.0, "fill": "#FFFFFF" },
      { "time": 1.0, "fill": "#FFD700" }
    ]
  }
}
```

#### High (Energy: 7-10)
```json
{
  "explosivePop": {
    "type": "keyframe",
    "trigger": "active",
    "duration": 0.2,
    "easing": "easeOutElastic",
    "keyframes": [
      { "time": 0.0, "scale": 1.0, "rotation": 0 },
      { "time": 0.3, "scale": 1.3, "rotation": 5 },
      { "time": 1.0, "scale": 1.0, "rotation": 0 }
    ]
  },
  "neonPulse": {
    "type": "keyframe",
    "trigger": "active",
    "duration": 0.15,
    "easing": "easeInOutQuad",
    "keyframes": [
      { "time": 0.0, "glow": { "blur": 10, "color": "#FF00FF" } },
      { "time": 0.5, "glow": { "blur": 30, "color": "#00FFFF" } },
      { "time": 1.0, "glow": { "blur": 10, "color": "#FF00FF" } }
    ]
  },
  "particles": {
    "type": "particle",
    "trigger": "active",
    "preset": "sparkle",
    "particleConfig": {
      "count": 30,
      "speed": 150,
      "life": 0.8,
      "colors": ["#FF00FF", "#00FFFF", "#FFFF00"]
    }
  }
}
```

### Karaoke Highlight Styles

```json
{
  "karaokeWipe": {
    "type": "karaoke",
    "trigger": "active",
    "mode": "wipe",
    "direction": "ltr"
  },
  "karaokeColor": {
    "type": "karaoke",
    "trigger": "active",
    "mode": "color"
  }
}
```

### Layout Strategies

#### Centered (Default)
```json
{
  "layout": {
    "mode": "horizontal",
    "align": "center",
    "justify": "middle"
  },
  "position": { "x": "50%", "y": "50%", "anchor": "center" }
}
```

#### Bottom Third (TV Style)
```json
{
  "position": { "x": "50%", "y": "85%", "anchor": "bottom-center" }
}
```

#### Staggered (Dynamic)
Alternate lines between positions:
- Odd lines: `{ "x": "30%", "y": "40%" }`
- Even lines: `{ "x": "70%", "y": "60%" }`

#### Vertical Stack (Multiple Lines)
```json
{
  "layout": { "mode": "vertical", "gap": 20 },
  "position": { "x": "50%", "y": "50%", "anchor": "center" }
}
```

## Generation Process

1. **Analyze the prompt** - Extract genre, mood, energy level, visual keywords
2. **Select color palette** - Match to genre/mood
3. **Choose typography** - Font family, size, weight based on genre
4. **Design effects** - Entry/exit animations, active highlights, particles
5. **Plan layout** - Positioning strategy based on content density
6. **Apply to lines** - Assign styles to different song sections (verse/chorus/bridge)

## Section Detection Heuristics

- **Verse**: Longer gaps before, moderate text density
- **Chorus**: Repeating patterns, shorter lines, higher energy
- **Bridge**: Unique timing pattern, often slower
- **Intro/Outro**: Beginning/end of song, often instrumental gaps

## Example Prompts → Outputs

### Prompt: "energetic J-pop with neon colors and bouncy text"

```json
{
  "styles": {
    "verse": {
      "font": { "family": "M PLUS Rounded 1c", "size": 72, "weight": 700 },
      "colors": {
        "inactive": { "fill": "#FF69B466" },
        "active": { "fill": "#FF69B4" },
        "complete": { "fill": "#00CED1" }
      },
      "glow": { "color": "#FF69B4", "blur": 15, "intensity": 0.6 }
    },
    "chorus": {
      "extends": "verse",
      "font": { "size": 96 },
      "glow": { "blur": 25, "intensity": 0.8 }
    }
  },
  "effects": {
    "bounceIn": {
      "type": "keyframe",
      "trigger": "enter",
      "duration": 0.4,
      "easing": "easeOutBounce",
      "keyframes": [
        { "time": 0.0, "scale": 0, "opacity": 0 },
        { "time": 1.0, "scale": 1, "opacity": 1 }
      ]
    },
    "activePop": {
      "type": "keyframe",
      "trigger": "active",
      "duration": 0.2,
      "easing": "easeOutBack",
      "keyframes": [
        { "time": 0.0, "scale": 1.0, "y": 0 },
        { "time": 0.5, "scale": 1.2, "y": -10 },
        { "time": 1.0, "scale": 1.0, "y": 0 }
      ]
    }
  }
}
```

### Prompt: "moody indie ballad, minimalist, soft fades"

```json
{
  "styles": {
    "default": {
      "font": { "family": "Quicksand", "size": 56, "weight": 400 },
      "colors": {
        "inactive": { "fill": "#DDA0DD33" },
        "active": { "fill": "#DDA0DD" },
        "complete": { "fill": "#87CEEBAA" }
      }
    }
  },
  "effects": {
    "softFadeIn": {
      "type": "keyframe",
      "trigger": "enter",
      "duration": 1.2,
      "easing": "easeOutSine",
      "keyframes": [
        { "time": 0.0, "opacity": 0 },
        { "time": 1.0, "opacity": 1 }
      ]
    },
    "gentleHighlight": {
      "type": "keyframe",
      "trigger": "active",
      "duration": 0.6,
      "easing": "easeInOutSine",
      "keyframes": [
        { "time": 0.0, "opacity": 0.6 },
        { "time": 1.0, "opacity": 1.0 }
      ]
    }
  }
}
```

## Easing Functions Reference

Use these easing names in keyframe definitions:

| Category | Options |
|----------|---------|
| Linear | `linear` |
| Quad | `easeInQuad`, `easeOutQuad`, `easeInOutQuad` |
| Cubic | `easeInCubic`, `easeOutCubic`, `easeInOutCubic` |
| Quart | `easeInQuart`, `easeOutQuart`, `easeInOutQuart` |
| Quint | `easeInQuint`, `easeOutQuint`, `easeInOutQuint` |
| Sine | `easeInSine`, `easeOutSine`, `easeInOutSine` |
| Expo | `easeInExpo`, `easeOutExpo`, `easeInOutExpo` |
| Circ | `easeInCirc`, `easeOutCirc`, `easeInOutCirc` |
| Elastic | `easeInElastic`, `easeOutElastic`, `easeInOutElastic` |
| Back | `easeInBack`, `easeOutBack`, `easeInOutBack` |
| Bounce | `easeInBounce`, `easeOutBounce`, `easeInOutBounce` |

**Usage guidance:**
- **Entries**: Use `easeOut*` variants (starts fast, ends smooth)
- **Exits**: Use `easeIn*` variants (starts smooth, ends fast)
- **Loops**: Use `easeInOut*` variants (smooth both ends)
- **Bouncy/Fun**: `easeOutBounce`, `easeOutElastic`, `easeOutBack`
- **Elegant/Smooth**: `easeOutSine`, `easeOutQuad`, `easeOutCubic`
- **Dramatic**: `easeOutExpo`, `easeOutQuart`

## Workflow

1. User provides timed lyrics (ASS/KLyric) or plain lyrics
2. User describes desired style: `/design-lyrics cyberpunk EDM with glitch effects`
3. Generate complete styled KLyric document
4. Output JSON that can be directly loaded into the editor

## Anti-Patterns

- Don't create overly complex animations that would be hard to render
- Don't use more than 3-4 effects per character (performance)
- Don't mix too many conflicting visual styles
- Don't ignore the song's natural rhythm and phrasing
- Don't make all lines identical - vary verse/chorus/bridge styling

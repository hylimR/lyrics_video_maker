---
name: klyric-format
description: KLyric v2.0 JSON format specification - document structure, styling, timing, and effects
---

# KLyric Format Patterns

## Overview

KLyric v2.0 is a JSON-based lyric format for rich per-character styling and animation.

**Specification**: `.agent/specs/KLYRIC_FORMAT_SPEC.md`
**Rust Models**: `crates/klyric-renderer/src/model/`
**JS Converter**: `src/utils/KLyricFormat.js`

## Document Structure

```json
{
  "version": "2.0",
  "meta": { "title": "...", "artist": "..." },
  "theme": { /* default styles */ },
  "lines": [
    {
      "id": "line-1",
      "start": 0.0,
      "end": 5.0,
      "chars": [
        {
          "char": "H",
          "start": 0.0,
          "end": 0.5,
          "style": { /* per-char overrides */ },
          "effects": [{ "type": "fadeIn", "duration": 0.2 }]
        }
      ]
    }
  ]
}
```

## Key Data Models (Rust)

| Struct | Purpose |
|--------|---------|
| `KLyricDocumentV2` | Root document |
| `Line` | Single lyric line with timing |
| `Char` | Individual character with timing/effects |
| `Style` | Visual properties (font, fill, stroke) |
| `Effect` | Animation definition |
| `Theme` | Default styles and colors |

## Style Resolution

Styles cascade with inheritance:
```
Theme.defaultStyle → Line.style → Char.style
```

Use `StyleResolver` to flatten inheritance before rendering.

## Effect Types

### Built-in Effects
```rust
pub enum EffectType {
    FadeIn,
    FadeOut,
    ScalePop,
    Wobbly,
    Pulse,
    Rainbow,
    Glow,
    // ...
}
```

### Effect Parameters
```json
{
  "type": "wobbly",
  "amplitude": 5.0,
  "frequency": 2.0,
  "phase": 0.0
}
```

## Timing Model

```
Line: start ─────────────────────────── end
      ├── Char 1: start ─── end
      │          ├── Char 2: start ─── end
      │          │          ├── Char 3: ...
```

- `Line.start/end`: Line visibility window
- `Char.start/end`: Character activation timing
- Effects trigger relative to `Char.start`

## Format Conversion

### Import Flow
```
ASS/SRT/LRC → importSubtitleToKLyric() → KLyric Document
```

### Export Flow
```
KLyric Document → klyricToASS() → ASS file
```

### Legacy Compatibility
```javascript
// Convert KLyric to legacy format for existing renderer
const legacy = klyricToLegacy(klyricDoc);
```

## Common Tasks

### Add a New Style Property
1. Add field to `Style` struct in Rust models
2. Update `StyleResolver` for inheritance
3. Update JS converter if needed
4. Run `npm run build:wasm`

### Add a New Effect
1. Add to `EffectType` enum
2. Implement calculation in `effects.rs`
3. Implement rendering in both targets
4. Add to JSON schema

## Schema Validation

```javascript
import schema from '.agent/specs/klyric-schema.json';
// Use JSON Schema to validate documents
```

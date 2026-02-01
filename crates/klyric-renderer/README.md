# KLyric Renderer

The core rendering engine for the Lyric Video Maker. This crate is responsible for parsing the KLyric JSON format and rendering frames using `skia-safe` (Native).

## Supported Targets

1.  **Native (Rust):** For high-performance video encoding (export) via Tauri and FFmpeg. This is the primary target and uses `skia-safe` for GPU-accelerated 2D rendering.
2.  **WebAssembly (WASM):** *Experimental / Currently Unavailable.* The WASM implementation for web preview is under development and may not be fully functional or feature-complete compared to the native renderer.

## Build Instructions

### Native (for Testing/Export)

To build as a standard Rust library:

```bash
cargo build -p klyric-renderer
```

## Architecture

```text
┌─────────────────┐
│  KLyric v2.0    │
│  JSON Document  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│     Parser      │ → model.rs
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Style Resolver  │ → style.rs
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Layout Engine   │ → layout.rs
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Effect Engine   │ → effects.rs
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   Renderer      │ → renderer.rs (skia-safe)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  RGBA Pixels    │
└─────────────────┘
```

## Structure

*   `src/lib.rs`: Entry point. Exposes `Renderer` for native use.
*   `src/model.rs`: Defines the `KLyric` data structures (Project, Theme, Style, Line, etc.) and JSON serialization.
*   `src/renderer.rs`: Contains the `render_frame` logic using `skia-safe`.
*   `src/layout.rs`: Handles text layout and font resolution.
*   `src/effects.rs`: Processes animation logic and easing functions.
*   `src/particle`: Particle system implementation.

## Expression System

The renderer supports dynamic expressions for particle overrides and effect properties.

**Available Variables:**
*   `t`: Current time in seconds.
*   `progress`: Effect progress (0.0 to 1.0).
*   `width`: Canvas width.
*   `height`: Canvas height.
*   `index` / `i`: Current character index.
*   `count`: Total character count.
*   `char_width`: Width of the current character (if available).
*   `char_height`: Height of the current character (if available).
*   `PI`: Mathematical constant PI.

## Usage (Native)

```rust
use klyric_renderer::{Renderer, model::KLyricDocumentV2};

let doc: KLyricDocumentV2 = serde_json::from_str(json_str)?;
let width = 1920;
let height = 1080;
let mut renderer = Renderer::new(width, height);

// Render a frame at specific time (seconds)
// Returns a Result<Vec<u8>> (RGBA pixel data)
let buffer = renderer.render_frame(&doc, time_in_seconds)?;
```

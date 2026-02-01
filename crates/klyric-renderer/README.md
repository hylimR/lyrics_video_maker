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

## Usage (Native)

```rust
use klyric_renderer::{Renderer, model::Project};

let project: Project = serde_json::from_str(json_str)?;
let mut renderer = Renderer::new(project);

// Initialize (loads fonts, resources)
renderer.initialize()?;

// Render a frame at specific time (seconds)
let width = 1920;
let height = 1080;
// Returns a Vec<u8> (RGBA pixel data)
let buffer = renderer.render_frame(time_in_seconds, width, height)?;
```

# KLyric Renderer

The core rendering engine for the Lyric Video Maker. This crate is responsible for parsing the KLyric JSON format and rendering frames using `tiny-skia`.

## Dual Compilation Targets

This crate is designed to be compiled for both:

1.  **WebAssembly (WASM):** For real-time preview in the React frontend.
2.  **Native (Rust):** For high-performance video encoding (export) via Tauri and FFmpeg.

## Build Instructions

### WASM (for Web Preview)

To build the WASM module for the frontend:

```bash
npm run build:wasm
```

This runs `wasm-pack build --target web` and outputs to `../../src/wasm`.

### Native (for Testing/Export)

To build as a standard Rust library (used by `src-tauri`):

```bash
cargo build -p klyric-renderer
```

## Structure

*   `src/lib.rs`: Entry point. Exposes `KLyricWasmRenderer` for WASM and `KLyricRenderer` for native use.
*   `src/model.rs`: Defines the `KLyric` data structures (Project, Theme, Style, Line, etc.) and JSON serialization. This is the source of truth for the data format.
*   `src/renderer.rs`: Contains the `render_frame` logic using `tiny-skia`.

## Usage

### WASM

```javascript
import init, { KLyricWasmRenderer } from '../wasm/klyric_renderer';

// Initialize WASM
await init();

// Create renderer with project JSON
const renderer = new KLyricWasmRenderer(JSON.stringify(projectData));

// Render a frame at specific time (seconds)
// Returns a Uint8ClampedArray (RGBA pixel data)
const frameData = renderer.render(currentTime);
```

### Native

```rust
use klyric_renderer::{KLyricRenderer, model::Project};

let project: Project = serde_json::from_str(json_str)?;
let mut renderer = KLyricRenderer::new(project);

// Render to a pixel buffer
let width = 1920;
let height = 1080;
let buffer = renderer.render(time_in_seconds, width, height)?;
```

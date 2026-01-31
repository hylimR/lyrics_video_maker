# GEMINI.md

This file provides guidance to the Gemini assistant when working with code in this repository.

## Project Overview

A professional **Lyric Video Generator** built entirely in Rust. Creates high-quality lyric videos with per-character timing (K-Timing), animations, and visual effects.

**Stack:** Pure Rust with Iced GUI framework and Skia rendering.

## Mandatory Checks

> [!IMPORTANT]
> **CRITICAL: You must run these checks before applying any code change.**
> 1. **Compilation**: `cargo check --workspace`
> 2. **Tests**: `cargo test --workspace`
> 3. **Lints**: `cargo clippy --workspace -- -D warnings`
> 4. **WASM Check**: `cargo check -p klyric-renderer --target wasm32-unknown-unknown`
>
> **Post-Coding Review:**
> You MUST run the `requesting-code-review` skill (or other relevant review skills) after every coding session to verify your work.
>
> **Failure to do so is a violation of protocol.**

## Development Commands

```bash
# Run the application
./run.ps1                              # PowerShell startup script
cargo run -p klyric-gui                # Run GUI directly

# Build
cargo build --workspace                # Debug build
cargo build --workspace --release      # Release build

# Code quality
cargo fmt                              # Format all code
cargo clippy --workspace               # Lint all crates
cargo test --workspace                 # Run all tests
cargo test -p klyric-renderer          # Test specific crate
cargo test -p klyric-renderer test_fn  # Run single test

# Check compilation
cargo check --workspace                # Check all crates
cargo check -p klyric-renderer --target wasm32-unknown-unknown # Check WASM
```

## Testing Strategy

All core logic in `klyric-renderer` is covered by unit tests.

### Test Organization
- **Unit Tests**: Inline in `src/*.rs` files using `#[cfg(test)] mod tests {}`.
- **Integration Tests**: In `tests/*.rs` (e.g., `renderer_test.rs`).
- **Helpers**: Shared test utilities in `tests/helpers.rs`.

### Coverage Goals
| Module | Scope | Status |
|--------|-------|--------|
| `style.rs` | Resolution, inheritance, merging | ✅ Covered |
| `effects.rs` | Easing, lerp, progress, transforms | ✅ Covered |
| `layout.rs` | Text layout, alignment, spacing | ✅ Covered |
| `text.rs` | Font loading, measurement (platform-aware) | ✅ Covered |
| `renderer/mod.rs` | Frame rendering, particles | ✅ Covered |

### Writing Tests
1. Use `tests/helpers.rs` for common setups (docs, styles).
2. Use `approx_eq` (or helpers) for float comparisons.
3. Mock system dependencies (fonts) where possible or make tests tolerant.
4. Use `#[cfg(target_os = ...)]` for platform-specific tests.


## Architecture

### Workspace Structure
```
crates/
├── klyric-gui/          # Iced GUI application (main entry point)
│   ├── src/
│   │   ├── main.rs      # Entry point, daemon setup
│   │   ├── app.rs       # Elm architecture: update(), view()
│   │   ├── state.rs     # AppState definition
│   │   ├── message.rs   # Message enum for events
│   │   ├── worker.rs    # Background render worker
│   │   ├── audio.rs     # Audio playback (rodio)
│   │   ├── theme.rs     # UI Styles & Constants (New standard)
│   │   └── widgets/     # UI components
│   │       ├── editor.rs
│   │       ├── preview.rs
│   │       ├── timeline.rs
│   │       ├── ktiming.rs
│   │       ├── inspector.rs
│   │       └── settings.rs
│   └── fonts/           # Embedded fonts
│
├── klyric-renderer/     # Core rendering library
│   ├── src/
│   │   ├── model/       # KLyric v2.0 data models (source of truth)
│   │   ├── parser.rs    # JSON parsing
│   │   ├── style.rs     # Style resolution
│   │   ├── layout.rs    # Text layout
│   │   ├── effects.rs   # Animation/effects
│   │   ├── particle/    # Particle system
│   │   ├── presets/     # Effect presets (particles, transitions)
│   │   ├── renderer/    # Native renderer (skia-safe)
│   │   └── wasm_renderer.rs  # WASM renderer (tiny-skia)
│   └── Cargo.toml
│
└── klyric-preview/      # Standalone OpenGL preview (winit + glutin)

samples/                 # Sample .klyric files and audio
.agent/specs/            # KLyric format specification
```

### Rendering Pipeline
```
KLyric v2.0 JSON → Parser → Style Resolver → Layout Engine → Effect Engine → Rasterizer → RGBA Pixels
```

The renderer supports dual targets via conditional compilation:
- **Native:** `skia-safe` (GPU accelerated). Used by the desktop app and preview.
- **WASM:** `tiny-skia` + `ab_glyph` (CPU, for browser preview). Used when compiling for `wasm32`.

### Iced Architecture (Elm Pattern)

The GUI follows Iced's Elm architecture:
- **Model:** `AppState` in `state.rs`. Stores the document as `Option<Arc<KLyricDocumentV2>>`.
- **Update:** `app::update()` handles `Message` variants.
- **View:** `app::view()` returns widget tree.

```rust
// Message flow
User Action → Message → update() → AppState mutation → view() re-render
```

## Critical Patterns

### 1. KLyric Format is Source of Truth
Rust structs in `crates/klyric-renderer/src/model/` define the KLyric v2.0 format:
- `KLyricDocumentV2` - Root document
- `Line` - Lyric line with timing
- `Char` - Individual character with timing/effects
- `Style`, `Effect`, `Theme` - Visual properties

Specification: `.agent/specs/KLYRIC_FORMAT_SPEC.md`

### 2. Background Render Worker
Heavy rendering runs in a separate thread via `worker.rs`. It shares the document state efficiently via `Arc<KLyricDocumentV2>`.
```rust
// GUI sends render requests
worker_connection.send(RenderRequest { ... });

// Worker responds with rendered frames
Subscription::run(worker_subscription) → Message::FrameRendered(handle)
```

### 3. Dual-Target Renderer
When modifying `klyric-renderer`, be aware of conditional compilation. **Always verify WASM compilation.**
```rust
#[cfg(not(target_arch = "wasm32"))]
pub mod renderer;  // skia-safe

#[cfg(target_arch = "wasm32")]
pub mod wasm_renderer;  // tiny-skia
```

### 4. Audio Playback
Audio is handled via `rodio` in `audio.rs`. The `AudioManager` syncs with playback state.

### 5. UI Styling
Use `crates/klyric-gui/src/theme.rs` for all styles, constants, and icons. Do not hardcode values in widgets.
- **Icons:** Use `theme::icons::*` constants.
- **Styles:** Define `StyleSheet` impls or container styles in `theme.rs`.

### 6. Async Operations
File operations (Open, Save, Export) use `Task::perform` to avoid blocking the UI.
1. Set status to "Busy" (optional, for immediate feedback).
2. Spawn async task (`rfd` dialogs, file I/O).
3. Dispatch result `Message` (e.g., `FileOpened`, `FileSaved`).
4. Handle result in `update()`.

## Troubleshooting & Known Issues

### Windows Build Locks
If you encounter "file in use" errors with `skia-bindings` (especially `pdb` or `dll` files), `rust-analyzer` is likely holding a lock.
**Fix:** Stop the language server or run `cargo clean -p skia-bindings`.

### Dependency Conflicts
There is a known conflict between `ashpd` (via `rfd` -> `ashpd` 0.11) and `zbus` versions if dependencies aren't managed carefully.
- `klyric-gui` currently manages this by pinning or ensuring compatible versions.

### Compilation Errors
- **UTF-8 Errors:** Clang 18 + GCC 13 headers can cause `string_view` UTF-8 errors in `skia-bindings`. Ensure your build environment is clean or use a compatible Clang version.

## Directory Reference

| Path | Purpose |
|------|---------|
| `crates/klyric-gui/src/app.rs` | Main update/view logic |
| `crates/klyric-gui/src/state.rs` | Application state (`AppState`) |
| `crates/klyric-gui/src/message.rs` | Event definitions (`Message`) |
| `crates/klyric-gui/src/theme.rs` | UI Styles, Icons, Constants |
| `crates/klyric-renderer/src/model/` | KLyric data models |
| `crates/klyric-renderer/src/renderer/` | Native skia-safe renderer |
| `crates/klyric-renderer/src/wasm_renderer.rs` | WASM tiny-skia renderer |

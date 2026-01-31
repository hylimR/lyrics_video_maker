# GEMINI.md

This file provides guidance to Gemini Code when working with code in this repository.

## Project Overview

A professional **Lyric Video Generator** built entirely in Rust. Creates high-quality lyric videos with per-character timing (K-Timing), animations, and visual effects.

**Stack:** Pure Rust with Iced GUI framework and skia-safe rendering.

## Mandatory Checks

> [!IMPORTANT]
> **CRITICAL: You must run these checks before applying any code change.**
> 1. **Compilation**: `cargo check --workspace`
> 2. **Tests**: `cargo test --workspace`
> 3. **Lints**: `cargo clippy --workspace -- -D warnings`
>
> **Post-Coding Review:**
> You MUST run the `requesting-code-review` skill (or other relevant review skills like `m15-anti-pattern`) after every coding session to verify your work.
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
│   │   ├── text.rs      # Font loading and text measurement
│   │   └── expressions.rs  # Expression evaluation
│   └── Cargo.toml


samples/                 # Sample .klyric files and audio
.agent/specs/            # KLyric format specification
```

### Rendering Pipeline
```
KLyric v2.0 JSON → Parser → Style Resolver → Layout Engine → Effect Engine → Renderer (skia-safe) → RGBA Pixels
```

The renderer uses `skia-safe` for GPU-accelerated rendering. Legacy WASM support using `tiny-skia` exists in the codebase but is no longer actively maintained.

### Iced Architecture (Elm Pattern)

The GUI follows Iced's Elm architecture:
- **Model:** `AppState` in `state.rs`
- **Update:** `app::update()` handles `Message` variants
- **View:** `app::view()` returns widget tree

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
Heavy rendering runs in a separate thread via `worker.rs`:
```rust
// GUI sends render requests
worker_connection.send(RenderRequest { ... });

// Worker responds with rendered frames
Subscription::run(worker_subscription) → Message::FrameRendered(handle)
```

### 3. Native Renderer
The renderer uses `skia-safe` for high-performance GPU-accelerated rendering:
- Text rendering with `TextRenderer` for font loading and measurement
- Canvas-based drawing with particle effects and animations
- Direct integration with the background render worker

### 4. Audio Playback
Audio is handled via `rodio` in `audio.rs`. The `AudioManager` syncs with playback state.

## Directory Reference

| Path | Purpose |
|------|---------|
| `crates/klyric-gui/src/app.rs` | Main update/view logic |
| `crates/klyric-gui/src/state.rs` | Application state |
| `crates/klyric-gui/src/message.rs` | Event definitions |
| `crates/klyric-renderer/src/model/` | KLyric data models |
| `crates/klyric-renderer/src/renderer/` | Native skia-safe renderer |
| `.agent/specs/KLYRIC_FORMAT_SPEC.md` | Format specification |

## Windows Build Notes

If you encounter "file in use" errors with `skia-bindings`, rust-analyzer may be holding file locks.

**Solution** (configured in `.vscode/settings.json`):
```json
{ "rust-analyzer.check.extraArgs": ["--target-dir", "target/ra"] }
```

Quick fixes:
1. Stop rust-analyzer: `Ctrl+Shift+P` → "rust-analyzer: Stop Server"
2. Clean: `cargo clean -p skia-bindings`

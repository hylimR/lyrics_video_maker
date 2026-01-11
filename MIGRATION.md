# Migration from tiny-skia to skia-safe

## Overview
We have migrated the rendering engine from `tiny-skia` (CPU-only, smaller) to `skia-safe` (Wrapper for Skia, industry standard, supports GPU/CPU).

## Changes made
1.  **Dependencies**:
    - Removed `tiny-skia` from `src-tauri/Cargo.toml`.
    - Removed `tiny-skia` from `crates/klyric-renderer/Cargo.toml` (if present).
    - Added `skia-safe` to `crates/klyric-renderer/Cargo.toml`.

2.  **Rendering Backend**:
    - `crates/klyric-renderer` now uses `skia_safe::Canvas`, `skia_safe::Paint`, etc. for drawing.
    - WASM export logic in `lib.rs` (or `wasm.rs`) likely interacts with `skia-safe` or acts as a bridge. *Note: `skia-safe` is typically native-only or requires specific WASM setup (like CanvasKit). Currently `klyric-renderer` uses conditional compilation.*

## Why the change?
- **Performance**: `skia-safe` offers better potential for hardware acceleration and richer features.
- **Features**: Support for more complex text effects, paths, and filters that were limited in `tiny-skia`.
- **Compatibility**: Aligns with industry standards for cross-platform rendering.

## Verification
- Ensure `cargo build` passes in `crates/klyric-renderer`.
- Ensure `npm run tauri:build` passes for the desktop app.

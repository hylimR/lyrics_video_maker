---
name: rust-renderer
description: Patterns for the klyric-renderer Rust crate - dual-target WASM/Native rendering with tiny-skia and skia-safe
---

# Rust Renderer Patterns

## Architecture Overview

The `klyric-renderer` crate (`crates/klyric-renderer/`) is a **dual-target** rendering engine:

| Target | Graphics Library | Use Case |
|--------|-----------------|----------|
| **WASM** | tiny-skia (CPU) | Browser preview via Canvas |
| **Native** | skia-safe (GPU) | Desktop export via FFmpeg |

## Critical Rule: WASM Rebuild

> [!CAUTION]
> After ANY change to `crates/klyric-renderer/`, you MUST run:
> ```bash
> npm run build:wasm
> ```
> The preview will NOT reflect changes until rebuilt.

## Pipeline Stages

```
KLyric JSON → Parser → Style Resolver → Layout Engine → Effect Engine → Rasterizer
```

### Key Files

| Stage | File(s) | Purpose |
|-------|---------|---------|
| Parser | `model/*.rs`, `parser.rs` | Parse KLyric v2.0 JSON |
| Style | `style.rs` | CSS-like inheritance resolution |
| Layout | `layout.rs` | Text positioning & wrapping |
| Effects | `effects.rs` | Animation calculations |
| WASM Render | `wasm_renderer.rs` | tiny-skia rasterization |
| Native Render | `renderer/mod.rs` | skia-safe rasterization |

## Dual-Target Patterns

### Feature Gating

```rust
#[cfg(target_arch = "wasm32")]
fn render_wasm() { /* tiny-skia implementation */ }

#[cfg(not(target_arch = "wasm32"))]
fn render_native() { /* skia-safe implementation */ }
```

### Effect Implementation Checklist

When adding a new effect:
1. Define in `effects.rs` (shared logic)
2. Implement in `wasm_renderer.rs` (WASM target)
3. Implement in `renderer/mod.rs` (Native target)
4. Run `npm run build:wasm`
5. Test both preview AND export

## Font Handling

### WASM (tiny-skia + ab_glyph)
- Fonts must be embedded in binary
- Limited to bundled fonts in `resources/fonts/`
- Use `fontdb::Database` for font selection

### Native (skia-safe)
- Full system font access
- Use `get_system_fonts()` Tauri command
- Support font family fallbacks

## Common Patterns

### Transform Application Order
```rust
// Always apply in this order:
// 1. Translate to origin
// 2. Apply scale
// 3. Apply rotation
// 4. Translate to final position
```

### Color Handling
```rust
// KLyric uses CSS-style colors, convert to:
// - tiny-skia: PremultipliedColorU8
// - skia-safe: Color4f
```

### Error Handling
```rust
// Use Result types, avoid panics in WASM
fn parse_effect(s: &str) -> Result<Effect, ParseError> {
    // ...
}
```

## Testing

```bash
# Check compilation for both targets
cargo check -p klyric-renderer
cargo check -p klyric-renderer --target wasm32-unknown-unknown

# Run tests
cargo test -p klyric-renderer
```

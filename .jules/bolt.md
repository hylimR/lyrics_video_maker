## 2024-05-24 - Hoisting Effect Progress Calculation
Learning: Calculating effect progress and easing inside the glyph rendering loop is redundant when the effect parameters (delay, duration) are uniform for the line.
Insight: Even though `EffectType::Transition` might use expressions depending on `char_index`, the *progress* (time 0..1) of the effect depends only on `current_time` and the effect's timing configuration, which are line-constant.
Action: Hoist `EffectEngine::calculate_progress` and `EffectEngine::ease` outside the loop. Pass the pre-calculated `eased_progress` to the inner loop. This saves N (glyphs) * M (effects) expensive math calls (powf, sin, div) per frame.

## 2024-05-24 - Native Build Environment Failure
Learning: The native build environment for `skia-bindings` is broken due to a Segmentation Fault in `clang++` (version 18.1.3) when processing standard library headers.
Insight: This prevents running `cargo test` or `cargo check` for native targets in `klyric-renderer`. Verification must rely on static analysis and potentially `wasm32` targets if applicable (though `line_renderer.rs` is excluded from wasm).
Action: Proceed with extreme caution on logical changes. Document the inability to run local native tests. Future work should investigate pinning a different clang version or using a Docker container for stable builds.

## 2024-05-24 - WASM Renderer Font Resolution Hot Path
Learning: `wasm_renderer.rs` resolves font families to IDs via string hashing and `HashMap` lookups for *every character* in `render_line`.
Insight: While `line_renderer.rs` (native) has some optimizations, the WASM path was missing local caching for font IDs in the hot loop. Since font family rarely changes within a line (unless overridden), we can cache the resolved ID locally.
Action: Implemented a local `cached_family_id` in `render_line` to bypass `TextRenderer::resolve_font_id` overhead for consecutive characters with the same font.

## 2024-05-24 - Hash Map Double Lookup Elimination
Learning: The `if map.contains_key(k) { map.get_mut(k) }` pattern performs hashing and bucket lookup twice.
Insight: Rust's `HashMap` API allows combining these into a single operation using `if let Some(v) = map.get_mut(k)`. For complex updates, helper methods can return `bool` or `Option` to indicate success, allowing the caller to handle the "not found" case without a redundant lookup.
Action: Refactored `ParticleRenderSystem::update_emitter_bounds` to return `bool`, saving one lookup per active particle emitter per frame in the hot render loop.

## 2024-05-24 - Native Build Segfault Verification
Learning: Confirmed `clang++` Segmentation Fault (exit code 139) when compiling `skia-bindings` with `cargo check -p klyric-renderer` on native target.
Insight: The environment's `clang` (version 18.1.3) is incompatible with the current project/system configuration for Skia. This reinforces the need to target optimizations in `wasm32` code paths (like `wasm_renderer.rs`) which use `tiny-skia` and compile successfully.
Action: Pivoted optimization strategy to focus on `wasm_renderer.rs` Paint reuse, which can be verified with `cargo check --target wasm32-unknown-unknown`.

## 2024-05-24 - Style Cloning in Render Loop
Learning: `Style` struct in `klyric-renderer` is a deep tree of `Option<String>` and nested structs, making `clone()` expensive.
Insight: The `wasm_renderer` was cloning `Style` twice per line per frame (once in `render_frame` fallback logic, once in `render_line`). Since `Style` is immutable during rendering, references `&Style` should be passed instead.
Action: Refactored `render_frame` and `render_line` to pass `&Style`, removing deep clones. Also optimized font family caching to use `&str` instead of `String` to avoid allocation on font switch.

## 2024-05-24 - Font Path Caching with Glyph ID
Learning: `TextRenderer::get_glyph_path_by_id` used `(ID, char)` as cache key, causing redundant `char -> GlyphId` lookups and larger map keys.
Insight: The layout engine already calculates `GlyphId` (u16) for every character. Using `(ID, u16)` as the cache key avoids re-calculating the glyph ID and uses a smaller, faster key for hashing.
Action: Updated `path_cache` to use `u16` instead of `char`. Modified `render_line` to pass the pre-calculated `glyph_info.glyph_id`, removing the need for `face.glyph_index(ch)` in the hot path.

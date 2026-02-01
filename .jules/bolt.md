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

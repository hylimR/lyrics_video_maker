## 2024-05-23 - Skia Font Creation Overhead
Learning: Recreating `skia_safe::Font` objects in a loop (e.g., for every character in layout) is expensive because it involves C++ object construction and potentially locking/lookup in Skia.
Insight: Even if the `Typeface` and size are the same, `Font::from_typeface` creates a new object.
Action: Use a simple cache keyed by `(typeface.unique_id(), size_bits)` to reuse `Font` instances across iterations when rendering or measuring text. This is especially important in `layout_line` and `render_line`.

## 2024-05-24 - Text Layout Re-calculation
Learning: `LineRenderer::render_line` calls `LayoutEngine::layout_line` every frame (via `Renderer::render_to_canvas`), causing expensive text measurement and font lookups (even with caching) to repeat ~60 times per second for static text.
Insight: Text layout for a line is deterministic based on the `Line` and `Style`. Since `Renderer` is persistent (unlike `LineRenderer`), it can cache the layout result (`Vec<GlyphInfo>`).
Action: Hoist `layout_line` out of `render_line` and cache the result in `Renderer` keyed by line index (or ID). Pass the cached glyphs to `render_line`. This eliminates per-frame layout overhead.

## 2024-05-25 - Glyph ID Hoisting
Learning: `LineRenderer::render_line` called `font.unichar_to_glyph(ch)` for every glyph in every frame, even though this mapping is constant for a given font and char.
Insight: The `LayoutEngine` already looks up the glyph ID to measure the character width. By storing this `glyph_id` in the `GlyphInfo` struct, we can avoid the repeated lookup during rendering.
Action: Added `glyph_id: u16` to `GlyphInfo` and populated it during layout. Updated `LineRenderer` to use the cached ID. This reduces FFI overhead and potential map lookups in the hot render loop.

## 2024-05-26 - Conditional Font Cache Clearing
Learning: Unconditionally clearing caches (like `resolved_font_cache`) "to prevent memory leaks" is a massive performance footgun for the common case (static content).
Insight: A cache should only be cleared if it is actually in danger of growing too large. For font caches, a threshold of ~500 entries is plenty for static text but prevents infinite growth from animations.
Action: Changed `clear_font_cache` to check `len() > 500` before clearing. This preserves the cache across frames for static text, eliminating the need to rebuild `skia_safe::Font` objects (and their underlying C++ structures) 60 times a second.

## 2024-05-27 - Skia Path Caching
Learning: `font.get_path(glyph_id)` involves FFI overhead and C++ object construction for every glyph in every frame, even for static text.
Insight: `skia_safe::Path` objects are reference-counted (copy-on-write) wrappers around C++ `SkPath`. Cloning them is cheap. We can cache the resulting `Path` object to avoid the FFI/construction cost entirely.
Action: Added `path_cache` to `TextRenderer` keyed by `(typeface_id, size_bits, glyph_id)`. Updated `LineRenderer` to use this cache. This reduces the per-glyph overhead significantly, especially for text-heavy scenes.

## 2024-05-28 - Effect Categorization Hoisting
Learning: `LineRenderer::render_line` was iterating over a mixed list of effects inside the hot glyph loop to find specific types (like `StrokeReveal`).
Insight: Pre-filtering effects into specific vectors (e.g., `stroke_reveal_effects`) outside the loop allows the inner loop to iterate only over relevant items (often zero), avoiding O(N) checks per glyph.
Action: Always hoist effect categorization outside of per-glyph or per-particle loops. Separate "Transform" effects from "Render" effects early.

## 2025-05-20 - TriggerContext Allocation and Vector Reuse
Learning: `LineRenderer::render_line` allocated multiple vectors and cloned `TriggerContext` for every glyph, creating significant allocator pressure per frame.
Insight: Vectors for effects can be populated directly from a single loop over effect names using `Cow<Effect>`, avoiding intermediate allocations. `TriggerContext` can be hoisted outside the glyph loop and passed by reference to `EffectEngine`.
Action: Optimized `render_line` to use a single resolution loop and reuse `TriggerContext`. Updated `EffectEngine::compute_transform` to accept `&TriggerContext` and `Borrow<Effect>`. This reduces allocations and memory traffic in the render loop.

## 2025-05-22 - Environment Compilation Blockers
Learning: The development environment lacks necessary system dependencies (`libasound2-dev` for `alsa-sys`) and the `wasm32-unknown-unknown` target, preventing full compilation and testing of the `klyric-renderer` crate. Additionally, `skia-bindings` compilation causes a clang segmentation fault.
Insight: Code verification must rely on rigorous reading and logic checks when toolchain verification is unavailable.
Action: Rely on expert code analysis and limited verification. Document constraints in PR.

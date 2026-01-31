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

## 2024-05-23 - Skia Font Creation Overhead
Learning: Recreating `skia_safe::Font` objects in a loop (e.g., for every character in layout) is expensive because it involves C++ object construction and potentially locking/lookup in Skia.
Insight: Even if the `Typeface` and size are the same, `Font::from_typeface` creates a new object.
Action: Use a simple cache keyed by `(typeface.unique_id(), size_bits)` to reuse `Font` instances across iterations when rendering or measuring text. This is especially important in `layout_line` and `render_line`.

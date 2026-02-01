## 2024-05-22 - [Performance] Redundant Hex Parsing in Render Loop
Learning: String parsing (`parse_color`) inside the main render loop (`render_line`) is a hidden performance cost. Even with a fast parser, doing it 5-7 times per line per frame adds up (e.g., 20 lines * 7 lookups * 60 fps = 8400 parses/sec).
Action: Implement a cache for resolved values (like `skia_safe::Color`) at the `Renderer` level, keyed by Style name. Pass these resolved values to the inner renderer. This changes O(Frames * Lines) parsing to O(Styles).

## 2024-05-24 - [Performance] StrokeReveal Overhead on Start Delay
Learning: The `StrokeReveal` effect performs expensive `PathMeasure` calculations (O(N) where N is path verbs) every frame for every character, even when the effect hasn't visually started yet (progress 0.0) due to delays.
Action: Short-circuit the effect logic when progress is effectively zero (<= 0.001) by returning an empty path, skipping the expensive `PathMeasure` initialization and segmentation.

## 2024-05-25 - [Performance] Vector Allocation Churn in Render Loop
Learning: `LineRenderer::render_line` allocated 4 new `Vec`s per call (per line, per frame) to track active effects. While small, these allocations add up (e.g., 240+ allocs/sec at 60fps).
Action: Introduced `LineRenderScratch` struct in `Renderer` to hold persistent `Vec` buffers. Passed this scratch struct to `render_line` to reuse capacity, replacing allocations with `clear()` and `push()`. Refactored `CompiledRenderOp` to use `Arc` instead of lifetimes to allow persistent storage.

## 2024-05-26 - [Performance] FFI Overhead in Hot Render Loop
Learning: Calling `path.bounds()` involves an FFI call to C++ Skia logic. Doing this for every character every frame (e.g. 2000 chars * 60 fps = 120,000 FFI calls/sec) creates measurable overhead. Additionally, retrieving the path from a HashMap cache (`get_path_cached`) involves hashing and lookup every frame.
Action: Cached `path` and `bounds` directly in the `GlyphInfo` struct during the `LayoutEngine` phase. Layout happens only when text/style changes, so this moves the cost from O(Frames) to O(LayoutUpdates).

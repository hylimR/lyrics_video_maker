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

## 2024-05-27 - [Correctness/Performance] Reference to Temporary in Expression Context
Learning: In `FastEvaluationContext::get_value`, returning `Some(&Value::Float(PI))` attempts to return a reference to a temporary value, which is invalid Rust and likely caused compilation failures or undefined behavior. It also constructed a new `Value` enum variant on every access.
Action: Added `pi` and `e` fields to the struct to store these constants once. Updated `get_value` to return references to these cached fields. This fixes the correctness issue and avoids construction overhead in the hot expression evaluation loop.

## 2024-05-28 - [Performance] Redundant HashMap Lookups in Particle System
Learning: The particle system performed 3 separate HashMap lookups per character per frame for active emitters: `has_emitter`, `update_bounds`, and `apply_overrides`. For a typical scene with 500 characters, this is 1500 lookups/frame (90k/sec).
Action: Consolidated these operations into a single `update_existing_emitter` method using the `HashMap` Entry API (or explicit `get_mut`), reducing the cost to 1 lookup per character per frame (3x reduction in map overhead).

## 2024-05-30 - [Performance] Canvas Save/Restore Overhead for Simple Translations
Learning: `skia_safe::Canvas::save()` and `restore()` involve stack allocation and clip state management. For simple translations (like shadows and glitch offsets) that don't involve rotation, scale, or clipping, using paired `translate()` calls (translate forward, draw, translate back) is significantly cheaper and avoids the stack overhead.
Action: Replaced `save/restore` blocks with `translate/translate(-)` pairs in the Shadow and Glitch rendering paths in `LineRenderer::render_line`. This is safe because these blocks only perform translation and do not modify other canvas states.

## 2026-02-02 - [Performance] Redundant Shadow/Stroke Parsing in Layout
Learning: `LayoutEngine::layout_line` re-parsed hex color strings for every character in a line, even if they shared the same override values (e.g. strict importers or bulk edits). While less critical than render loops, this added O(N) string parsing overhead to layout updates.
Action: Implemented a local cache (`cached_shadow_hex`, `cached_stroke_hex`) inside the layout loop to reuse parsed `skia_safe::Color` values for consecutive characters with identical hex strings.

## 2026-02-02 - [Performance] Clone Overhead in Particle Hot Loop
Learning: `CharBounds` was implicitly cloned (memcpy via `Clone` trait, explicitly called) in the `render_line` hot loop for every character with an active particle effect. For high character counts (e.g. 1000 chars), this explicit method call adds unnecessary overhead compared to simple stack copying.
Action: Derived `Copy` for the `CharBounds` struct (16 bytes) and removed the explicit `.clone()` call. This allows the compiler to optimize the passing of bounds (likely in registers) and removes the semantic overhead of cloning.

## 2026-02-02 - [Performance] Allocation in Expression Hot Loop
Learning: `FastEvaluationContext::set_index` and `set_progress` were constructing new `Value` enum variants (allocating `Some` and moving data) for every character/op in the render loop. Since `Value` is ~32 bytes, this resulted in significant memory write traffic.
Action: Optimized `set_index` and `set_progress` to update existing enum variants in-place using pattern matching. Added `get_index_raw` to bypass string matching in `TypewriterLimit` checks.

## 2026-02-02 - [Performance] Redundant Constant Ops in Mixed Effect Lists
Learning: `EffectEngine::compile_active_effects` used an "all-or-nothing" optimization. If a single dynamic effect (Expression) was present, ALL constant effects were pushed to the render ops vector, causing redundant iteration in the hot loop.
Action: Implemented granular filtering using `dynamic_seen_mask`. Constant effects are now only pushed to the ops vector if they need to override a preceding dynamic effect for the same property. This reduces the ops vector size in mixed scenarios (e.g. Fade + Wiggle).

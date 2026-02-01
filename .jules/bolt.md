## 2024-05-23 - Hoisting Effect Categorization in Render Loop

Learning: Found a significant O(N) allocation bottleneck in the render loop where vectors were being allocated per-character to categorize effects, despite the effects being constant for the line.
Action: Always look for invariant calculations inside tight loops (like per-character rendering) and hoist them. Even "small" vector allocations add up when done thousands of times per frame.

## 2024-05-23 - Caching Strategy Ignored

Learning: The `Renderer` had a sophisticated caching mechanism (`CategorizedLineEffects`) that was fully implemented but *completely ignored* by the `LineRenderer`. The `LineRenderer` was re-computing the work that was already cached.
Action: When optimizing, first check if existing caches are actually being used. Unconnected caches are a common source of wasted performance.

## 2024-05-24 - Skia Bindings Segfault on Linux
Learning: Native compilation of `klyric-renderer` (specifically `skia-bindings`) can fail due to Clang/GCC header conflicts in certain environments (e.g., SEGFAULTs in `string_view`), preventing local verification of native-only code paths.
Action: When working on native renderers in this codebase, rely on static analysis and partial verification (wasm target if available) if the local environment is broken. Ensure strict adherence to existing patterns.

## 2024-05-24 - Context Reuse in Particle Loop
Learning: Inner loops (like particle spawning inside glyph rendering) were allocating `EvaluationContext` (via `Default::default()`) which not only caused allocation overhead but also used incorrect default dimensions (1920x1080) instead of actual canvas dimensions.
Action: Reuse context objects (`FastEvaluationContext`) from outer scopes whenever possible, updating only the changed fields (`progress`, `index`). This improves performance and ensures consistency (correct dimensions).

## 2024-05-24 - Active Particle Set Allocation in Render Loop
Learning: The renderer was maintaining a `HashSet<u64>` of active particle keys by clearing it every frame and inserting keys inside the inner character loop. This involved hashing, probing, and potential resizing overhead for every character with an active effect, every frame.
Action: Replaced the external `HashSet` tracking with an internal `active` flag on the `ParticleEmitter` objects. By resetting flags at the start of the frame (`reset_active_flags`) and setting them to true when accessed (`ensure_emitter`), we eliminated the `HashSet` entirely, removing O(N) insertions from the hot path.

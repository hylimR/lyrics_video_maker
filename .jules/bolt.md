## 2024-05-23 - Hoisting Effect Categorization in Render Loop

Learning: Found a significant O(N) allocation bottleneck in the render loop where vectors were being allocated per-character to categorize effects, despite the effects being constant for the line.
Action: Always look for invariant calculations inside tight loops (like per-character rendering) and hoist them. Even "small" vector allocations add up when done thousands of times per frame.

## 2024-05-23 - Caching Strategy Ignored

Learning: The `Renderer` had a sophisticated caching mechanism (`CategorizedLineEffects`) that was fully implemented but *completely ignored* by the `LineRenderer`. The `LineRenderer` was re-computing the work that was already cached.
Action: When optimizing, first check if existing caches are actually being used. Unconnected caches are a common source of wasted performance.

## 2024-05-24 - Skia Bindings Segfault on Linux
Learning: Native compilation of `klyric-renderer` (specifically `skia-bindings`) can fail due to Clang/GCC header conflicts in certain environments (e.g., SEGFAULTs in `string_view`), preventing local verification of native-only code paths.
Action: When working on native renderers in this codebase, rely on static analysis and partial verification (wasm target if available) if the local environment is broken. Ensure strict adherence to existing patterns.

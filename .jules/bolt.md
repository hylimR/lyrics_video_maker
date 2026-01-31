## 2024-05-22 - Native Build Environment Issues
Learning: The native build environment for `klyric-renderer` and `klyric-gui` is broken due to `skia-bindings` causing segmentation faults with Clang 18 and GCC 13 headers.
Action: When working on this repository, prioritize `wasm32-unknown-unknown` target for verification of shared or WASM-specific code. Avoid relying on native tests until the environment is fixed.

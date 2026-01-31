## 2024-05-23 - Clang Segmentation Fault with Skia Bindings
Learning: Native compilation of `klyric-renderer` (and `klyric-gui`) fails in this environment due to `clang++` segmentation fault when compiling `skia-bindings` (v0.75.0). This seems related to `string_view` UTF-8 validation errors in GCC 13 headers when parsed by Clang 18.
Action: Verification of native code changes relies on code inspection and logic, as running native tests is not possible. WASM target verification (`cargo check -p klyric-renderer --target wasm32-unknown-unknown`) should be used to ensure shared code validity.

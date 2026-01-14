## 2025-05-15 - Missing WASM Build Artifacts
**Learning:** The frontend requires `src/wasm/klyric_renderer.js` to run, which is a build artifact from `npm run build:wasm`. If this is missing, the app crashes immediately.
**Action:** When running the frontend in a fresh environment, verify if `src/wasm` exists. If not, either run the build command (if Rust is available) or mock the module to allow UI testing.

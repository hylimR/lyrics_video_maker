## 2025-05-19 - WASM Artifact Dependency in Tests
**Learning:** Frontend tests involving WASM components imported from build artifacts (e.g., `src/wasm/klyric_renderer`) fail module resolution if the artifact is missing, even if mocked via `vi.mock`.
**Action:** Create a stub file during test setup or ensure the build process runs before testing. For this task, a temporary stub allowed verification without full build.

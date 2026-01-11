# Project TODO List

This file tracks the pending tasks and improvements for the Lyric Video Maker project.

## 🛠️ High Priority
- [ ] **Project Save/Load**: Implement JSON export/import for the entire project state (Zustand store).
- [ ] **Robust Font Discovery**: Replace hardcoded font paths in `pipeline.rs` with a proper system font discovery mechanism (e.g., using `font-kit`).
- [ ] **FFmpeg Path Consistency**: Ensure `muxer.rs` uses the same FFmpeg discovery logic as `encoder.rs`.
- [ ] **Error Handling**: Improve error reporting in the video export pipeline, especially for FFmpeg-related failures.

## 🎨 Rendering & Effects
- [ ] **Background Media**: Add support for background images and videos in the render pipeline.
- [ ] **More Effect Presets**: Implement additional visual effects in both WASM and native renderers.
- [ ] **Preview Optimization**: Optimize the JPEG encoding throttle in `pipeline.rs` for smoother real-time preview during export.

## 🖥️ Desktop & UI
- [ ] **Packaging**: Finalize Tauri configuration for production builds and installers.
- [ ] **Multi-Window Sync**: Further refine the P2P sync logic for better performance in the full preview window.
- [ ] **Audio Visualization**: Improve the waveform view in the K-Timing Editor for better accuracy.

## 🧪 Testing & CI
- [ ] **Integration Tests**: Add end-to-end tests for the video export pipeline.
- [ ] **Unit Tests**: Expand Rust crates' unit tests for parser and layout engines.
- [ ] **CI Pipeline**: Set up GitHub Actions for automated builds and testing.

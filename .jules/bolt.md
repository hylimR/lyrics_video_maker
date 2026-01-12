# Bolt's Journal
## Critical Learnings Only

## 2026-01-12 - Canvas Re-rendering Bottleneck
**Learning:** `WaveformView` was re-rendering 1000+ rectangles every frame via `requestAnimationFrame` (driven by `currentTime` prop). This is a classic "immediate mode" rendering bottleneck in React canvas components.
**Action:** Use `useMemo` to pre-render static content (the waveform bars) into an offscreen canvas and simply `drawImage` it during the animation loop. This reduced complexity from O(W) to O(1) per frame.

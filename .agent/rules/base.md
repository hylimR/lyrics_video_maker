---
trigger: always_on
---

# Lyric Video Maker - Agent Handover Documentation

**Last Updated:** 2026-01-04  
**Project Path:** `d:\Workspace\lyric_video_maker_claude`  
**Dev Server:** `npm run dev --host` (runs on http://localhost:5173)

---

## 📋 Project Overview

A web-based **Lyric Video Generator** built with React + Vite + PixiJS v8. The application allows users to:
- Import lyrics from ASS/SSA karaoke files with per-character (syllable) timing
- Preview lyrics rendering with various visual effects
- Edit syllable timing using a K-Timing Editor (Aegisub-style)
- Sync state across multiple browser tabs in real-time

---

## 🏗️ Architecture

### Technology Stack
| Layer | Technology |
|-------|------------|
| Framework | React 18 + Vite |
| Rendering | PixiJS v8 (WebGL) |
| State Management | **Zustand** (Global Store + Persistence) |
| State Sync | BroadcastChannel API (Middleware) |
| Styling | Vanilla CSS (glassmorphic dark theme) |
| Routing | react-router-dom v6 |

### Single Source of Truth ("One-Way Data Flow")
The app uses a **Zustand Store** as the centralized source of truth for all application state.

```
┌─────────────────┐      BroadcastChannel      ┌─────────────────┐
│   Master Tab    │ ◄──────────────────────►   │   Client Tab    │
│  (Main Editor)  │                            │ (Preview/KTime) │
│                 │                            │                 │
│ ┌─────────────┐ │       STATE_SYNC           │                 │
│ │Zustand Store│ │ ────────────────────────►  │  Replaces Local │
│ │ + Persist   │ │                            │      Store      │
│ │ + History   │ │ ◄───────────────────────   │                 │
│ └─────────────┘ │      REQUEST_UPDATE        │                 │
│                 │                            │                 │
└─────────────────┘                            └─────────────────┘
```

- **Master Tab**: Use `useAppStore` in write-mode. Saves to `localStorage`, broadcasts updates.
- **Client Tabs**: Use `useAppStore` in read-only mode (mostly). UI actions send `REQUEST_UPDATE` messages to Master.
- **Persistence**: Zustand `persist` middleware automatically saves/loads sync state (Lyrics, Resolution, Settings).

---

## 📁 Key File Structure

```
src/
├── App.jsx                    # Main application orchestrator
├── App.css                    # Main styles
├── main.jsx                   # Entry point with routes
│
├── constants/
│   └── index.js               # Centralized configuration (effects, resolutions, sync)
│
├── store/
│   └── useAppStore.js         # GLOBAL STATE STORE (Logic, Sync, History)
│
├── components/
│   ├── PixiCanvas.jsx         # PixiJS WebGL canvas wrapper
│   ├── PixiCanvas.css         # Canvas styling
│   ├── ControlPanel.jsx       # Playback controls + settings
│   ├── LyricsEditor/          # Text-based lyrics editor (modular)
│   ├── KTimingEditor/         # Aegisub-style syllable timing (modular)
│   │   ├── index.jsx          # Main K-Timing editor
│   │   ├── WaveformView.jsx   # Audio waveform visualization
│   │   ├── DraggableTimeline.jsx
│   │   ├── CharacterBoxes.jsx
│   │   └── CharacterPropertyPanel.jsx
│   ├── FileUploader.jsx       # ASS/WAV file import
│   └── MasterOfflineOverlay.jsx
│
├── pages/
│   ├── PreviewPage.jsx        # Full-screen preview (/preview)
│   └── KTimingPage.jsx        # Standalone K-Timing (/ktiming)
│
├── core/
│   ├── LyricRenderer.js       # Main rendering engine
│   └── LyricChar.js           # Per-character lyric model
│
├── effects/
│   ├── EffectManager.js       # Visual effects orchestrator
│   ├── ParticleSystem.js      # Particle effects engine
│   └── presets/               # Modular effect implementations
│       ├── index.js           # Effect preset exports
│       ├── utils.js           # Shared effect utilities
│       ├── basicEffects.js    # blur, wobbly, scalePop, colorShift, pulseGlow
│       └── advancedEffects.js # particles, flip3D, wave3D, typewriter, shatter
│
├── hooks/
│   ├── useLyricSync.js        # Lyric timing calculations
│   ├── useSimulatedAudio.js   # Demo mode audio simulation
│   └── useAudioSync.js        # Audio playback sync logic
│
└── utils/
    ├── lyricParsers.js        # ASS/SSA/LRC parsing utilities
    ├── karaokeUtils.js        # Per-character timing utilities
    ├── KLyricFormat.js        # KLyric format converter (NEW!)
    └── timeUtils.js           # Time formatting helpers
```

---

## 🔧 Core Systems

### 1. Zustand Store (`src/store/useAppStore.js`)
**Purpose:** Replaces all previous `Context` and `SyncManager` logic.

**State Shape:**
```javascript
{
  // Content (Persisted)
  lyrics: [...],
  resolution: { width: 1920, height: 1080 },
  selectedEffect: 'blur',
  duration: 28,

  // Playback (Transient)
  currentTime: 0,
  isPlaying: false,

  // System
  isMaster: boolean,
  isMasterOnline: boolean,

  // History
  past: [...],
  future: [...]
}
```

**Key Actions:**
- `updateState(updates)`:
    - **Master**: Updates store, saves history, broadcasts `STATE_SYNC`.
    - **Client**: Sends `REQUEST_UPDATE` to Master.
- `setPlayback(playback)`:
    - **Master**: Updates store, broadcasts `PLAYBACK_SYNC`.
    - **Client**: Sends `PLAYBACK_REQUEST` to Master.
- `undo()` / `redo()`: Managed by Master history stack.

### 2. LyricRenderer (`src/core/LyricRenderer.js`)
**Purpose:** PixiJS-based lyric rendering
- **init()**: Initialize PixiJS containers.
- **update()**: Frame update loop.
- **setEffect()**: Switch visual strategy.

---

## 🎨 Visual Effects System

Effects are located in `src/core/effects/` and registered in `EffectManager.js`.

| Effect | File | Description |
|--------|------|-------------|
| `blur` | BlurFadeEffect.js | Characters fade in with blur |
| `neonGlow` | NeonGlowEffect.js | Neon glow with color shift |
| `scalePop` | ScalePopEffect.js | Scale bounce animation |
| `shatter` | ShatterEffect.js | Shatter/explode effect |

---

## 📝 KLyric Format System (v1.0)

KLyric is a new JSON-based lyric format that extends ASS capabilities.

### Key Features
- **Per-character transforms**: position, rotation, scale, opacity
- **CSS-like styling**: inheritance, classes, inline styles
- **Keyframe animations**: similar to CSS `@keyframes`
- **Individual character timing**: start/end per character
- **Parent-child hierarchies**: relative transforms
- **Layout modes**: horizontal, vertical, path-based

### File Structure
```
.agent/specs/
├── KLYRIC_FORMAT_SPEC.md     # Full format specification
└── klyric-schema.json        # JSON Schema for validation
src/utils/
└── KLyricFormat.js           # Parser/converter implementation
```

### Usage
```javascript
import { importSubtitleToKLyric, klyricToLegacy, klyricToASS } from '@/utils/KLyricFormat';

// Import any format → KLyric
const { klyric, legacy, format } = importSubtitleToKLyric(content, 'song.ass');

// Convert KLyric → legacy format (for LyricRenderer compatibility)
const legacyLyrics = klyricToLegacy(klyricDoc);

// Export KLyric → ASS format
const assContent = klyricToASS(klyricDoc);
```

### Data Flow
```
[ASS/SRT/LRC File] 
    ↓ importSubtitleToKLyric()
[KLyric Document] ←→ [Editor edits]
    ↓ klyricToLegacy()
[Legacy Format] → LyricRenderer → PixiJS
    ↓ klyricToASS()
[ASS Export]
```

## 🛤️ Routes

| Route | Component | Description |
|-------|-----------|-------------|
| `/` | App | Main editor with integrated K-Timing (master) |
| `/preview` | PreviewPage | Full-screen preview (client) |

---

## ✅ Features Implemented

### Core Features
- [x] ASS/SSA file parsing with K-timing support
- [x] PixiJS v8 WebGL rendering
- [x] Multiple visual effects
- [x] Demo mode with simulated audio
- [x] Real audio playback support

### Cross-Tab Sync (Zustand)
- [x] Master/Client architecture via Global Store
- [x] Real-time state synchronization
- [x] Playback sync (time + play/pause)
- [x] Master heartbeat detection
- [x] Client offline overlay when master dies

### Undo/Redo System
- [x] Stack-based history (50 steps max) built into Store
- [x] Ctrl+Z / Ctrl+Shift+Z shortcuts
- [x] Session persistence to localStorage (via Zustand persist)

### K-Timing Editor
- [x] Integrated panel in main editor (always visible)
- [x] Split-layout with compact preview on left, editor on right
- [x] Keyboard-driven syllable marking
- [x] Responsive design (stacks vertically on smaller screens)
- [x] Enhanced Character Property Panel with tabs (Transform/Effects)
- [x] Per-character transform controls (offset, scale, rotation, opacity)
- [x] Per-character effect and animation selection

### KLyric Format System (v1.0)
- [x] JSON-based format specification (see `.agent/specs/KLYRIC_FORMAT_SPEC.md`)
- [x] JSON Schema for validation (`klyric-schema.json`)
- [x] Automatic conversion: ASS/SRT/LRC → KLyric on import
- [x] KLyric document stored in Zustand global state
- [x] Export to KLyric (.klyric) or ASS format
- [x] Export panel with copy-to-clipboard support

---

## ⚠️ Known Issues / Gotchas

1. **BroadcastChannel Singleton**: The `BroadcastChannel` in `useAppStore.js` is a **module-level singleton**. DO NOT close it in the `useEffect` cleanup or it will break subsequent re-mounts (especially in Development/Strict Mode).

2. **Master Election**: If `localStorage` heartbeat is stale, a new tab might claim Master status. The Logic includes an `IAM_MASTER` conflict resolution (older tab ID wins usually, but simple yielding implemented).

---

## 🦀 Tauri + Rust Backend (NEW!)

The project now includes a Tauri integration for native desktop app capabilities:

### Directory Structure
```
src-tauri/
├── Cargo.toml          # Rust dependencies
├── tauri.conf.json     # Tauri configuration
├── src/
│   ├── main.rs         # Application entry point
│   ├── lib.rs          # Library exports
│   ├── commands/       # Tauri IPC commands
│   ├── renderer/       # KLYRIC rendering engine (Rust)
│   └── video/          # FFmpeg integration
```

### Features (In Progress)
- **Native Video Export**: H.264/H.265/VP9/AV1 via FFmpeg
- **Rust Rendering Engine**: High-performance frame rendering with tiny-skia
- **KLYRIC Parser**: Full KLYRIC format support in Rust
- **System Font Discovery**: Cross-platform font enumeration

### Frontend Hook
```javascript
import { useVideoExport } from '@/hooks/useVideoExport';

const { startRender, progress, isRendering, cancelRender } = useVideoExport();
```

### Status
See `TODO_TAURI_RUST_BACKEND.md` for detailed implementation progress.

---

## 🚀 Potential Next Steps

| Feature | Priority | Notes |
|---------|----------|-------|
| Video Export | High | ✅ Tauri backend ready, needs FFmpeg integration |
| Save/Load Project | High | JSON export/import of state |
| Background Images/Videos | Medium | Add to PixiJS render pipeline |
| Desktop App Release | Medium | Complete FFmpeg integration + packaging |

---

## 🖥️ Development Commands

```bash
# Start web dev server
npm run dev --host

# Build for web production
npm run build

# Start Tauri desktop app (requires Rust)
npm run tauri:dev

# Build Tauri desktop app
npm run tauri:build
```

---

## 📝 Quick Reference

### Add a new state property
1. Add property to `useAppStore` definition (`src/store/useAppStore.js`).
2. Add to `persist` whitelist if it should be saved.
3. Use `updateState({ myProp: newValue })` to update it.
4. Access via `const { myProp } = useAppStore()` in components.

### Add a new Tauri command
1. Create function in `src-tauri/src/commands/`.
2. Add `#[tauri::command]` attribute.
3. Register in `main.rs` `invoke_handler`.
4. Call from frontend: `invoke('command_name', { args })`.

### Debug sync issues
1. Check Console Logs for `👑` (Master) or `👤` (Client).
2. Check `localStorage` key `lyric-video-storage` for persisted data.
3. Verify Master Heartbeat in `lyric-video-master-last-seen`.

---

*End of Handover Documentation*

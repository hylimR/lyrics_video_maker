# Current UI Structure & Identifiers

This document lists the identified UI panels and their corresponding component identifiers to assist in the redesign process.

## Top Level Structure

*   **Top Navigation** (`app.navbar`)
    *   **File:** `src/App.jsx`
    *   **Contains:**
        *   Logo & Status
        *   Undo/Redo History Buttons
        *   Mode Toggle (Demo vs Audio)
        *   Main Actions:
            *   Full Preview (New Tab)
            *   Toggle Editor (Overlay)
            *   Toggle Import (Overlay)
            *   Toggle Export (Overlay)

*   **Main Content Layout** (`layout.main`)
    *   **File:** `src/App.jsx`
    *   **Description:** Split-screen layout containing the Preview and the K-Timing Editor.

### 1. Preview Section (`layout.preview_section`)
*   **File:** `src/App.jsx` -> `src/components/WasmPreview.jsx`
*   **Identifier:** `preview.canvas`
*   **Description:** The main visual output area using the WASM renderer.
*   **Sub-components:**
    *   `preview.badges`: Resolution and Effect indicators.
    *   `preview.global_style` (`src/components/GlobalStyleEditor.jsx`): Panel below the canvas for editing global font/transform settings.

### 2. K-Timing Editor Section (`layout.ktiming_section`)
*   **File:** `src/components/KTimingEditor/index.jsx`
*   **Identifier:** `ktiming.container`
*   **Sub-components:**
    *   `ktiming.header`: Line selector dropdown and time info.
    *   `ktiming.waveform`: Audio waveform visualization.
    *   `ktiming.timeline`: Draggable timeline for syllable adjustments.
    *   `ktiming.properties`:
        *   `ktiming.properties.character` (`CharacterPropertyPanel`): Selected character settings.
        *   `ktiming.properties.line` (`StyleEditor`): Line-level style settings.
    *   `ktiming.controls`: Playback controls, loop toggle, auto-split, and navigation buttons.

### 3. Bottom Player Bar (`app.control_panel`)
*   **File:** `src/components/ControlPanel.jsx`
*   **Identifier:** `app.control_panel`
*   **Description:** YouTube Music style bottom bar.
*   **Contains:**
    *   Timeline scrub bar (full width top).
    *   Playback Controls (Play/Pause/Time).
    *   Current Lyric Display.
    *   Quick Settings (Effect & Resolution Selectors).

## Overlays & Modals

### Import Panel (`overlay.file_panel`)
*   **File:** `src/components/FileUploader.jsx` (wrapped in `src/App.jsx`)
*   **Description:** File upload interface for `.ass`, `.srt`, `.lrc` lyrics and audio files.

### Lyrics Editor (`overlay.lyrics_editor`)
*   **File:** `src/components/LyricsEditor/index.jsx`
*   **Description:** Text-area based editor for raw text manipulation.

### Export Panel (`overlay.export_panel`)
*   **File:** `src/components/ExportPanel.jsx`
*   **Description:** Options to export to .klyric, .ass, or render Video.

## Component Map

| Identifier | Component File | Description |
| :--- | :--- | :--- |
| `app.navbar` | `src/App.jsx` | Top navigation bar |
| `app.control_panel` | `src/components/ControlPanel.jsx` | Bottom player bar |
| `layout.preview` | `src/components/WasmPreview.jsx` | Main preview canvas |
| `preview.global_style` | `src/components/GlobalStyleEditor.jsx` | Global style constraints |
| `ktiming.editor` | `src/components/KTimingEditor/index.jsx` | Main timing logic |
| `ktiming.timeline` | `src/components/KTimingEditor/DraggableTimeline.jsx` | Syllable timeline |
| `overlay.file_panel` | `src/components/FileUploader.jsx` | File import |
| `overlay.export_panel` | `src/components/ExportPanel.jsx` | Export options |

# Lyric Video Maker

A professional **Lyric Video Generator** built entirely in **Rust**. This tool allows users to create high-quality lyric videos with advanced timing (K-Timing), per-character animations, and visual effects, utilizing the **Iced** GUI framework and **Skia** rendering engine.

## 🚀 Project Overview

**Core Features:**
- **K-Timing Editor:** Precise per-character timing adjustments.
- **Real-time Preview:** High-performance preview using a custom Rust renderer (`klyric-renderer`).
- **Rich Styling:** CSS-like styling (stroke, shadow, fill, fonts) and particle effects.
- **Native Export:** High-quality video encoding via FFmpeg.
- **Cross-Platform:** Runs natively on Linux, Windows, and macOS.

## 🛠️ Tech Stack

| Component | Technology | Description |
|-----------|------------|-------------|
| **GUI** | Iced 0.13 | Native, Elm-inspired GUI framework |
| **Language** | Rust | Core logic, safety, and performance |
| **Rendering** | Skia (via `skia-safe`) | Industry-standard 2D graphics engine |
| **Audio** | Rodio | Audio playback and synchronization |
| **Video** | FFmpeg | Video encoding pipeline |
| **Format** | KLyric v2.0 | Custom JSON format for rich lyrics styling |

## 📂 Architecture

The workspace is organized into three main crates:

```
crates/
├── klyric-gui/          # Main Desktop Application
│   └── Built with Iced. Handles UI, state management, and user interaction.
│
├── klyric-renderer/     # Core Rendering Engine
│   └── Pure Rust library. Handles parsing, layout, effects, and drawing.
│   └── Supports dual targets: Native (skia-safe) and WASM (tiny-skia).
│
└── klyric-preview/      # Standalone Previewer
    └── Lightweight OpenGL preview window using winit + glutin.
```

## ⚡ Getting Started

### Prerequisites

1.  **Rust Toolchain**: Install via [rustup.rs](https://rustup.rs).
2.  **FFmpeg**: Must be installed and available in your system PATH.
3.  **System Dependencies** (Linux only):
    ```bash
    sudo apt install libasound2-dev libglib2.0-dev libgtk-3-dev pkg-config clang lld ninja-build python3
    ```

### Running the Application

To run the main GUI editor:

```bash
cargo run -p klyric-gui
```

### Building for Release

```bash
cargo build --release --workspace
```

The binary will be available at `target/release/klyric`.

## 🧪 Development

### Running Tests

We maintain comprehensive test coverage for the rendering engine.

```bash
# Run all tests
cargo test --workspace

# Run renderer tests only
cargo test -p klyric-renderer
```

### Key Concepts

- **AppState:** The single source of truth for the UI state, defined in `crates/klyric-gui/src/state.rs`.
- **Message:** All user actions generate a `Message` enum variant, handled by the `update` function in `app.rs`.
- **Renderer:** The `klyric-renderer` crate is independent of the GUI and can be used in headless environments.

## ⚠️ Notes

- **Font Discovery:** The application currently relies on embedded fonts or specific system fonts.
- **WASM Support:** The renderer has a WASM target for potential web-based previews, utilizing `tiny-skia`.

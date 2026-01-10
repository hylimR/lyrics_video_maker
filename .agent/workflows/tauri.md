---
description: Build and run the Tauri desktop application
---

# Tauri Desktop App Workflow

This workflow describes how to build and run the Lyric Video Maker as a native desktop application using Tauri.

## Prerequisites

Before using Tauri, ensure you have:

1. **Rust toolchain** installed:
   ```powershell
   # Install via winget (Windows)
   winget install Rustlang.Rustup
   
   # Or download from https://rustup.rs/
   ```

2. **Visual Studio C++ Build Tools** (Windows only):
   ```powershell
   winget install Microsoft.VisualStudio.2022.BuildTools --override "--add Microsoft.VisualStudio.Workload.VCTools"
   ```

3. **Verify installation**:
   ```powershell
   rustc --version
   cargo --version
   ```

## Development

// turbo
1. Start the Tauri development server (runs both Vite and Tauri):
   ```powershell
   npm run tauri:dev
   ```

   This will:
   - Start the Vite dev server on http://localhost:5173
   - Build and launch the Tauri desktop window
   - Enable hot reloading for both frontend and backend changes

## Production Build

2. Build the production application:
   ```powershell
   npm run tauri:build
   ```

   This creates platform-specific installers in:
   - Windows: `src-tauri/target/release/bundle/msi/` and `nsis/`
   - macOS: `src-tauri/target/release/bundle/dmg/`
   - Linux: `src-tauri/target/release/bundle/appimage/`

## Project Structure

```
src-tauri/
├── Cargo.toml          # Rust dependencies
├── tauri.conf.json     # Tauri configuration
├── build.rs            # Build script
├── icons/              # App icons (see README inside)
└── src/
    ├── main.rs         # Application entry point
    ├── lib.rs          # Library exports
    ├── commands/       # Tauri IPC commands
    │   ├── mod.rs
    │   ├── render.rs   # Video rendering
    │   └── export.rs   # File export
    ├── renderer/       # KLYRIC rendering engine
    │   ├── mod.rs
    │   ├── klyric.rs   # KLYRIC format parser
    │   ├── frame.rs    # Frame generation
    │   └── text.rs     # Text rendering
    └── video/          # FFmpeg integration
        ├── mod.rs
        ├── encoder.rs  # Video encoding
        └── muxer.rs    # Audio/video muxing
```

## Common Issues

### "rustc not found"
Rust is not installed. Run the installation commands above.

### "link.exe not found" (Windows)
Visual Studio Build Tools are not installed or not in PATH.
Install them and restart your terminal.

### Build takes too long
First build downloads and compiles dependencies. Subsequent builds are faster.
Debug builds are faster than release builds.

## Frontend Integration

Use the `useVideoExport` hook in React components:

```jsx
import { useVideoExport } from '@/hooks/useVideoExport';

function ExportButton({ klyricDocument }) {
  const { startRender, progress, isRendering, cancelRender } = useVideoExport();

  const handleExport = async () => {
    await startRender(klyricDocument, '/path/to/audio.mp3', {
      width: 1920,
      height: 1080,
      fps: 30,
    });
  };

  if (isRendering) {
    return (
      <div>
        <progress value={progress.percentage} max="100" />
        <button onClick={cancelRender}>Cancel</button>
      </div>
    );
  }

  return <button onClick={handleExport}>Export Video</button>;
}
```

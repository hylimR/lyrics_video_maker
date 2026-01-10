# Tauri App Icons

This directory should contain app icons in various sizes for different platforms.

## Required Icons

### Windows
- `icon.ico` - Windows application icon (contains multiple sizes)

### macOS
- `icon.icns` - macOS application icon bundle

### Linux / General
- `32x32.png` - 32x32 pixel icon
- `128x128.png` - 128x128 pixel icon
- `128x128@2x.png` - 256x256 pixel icon (Retina)

## Generating Icons

You can generate all required icons from a single high-resolution source image (1024x1024 PNG recommended) using:

```bash
# Option 1: Using Tauri's built-in icon generator
npx tauri icon path/to/source-icon.png

# Option 2: Manual creation using ImageMagick
# Windows ICO (includes 16, 32, 48, 256):
magick source.png -define icon:auto-resize=256,48,32,16 icon.ico

# macOS ICNS:
# Use iconutil on macOS or img2icns

# PNG sizes:
magick source.png -resize 32x32 32x32.png
magick source.png -resize 128x128 128x128.png
magick source.png -resize 256x256 128x128@2x.png
```

## Placeholder

Until real icons are created, Tauri will use default icons.
To create custom icons, design a logo that works well at small sizes
and represents the Lyric Video Maker application.

Suggested design elements:
- Musical notes or waveform
- Video/play symbol
- Text/typography element
- Vibrant colors matching the app theme

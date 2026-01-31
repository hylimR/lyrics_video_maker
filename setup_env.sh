#!/bin/bash
set -e

echo "Checking for root privileges..."
if [ "$EUID" -ne 0 ]; then
  echo "Please run as root (use sudo)"
  exit 1
fi

echo "Updating package lists..."
apt-get update

echo "Installing System Dependencies (FFmpeg, Audio, Graphics)..."
# Dependencies from README + ffmpeg
apt-get install -y \
    ffmpeg \
    libasound2-dev \
    libglib2.0-dev \
    libgtk-3-dev \
    pkg-config \
    clang \
    lld \
    ninja-build \
    python3 \
    build-essential

echo "System dependencies installed successfully."

# Check for Rust
if command -v rustc &> /dev/null; then
    echo "Rust is already installed: $(rustc --version)"
else
    echo "Rust is NOT installed."
    echo "To install Rust, please run the following command as your NORMAL user (not root):"
    echo ""
    echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo ""
fi

# Quota-Re (Quote Engine Rust)

A highly performant, memory-safe rewrite of the Python Quote Engine using Rust, Axum, and Skia. 

## Features (1:1 with Python Version)
- **Telegram Stickers (`/sticker`)**: Generates 512px WebP stickers mimicking QuotLy. Features native color emoji, complex script shaping (Arabic/RTL), circular avatars, and the exact Telegram bubble dimensions/shine.
- **Classic Quotes (`/quote`)**: High-res JPEG quotes over custom backgrounds with hardware-accelerated Gaussian blur.
- **Audio Cards (`/audio-card`)**: PNG music player cards with progress bars and dynamic timestamps.
- **File Cards (`/file-card`)**: PNG file attachment cards.

## Why Skia?
By utilizing `skia-safe`, we replaced Pillow (PIL), `arabic_reshaper`, and `unicode-bidi`. Skia natively parses TrueType/OpenType fonts, handles HarfBuzz shaping internally, and draws Noto Color Emoji flawlessly out of the box.

## Architecture
- **`src/api/`**: Axum HTTP routing and zero-copy multipart form extraction.
- **`src/engine/`**: Skia surface rendering, bounding-box math, and canvas drawing.
- **`src/fonts/`**: Skia `FontCollection` initialization and fallback chains.

## Local Setup

### Prerequisites
Skia requires a C++ build environment (LLVM/Clang) to compile its bindings.
- **Linux (Ubuntu/Debian)**: `sudo apt install clang libclang-dev`
- **macOS**: `xcode-select --install`
- **Windows**: Install LLVM and ensure it's in your PATH.

### Running the Server
```bash
# Clone the repo
git clone https://github.com/sidwiskers/quota-re.git
cd quota-re

# Run the server on port 5000
PORT=5000 RUST_LOG=info cargo run --release
```

## API Testing
You can test the API using curl:

```bash
# Test Sticker Generation
curl -X POST http://localhost:5000/sticker \
  -F "username=Siddhartha" \
  -F "message=This is rendering natively in Rust with Skia! 🚀" \
  -F "theme=dark" \
  -o sticker.webp
```

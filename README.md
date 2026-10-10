# Quota-Re (Quote Engine Rust)

Quota-Re is a Rust HTTP service for rendering Telegram-style stickers and quote, audio, and file cards. It uses Axum for HTTP and Skia for image rendering and text shaping.

## Endpoints

All generation endpoints accept `multipart/form-data` and return image bytes directly.

### `POST /sticker`

Required fields:
- `username`: up to 128 characters.
- `message`: up to 4,096 characters.

Optional fields:
- `theme`: `dark` or `light` (defaults to `dark`).
- `avatar_file`: image upload.
- `reply_username` and `reply_message`: supply both to add a reply block.

Returns `image/webp` with a transparent 512 × 512 canvas.

### `POST /quote`

Required fields:
- `quote_text`: up to 4,000 characters.
- `author_name`: up to 256 characters.

Optional fields:
- `quote_style`: `0` (horizontal), `2` (vertical), or `3` (cloud card); defaults to `0`.
- `output_format`: `jpg`, `jpeg`, or `png`; defaults to `jpg`.
- `avatar_file`: image upload.
- `bg_file`: background image upload.

The response content type matches the selected encoding.

### `POST /audio-card`

Required fields:
- `title`: up to 256 characters.
- `performer`: up to 256 characters.

Optional fields:
- `duration`: duration in seconds, from 0 to 86,400; defaults to 0.
- `progress`: playback progress from 0 to 1; defaults to 0.
- `thumb_file`: thumbnail image upload.

Returns `image/png`.

### `POST /file-card`

Required fields:
- `file_name`: up to 512 characters.
- `file_size_str`: up to 64 characters.

Optional fields:
- `file_ext`: up to 16 characters.
- `thumb_file`: thumbnail image upload.

Returns `image/png`.

## Limits and behavior

- Requests are limited to 12 MiB.
- Uploaded images are checked for encoded size, dimensions, and total pixel count before being rendered.
- Two render jobs may run at once. When both slots are busy, the service returns HTTP 429 instead of building an unbounded rendering queue.
- Missing or invalid required fields return HTTP 400. Oversized text returns HTTP 413.
- `GET /health` returns a simple JSON health response.

## Run locally

Install Rust (stable), a C++ build toolchain, Clang, Ninja, FreeType, Fontconfig, Roboto, and Noto Color Emoji. On Debian/Ubuntu:

```sh
sudo apt-get update
sudo apt-get install -y build-essential clang libclang-dev libfontconfig1-dev libfreetype6-dev ninja-build fontconfig fonts-roboto fonts-noto-color-emoji
```

Then run:

```sh
PORT=5000 RUST_LOG=info cargo run --release
```

## Run with Docker

```sh
docker compose up --build -d
```

The service listens on port 5000 by default. Set `PORT` to change it.

## Tests

```sh
cargo test
```

The tests cover image format signatures, accepted quote formats, input validation, and safe image input handling. Rendering tests require the native Skia build dependencies.

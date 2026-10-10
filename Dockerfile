# Build stage
FROM rust:bookworm AS builder

# Dependencies required to compile Skia and its native text stack.
RUN apt-get update && apt-get install -y --no-install-recommends \
    clang \
    python3 \
    libfontconfig1-dev \
    libfreetype6-dev \
    ninja-build \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

# Keep native Skia compilation bounded on memory-constrained builders.
ENV SKIA_NINJA_COMMAND="ninja -j 1"
ENV CARGO_BUILD_JOBS=2

RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    fontconfig \
    fonts-roboto \
    fonts-noto-color-emoji \
    libfontconfig1 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/quota-re /app/quota-re

EXPOSE 5000
ENV PORT=5000
ENV RUST_LOG=info

CMD ["./quota-re"]

# Build stage
FROM rust:bookworm AS builder

# Install dependencies required by Skia to compile
RUN apt-get update && apt-get install -y \
    clang \
    python3 \
    libfontconfig1-dev \
    libfreetype6-dev \
    ninja-build \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the source code
COPY . .

# Restrict Ninja (the C++ build system) to 1 concurrent job to prevent 
# Out-Of-Memory (OOM) crashes on constrained systems like Cloud Shell.
ENV SKIA_NINJA_COMMAND="ninja -j 1"

# Build the release binary
RUN cargo build --release

# Runtime stage (Smaller final image)
FROM debian:bookworm-slim

# Install runtime dependencies (fontconfig is needed by Skia for text layout)
RUN apt-get update && apt-get install -y \
    libfontconfig1 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/quota-re /app/quota-re

# Expose the API port
EXPOSE 5000

# Environment variables
ENV PORT=5000
ENV RUST_LOG=info

# Run the server
CMD ["./quota-re"]

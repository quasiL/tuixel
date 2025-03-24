#!/bin/bash

# Application name
APP_NAME=tuixel

# Rust Docker image tag
# Available image tags:
# 1-bullseye, 1.84-bullseye, 1.84.0-bullseye, bullseye
# 1-slim-bullseye, 1.84-slim-bullseye, 1.84.0-slim-bullseye, slim-bullseye
# 1-bookworm, 1.84-bookworm, 1.84.0-bookworm, bookworm, 1, 1.84, 1.84.0, latest
# 1-slim-bookworm, 1.84-slim-bookworm, 1.84.0-slim-bookworm, slim-bookworm, 1-slim, 1.84-slim, 1.84.0-slim, slim
# 1-alpine3.20, 1.84-alpine3.20, 1.84.0-alpine3.20, alpine3.20
# 1-alpine3.21, 1.84-alpine3.21, 1.84.0-alpine3.21, alpine3.21, 1-alpine, 1.84-alpine, 1.84.0-alpine, alpine
RUST_IMAGE_TAG=bullseye

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT_DIR="$SCRIPT_DIR/output"

mkdir -p "$OUTPUT_DIR"

docker run --rm \
  --workdir /usr/src/app \
  -v "$PROJECT_ROOT":/usr/src/app \
  -v "$PROJECT_ROOT/target":/usr/src/app/target \
  -v "$OUTPUT_DIR":/output \
  -e CARGO_HOME=/usr/local/cargo \
  rust:$RUST_IMAGE_TAG \
  bash -c "cargo build --release && cp target/release/$APP_NAME /output/"

echo "Build complete. The executable is saved in '$OUTPUT_DIR'."
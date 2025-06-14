#!/bin/bash

# build all!
TARGETS=(
    "x86_64-unknown-linux-gnu"
    "x86_64-pc-windows-gnu"
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
    "aarch64-unknown-linux-gnu"
)

echo "Installing targets..."
for target in "${TARGETS[@]}"; do
    rustup target add "$target"
done

echo "Building for all targets..."
for target in "${TARGETS[@]}"; do
    echo "Building for $target..."
    cargo build --release --target "$target"
    if [ $? -eq 0 ]; then
        echo "✅ Successfully built for $target"
    else
        echo "❌ Failed to build for $target"
    fi
done

echo "Build complete! Binaries are in target/*/release/"
#!/bin/bash
# fixed_build.sh

# Make sure the wasm32 target is installed
echo "Ensuring wasm32 target is installed..."
rustup target add wasm32-unknown-unknown

# Clean previous build artifacts
echo "Cleaning workspace..."
rm -rf out
rm -rf target

# Create the output directory
echo "Creating output directory..."
mkdir -p out

# Build process with specific flags for WASM
echo "Building WASM package..."
cd renderer-wasm

# Force rebuild of dependencies
cargo update -p getrandom --precise 0.2.8

# Build with explicit target and features
RUSTFLAGS=--cfg=web_sys_unstable_apis cargo build --target wasm32-unknown-unknown --release

# Check if the build was successful
if [ ! -f "../target/wasm32-unknown-unknown/release/renderer_wasm.wasm" ]; then
    echo "Error: Build failed. WASM file not created."
    exit 1
fi

# Run wasm-bindgen on the generated wasm
echo "Running wasm-bindgen..."
wasm-bindgen --target web --out-dir ../out --no-typescript ../target/wasm32-unknown-unknown/release/renderer_wasm.wasm

# Back to root directory
cd ..

# Copy the HTML interface to the output directory
echo "Copying HTML files..."
cp index.html out/

echo "Build complete! Files are in the 'out' directory."
echo "Run a local web server in the 'out' directory to test:"
echo "  cd out && python -m http.server"
#!/bin/bash
# clean.sh - Reset cargo state and rebuild

# Remove existing lock file and build artifacts
echo "Cleaning workspace..."
rm -f Cargo.lock
rm -rf target
# rm -rf out

# Clean individual crates
cd simulation
cargo clean
cd ..

cd renderer-wasm
cargo clean
cd ..

if [ -d "renderer-native" ]; then
  cd renderer-native
  cargo clean
  cd ..
fi

# Rebuild with updated dependencies
echo "Building simulation crate..."
cd simulation
cargo build
cd ..

echo "Building WASM renderer..."
cd renderer-wasm
cargo build
cd ..

echo "Running wasm-pack build..."
cd renderer-wasm
wasm-pack build --target web --out-dir ../out
cd ..

# Copy the HTML interface to the output directory
echo "Copying HTML files..."
cp index.html out/

echo "Build complete! Files are in the 'out' directory."
echo "Run a local web server in the 'out' directory to test:"
echo "  cd out && python -m http.server"
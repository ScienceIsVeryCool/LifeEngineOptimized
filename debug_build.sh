#!/bin/bash
# debug_build.sh

# Set up verbose output and error handling
set -x  # Print each command as it's executed
set -e  # Exit on first error

# Create the output directory
mkdir -p out

# Show Rust and cargo versions
echo "Checking Rust and cargo versions..."
rustc --version
cargo --version
wasm-bindgen --version

# Show the installed wasm target
echo "Checking wasm32 target..."
rustup target list | grep wasm32

# Check the Cargo.toml files
echo "Checking simulation Cargo.toml..."
cat simulation/Cargo.toml

echo "Checking renderer-wasm Cargo.toml..."
cat renderer-wasm/Cargo.toml

# Try to compile just the simulation crate
echo "Compiling simulation crate..."
cd simulation
cargo build
cd ..

# Compile the WASM target with more verbose output
echo "Compiling WASM target with verbose output..."
cd renderer-wasm
RUSTFLAGS="--cfg=web_sys_unstable_apis -C debuginfo=2" cargo build --target wasm32-unknown-unknown -v
cd ..

echo "Debug build completed"
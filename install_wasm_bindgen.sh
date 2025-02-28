#!/bin/bash
# install_wasm_bindgen.sh

# Install the specific version of wasm-bindgen-cli that matches our dependency
WASM_BINDGEN_VERSION=0.2.84

echo "Installing wasm-bindgen-cli version ${WASM_BINDGEN_VERSION}..."
cargo install --force wasm-bindgen-cli --version=${WASM_BINDGEN_VERSION}

echo "wasm-bindgen-cli installation complete!"
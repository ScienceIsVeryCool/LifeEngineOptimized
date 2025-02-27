# Use the official Rust image
FROM rust:latest

# Set the working directory inside the container
WORKDIR /app

# (Optional) Pre-copy dependencies to take advantage of caching:
# COPY Cargo.toml Cargo.lock ./
# RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src
RUN cargo install wasm-bindgen-cli
# Copy the full project into the container (if not using volumes)
# COPY . .

# Default command: open a shell so you can run cargo commands interactively
CMD ["bash"]

#/bin/bash
set -e

# TODO: Use a build.rs

# TODO: Build frontend app
pushd frontend
cargo build --release --target wasm32-unknown-unknown
popd

# Package wasm into backend static files
wasm-bindgen \
    frontend/target/wasm32-unknown-unknown/release/frontend.wasm \
    --target web --out-dir server/static/



# Build and run backend.
pushd server
cargo r

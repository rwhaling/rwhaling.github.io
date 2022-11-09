#!/bin/bash
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/release/$(basename $(pwd)).wasm --out-dir wasm --no-modules --no-typescript

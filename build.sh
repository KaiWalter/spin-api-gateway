#!/bin/sh

cargo component build --manifest-path api1/Cargo.toml --target wasm32-unknown-unknown
cargo component build --manifest-path api-gateway/Cargo.toml --target wasm32-wasip1
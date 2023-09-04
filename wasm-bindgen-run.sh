#!/bin/bash
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --no-typescript --out-dir ./docs --target web ./target/wasm32-unknown-unknown/release/asteriods.wasm
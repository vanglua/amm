#!/bin/bash
set -e

RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release
cp ./target/wasm32-unknown-unknown/release/amm.wasm ./res
cp ./target/wasm32-unknown-unknown/release/dao.wasm ./res
cp ./target/wasm32-unknown-unknown/release/oracle.wasm ./res
cp ./target/wasm32-unknown-unknown/release/token.wasm ./res
#!/bin/bash
set -e

pushd flux
./scripts/build.sh
cp ../res/flux.wasm ./tests/wasm/
popd
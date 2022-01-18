#!/bin/bash
set -e
cd "`dirname $0`"
package_name=$(cat Cargo.toml | sed -ne 's/^name = "\(.*\)"$/\1/p')
rm -f "./res/$package_name.wasm"

RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release
cp "target/wasm32-unknown-unknown/release/$package_name.wasm" "./res/$package_name.wasm"
touch -r target/wasm32-unknown-unknown/release/$package_name.wasm ./res/$package_name.wasm

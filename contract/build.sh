#!/bin/bash
set -e
cd "`dirname $0`"
package_name=$(sed Cargo.toml -ne 's/^name = "\(.*\)"$/\1/p')
rm "./res/$package_name.wasm"

RUSTFLAGS='-C link-arg=-s' cargo +nightly build --target wasm32-unknown-unknown --release
cp "target/wasm32-unknown-unknown/release/$package_name.wasm" ./res/

#!/bin/bash
set -e
cd "`dirname $0`"
cargo +nightly build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/lock_unlock_account.wasm ./res/

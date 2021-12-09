#!/bin/bash
set -e
cd "`dirname $0`"
bash ./build.sh
echo cargo +nightly test $@
cargo test $@

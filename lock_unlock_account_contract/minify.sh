#!/bin/bash

if [[ "$#" -ne 2 ]]; then
  echo "usage: $0 in_file.wasm out_file.wasm" 1>&2
  exit 1
fi

in_filename="$1"
out_filename="$2"

w=$(basename -- $in_filename)
echo "Minifying $w, make sure it is not stripped"
wasm-snip $in_filename --snip-rust-fmt-code --snip-rust-panicking-code -p 'core::num::flt2dec::.*' -p 'core::fmt::float::.*' --output temp-$w
wasm-gc temp-$w
wasm-strip temp-$w
wasm-opt -Oz temp-$w --output opt_result_$w
rm temp-$w
mv opt_result_$w $out_filename
echo $w `stat -c "%s" $in_filename` "bytes ->" `stat -c "%s" $out_filename` "bytes, see $out_filename"

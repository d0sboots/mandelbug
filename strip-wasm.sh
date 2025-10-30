#!/bin/bash
set -euxo pipefail
cd -- "${BASH_SOURCE%/*}"
wasm-opt -O3 --strip-producers --strip-target-features web_wasm/target/wasm32-unknown-unknown/release/mandel_wasm.wasm -o - |\
wasm2wat --enable-annotations - |\
sed -e '/(memory (/d;/(global /d;/(export "_/d;s/(@custom\(.*\)))$/(@custom\1)/' |\
sed -e '$a\
  (memory (;0;) 4))' |\
wat2wasm --enable-annotations - -o out.wasm
./include_wasm.py

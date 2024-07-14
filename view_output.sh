#!/bin/bash
OUT_FILE="target/out.rs"
echo "fn main() {" > "$OUT_FILE"

echo "" >> src/lib.rs # Force cargo to recompile
rustfmt src/lib.rs

"$@" >> "$OUT_FILE"

# tests print their output after compilation finished, so delete everything after the "running <N> tests" line
sed --in-place --silent '/running [0-9]* tests/q;p' "$OUT_FILE"

echo "}" >> "$OUT_FILE"
rustfmt "$OUT_FILE"

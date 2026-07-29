#!/usr/bin/env bash
set -euo pipefail
cd "${SRC:?}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$SRC/target}"
export CARGO_NET_OFFLINE=true
export CARGO="${CARGO:-cargo}"

# Hermetic: dependencies are vendored under $SRC/vendor (see .cargo/config.toml).
"$CARGO" build --manifest-path fuzz/Cargo.toml --release --offline

for b in load_run_fuzzer typeflow_fuzzer deser_fuzzer assemble_fuzzer; do
  cp -f "target/release/$b" "$OUT/$b"
  if [ -d "fuzz/corpus/${b}" ]; then
    (cd "fuzz/corpus/${b}" && zip -q -r "$OUT/${b}_seed_corpus.zip" .)
  fi
done

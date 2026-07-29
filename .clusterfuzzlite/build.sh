#!/usr/bin/env bash
set -euo pipefail
cd "${SRC:?}"

# Hermetic offline build: all crates live under $SRC/vendor (.cargo/config.toml).
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$SRC/target}"
export CARGO_HOME="${CARGO_HOME:-$SRC/.cargo_home}"
export CARGO_NET_OFFLINE=true
export CARGO="${CARGO:-cargo}"
mkdir -p "$CARGO_HOME"

if [[ ! -f .cargo/config.toml ]]; then
  echo "missing .cargo/config.toml (vendored-sources)" >&2
  exit 1
fi
if [[ ! -d vendor ]]; then
  echo "missing vendor/ directory" >&2
  exit 1
fi

"$CARGO" build --manifest-path fuzz/Cargo.toml --release --offline --locked

for b in load_run_fuzzer typeflow_fuzzer deser_fuzzer assemble_fuzzer; do
  cp -f "${CARGO_TARGET_DIR}/release/$b" "$OUT/$b"
  if [[ -d "fuzz/corpus/${b}" ]]; then
    (cd "fuzz/corpus/${b}" && zip -q -r "$OUT/${b}_seed_corpus.zip" .)
  fi
done

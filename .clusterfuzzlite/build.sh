#!/usr/bin/env bash
        set -euo pipefail
        cd "${SRC:?}"
        export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$SRC/target}"
        export PATH="${HOME}/.cargo/bin:${PATH}"
        if command -v rustup >/dev/null 2>&1 && rustup toolchain list 2>/dev/null | grep -q nightly; then
          export RUSTFLAGS="-Zsanitizer=address"
          cargo +nightly build --manifest-path fuzz/Cargo.toml --release
        else
          cargo build --manifest-path fuzz/Cargo.toml --release
        fi
        for b in load_run_fuzzer typeflow_fuzzer deser_fuzzer assemble_fuzzer; do
          cp -f "target/release/$b" "$OUT/$b"
        done

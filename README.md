# cellvm

Compact stack bytecode VM with an explicit **load → verify → link → execute** pipeline.

The verifier (safe Rust) establishes frame and type invariants. The executor uses
`unsafe` fast paths that rely on those invariants holding. Optional tooling covers
disassembly, CFG construction, stack maps, and static analyses.

## Layout

| Stage | Crate surface |
|-------|----------------|
| Load | `loader`, `encode` |
| Verify | `verifier::{frame,typeflow}`, `validate_extra` |
| Link | `intern` |
| Execute | `exec`, `runtime` |
| Tooling | `asm`, `disasm`, `cfg`, `optimize`, `analysis` |

Binary modules use the `CLVM` container (code + constants + optional string/function trailers).

## Build

```bash
cargo test
cargo build --bin poc_driver
cargo run --example verify_blob -- fixtures/hello.clvm   # if present
```

## Fuzz

ClusterFuzzLite harnesses live under `fuzz/`:

- `load_run_fuzzer`
- `typeflow_fuzzer`
- `deser_fuzzer`
- `assemble_fuzzer`

```bash
cargo +nightly fuzz run load_run_fuzzer
```

## License

MIT

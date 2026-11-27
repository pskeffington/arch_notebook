# SNG-DELIVERY-002 G0 CPU vector geometry evidence

- **Source baseline:** `3ada2479766d026ee497925436ad6b87c7d5ea72` on `codex/ui-00-chart-baseline`; measurements were built from the candidate worktree after that commit.
- **Candidate source file:** `src/main.rs`, SHA-256 `e068d87d7e69c9a7660c0f89f875c14bf217e9cce30b23a5d912f491ea44b1a3`.
- **Measurement build:** release profile, Arch Notebook 0.1.0, built offline from the candidate worktree on 2026-11-26.
- **Host:** Linux x86_64, Intel Core i7-6700K @ 4.00 GHz.
- **Binary SHA-256:** `e7e91afc143ac11d4cec1d14699a8b84dc7bfb570fde3005911afc247d7bed64`.
- **Command:** `python3 scripts/benchmark-chart-vector-geometry.py --binary /tmp/arch-notebook-target/release/arch-notebook --output /tmp/arch-notebook-chart-vector-benchmark-25samples.json --iterations 25` — exit 0; 12 cases completed.
- **Fixture generation:** deterministic fictional fixtures from `scripts/generate-chart-fixtures.py`; each case used 25 timed iterations after three warmups. Fixture hashes, p95/min/max, and full JSON are committed in [the machine-readable report](artifacts/SNG-DELIVERY-002-G0-vector-cpu-25samples.json), SHA-256 `144a206771b2c4ef47d00840f128eaf4cb15cdc2021cb6636c3807d6775fa1f0`.

The table compares current chart vector preparation, entity-index preparation, and the actual sorted pointer cycle-candidate scan with a proxy for the previous pointer path, which repeated vector geometry preparation before running that same scan. Both paths share the same picking function, pointer location, canvas and fixture. Values are same-process medians from one local run; they are CPU operation timings, not input-to-display latency.

| Fixture | Entities | Relationship records | Current frame prep + pick (µs) | Previous duplicate-prep proxy (µs) | Proxy/current |
| --- | ---: | ---: | ---: | ---: | ---: |
| Sparse | 100 | 300 | 1,223.6 | 2,139.7 | 1.75× |
| Sparse | 1,000 | 3,000 | 3,252.1 | 5,220.0 | 1.61× |
| Sparse | 10,000 | 30,000 | 28,003.4 | 46,367.0 | 1.66× |
| Hub-heavy | 100 | 300 | 437.3 | 699.2 | 1.60× |
| Hub-heavy | 1,000 | 3,000 | 4,338.4 | 7,343.8 | 1.69× |
| Hub-heavy | 10,000 | 30,000 | 30,508.0 | 54,516.8 | 1.79× |
| Disconnected | 100 | 200 | 287.7 | 524.2 | 1.82× |
| Disconnected | 1,000 | 2,000 | 1,622.4 | 2,801.9 | 1.73× |
| Disconnected | 10,000 | 20,000 | 19,809.7 | 35,833.2 | 1.81× |
| Dense local clusters | 100 | 1,000 | 3,307.0 | 5,206.3 | 1.57× |
| Dense local clusters | 1,000 | 10,000 | 8,346.5 | 15,028.6 | 1.80× |
| Dense local clusters | 10,000 | 100,000 | 110,349.0 | 187,305.4 | 1.70× |

**Result:** PASS for CPU geometry reuse on these synthetic workloads. The measured path removes a second lane/position/route preparation from pointer handling while preserving the exact ordered pointer candidate list. Current path medians were 1.57×–1.82× faster than the duplicate-preparation proxy on this host, with 25 timed samples per case. This does not establish a universal speedup or validate full graph rendering performance.

**Not measured:** chart painting, text shaping, memory peak, GPU, compositor, presentation, interactive input-to-paint latency, or FPS. The native visual/input gate remains BLOCKED until the application can be exercised on an input-capable desktop. The force-directed layout is implemented separately and its CPU/native evidence is recorded in [G3 workload evidence](G3-community-force-benchmark-linux.md); it is not included in this vector benchmark.

**Companion implementation checks:** `cargo fmt --all -- --check`, `git diff --check`, `python3 -m py_compile scripts/benchmark-chart-vector-geometry.py`, `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline` (51 passed), `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline`, and `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` all exited 0. The report records complete timing distributions; the measurements exclude painting, presentation, and layout.

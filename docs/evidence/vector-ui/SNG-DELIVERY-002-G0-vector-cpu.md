# SNG-DELIVERY-002 G0 CPU vector geometry evidence

- **Source baseline:** `3ada2479766d026ee497925436ad6b87c7d5ea72` on `codex/ui-00-chart-baseline`; measurements were built from the candidate worktree after that commit.
- **Candidate source file:** `src/main.rs`, SHA-256 `ecb6561e873761e0a670ea3f84f15745ecff5d127756e1348db2715fb89babe7`.
- **Measurement build:** release profile, Arch Notebook 0.1.0, built offline from the candidate worktree on 2026-11-26.
- **Host:** Linux x86_64, Intel Core i7-6700K @ 4.00 GHz.
- **Binary SHA-256:** `ffdc1912b5a9f0fd562d548b64dac1cac519f92e9ee228d6d447fa84286914d1`.
- **Command:** `python3 scripts/benchmark-chart-vector-geometry.py --binary /tmp/arch-notebook-target/release/arch-notebook --output /tmp/arch-notebook-chart-vector-benchmark-25samples.json --iterations 25` — exit 0; 12 cases completed.
- **Fixture generation:** deterministic fictional fixtures from `scripts/generate-chart-fixtures.py`; each case used 25 timed iterations after three warmups. Fixture hashes, p95/min/max, and full JSON are committed in [the machine-readable report](artifacts/SNG-DELIVERY-002-G0-vector-cpu-25samples.json), SHA-256 `2eb10f1da085fa9a909f9818e873b73c272852f7beafdfd67d182ff4b45d6148`.

The table compares current chart vector preparation, entity-index preparation, and the actual sorted pointer cycle-candidate scan with a proxy for the previous pointer path, which repeated vector geometry preparation before running that same scan. Both paths share the same picking function, pointer location, canvas and fixture. Values are same-process medians from one local run; they are CPU operation timings, not input-to-display latency.

| Fixture | Entities | Relationship records | Current frame prep + pick (µs) | Previous duplicate-prep proxy (µs) | Proxy/current |
| --- | ---: | ---: | ---: | ---: | ---: |
| Sparse | 100 | 300 | 566.7 | 1,000.0 | 1.76× |
| Sparse | 1,000 | 3,000 | 3,590.0 | 6,199.4 | 1.73× |
| Sparse | 10,000 | 30,000 | 27,459.2 | 48,366.0 | 1.76× |
| Hub-heavy | 100 | 300 | 662.5 | 1,082.3 | 1.63× |
| Hub-heavy | 1,000 | 3,000 | 2,180.0 | 4,261.6 | 1.95× |
| Hub-heavy | 10,000 | 30,000 | 29,951.5 | 51,916.0 | 1.73× |
| Disconnected | 100 | 200 | 194.0 | 322.1 | 1.66× |
| Disconnected | 1,000 | 2,000 | 4,105.6 | 7,423.2 | 1.81× |
| Disconnected | 10,000 | 20,000 | 22,753.9 | 39,333.3 | 1.73× |
| Dense local clusters | 100 | 1,000 | 3,877.8 | 6,098.4 | 1.57× |
| Dense local clusters | 1,000 | 10,000 | 8,166.8 | 13,864.7 | 1.70× |
| Dense local clusters | 10,000 | 100,000 | 101,963.3 | 174,152.2 | 1.71× |

**Result:** PASS for CPU geometry reuse on these synthetic workloads. The measured path removes a second lane/position/route preparation from pointer handling while preserving the exact ordered pointer candidate list. Current path medians were 1.57×–1.95× faster than the duplicate-preparation proxy on this host, with 25 timed samples per case. This does not establish a universal speedup or validate full graph rendering performance.

**Not measured:** chart painting, text shaping, memory peak, GPU, compositor, presentation, interactive input-to-paint latency, or FPS. The native visual/input gate remains BLOCKED until the application can be exercised on an input-capable desktop. Force-directed layout remains unimplemented.

**Companion implementation checks:** `cargo fmt --all -- --check`, `git diff --check`, `python3 -m py_compile scripts/benchmark-chart-vector-geometry.py`, `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` (42 passed), `cargo check --locked --offline`, and `cargo build --release --locked --offline` all exited 0. The report records complete timing distributions; the measurements exclude painting, presentation, and layout.

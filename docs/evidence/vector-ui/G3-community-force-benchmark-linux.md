# G3 community-aware force layout — Linux workload evidence

- **Task/owner:** G3 layout performance follow-up / Codex implementation
- **Source baseline:** `dc53df2805397bc90215a2f23139a768bfb97c66` on `codex/ui-00-chart-baseline`
- **Candidate:** uncommitted working tree containing the asynchronous arrangement and benchmark CLI; source commit will be recorded in Git history.
- **Fixture source:** `scripts/generate-chart-fixtures.py`; deterministic fictional sparse, hub, disconnected, and dense graphs.
- **Raw data:** [25-sample JSON](artifacts/G3-community-force-25samples.json), SHA-256 `c0fd5ca8c9976e58ea29851bbe12cc3c35c7506bccc641f60b7503710bdd1314`.
- **Release binary:** `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `4666b683af0cb9f59e9bffdae81a69389916b9a56082103181104e0ee4c0c1a1`.
- **Host:** Linux x86_64, kernel `7.3.5-zen1-1-zen`, glibc 2.44.

## Reproduction and results

| Command/check | Result |
| --- | --- |
| `cargo fmt --all` | **PASS**, exit 0 |
| `python3 -m py_compile scripts/benchmark-community-layout.py` | **PASS**, exit 0 |
| `git diff --check` | **PASS**, exit 0 |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` | **PASS**, 45 passed, 0 failed |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` | **PASS**, exit 0 |
| `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` | **PASS**, exit 0 |
| `python3 scripts/benchmark-community-layout.py --binary /tmp/arch-notebook-target/release/arch-notebook --output /tmp/arch-notebook-g3-layout-benchmark-final.json --iterations 20 --samples 25` | **PASS**, 12 cases, exit 0 |

Layout coordinate timings below are medians and p95s over 25 samples after one warmup; all use 20 force iterations. Community detection was timed once separately for each fixture and is not included in the repeated layout timings.

| Fixture | Nodes | Relationship records | Communities | Community detection (one sample) | Layout median | Layout p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Sparse | 1,000 | 3,000 | 61 | 6.01 ms | 9.66 ms | 15.12 ms |
| Sparse | 5,000 | 15,000 | 310 | 21.72 ms | 43.47 ms | 68.38 ms |
| Sparse | 10,000 | 30,000 | 635 | 37.78 ms | 84.88 ms | 87.30 ms |
| Hub | 1,000 | 3,000 | 393 | 13.05 ms | 8.54 ms | 17.97 ms |
| Hub | 5,000 | 15,000 | 2,087 | 76.85 ms | 43.90 ms | 47.14 ms |
| Hub | 10,000 | 30,000 | 4,259 | 232.16 ms | 89.61 ms | 92.30 ms |
| Disconnected | 1,000 | 2,000 | 237 | 2.22 ms | 8.76 ms | 11.42 ms |
| Disconnected | 5,000 | 10,000 | 1,192 | 7.96 ms | 44.46 ms | 82.85 ms |
| Disconnected | 10,000 | 20,000 | 2,372 | 24.33 ms | 90.28 ms | 94.83 ms |
| Dense | 1,000 | 10,000 | 444 | 1.74 ms | 11.13 ms | 11.48 ms |
| Dense | 5,000 | 50,000 | 2,230 | 16.53 ms | 47.34 ms | 49.67 ms |
| Dense | 10,000 | 100,000 | 4,462 | 45.33 ms | 96.38 ms | 100.14 ms |

The hub community detection timing was notably expensive. Both detection and layout now run off the UI thread. The worker applies results only when the active chart ID, ordered entities and positions, pin states, and ordered relationship endpoints still match its input snapshot; stale results are discarded. Regression coverage checks chart switch, entity movement, and endpoint changes. A chart-scoped **Restore positions from before force layout** action is now available when entity order and pin states still match; source tests cover those guards, but native interaction is unreviewed.

## Acceptance and limits

- Candidate CPU timing characterization: **PASS for measured 1k/5k/10k synthetic fixtures at 20 iterations**. Direct comparison against a G0 layout baseline was **NOT RUN**. These are machine-specific timings, not service-level guarantees.
- Native visual/input acceptance: **NOT RUN**. Thread responsiveness and perceived layout quality still need an input-capable desktop review.
- Restore/undo UX: **SOURCE PASS; NATIVE REVIEW NOT RUN**. The one-action restore is transient for the open case session and is not preserved across app restart.
- This measures the CPU coordinate routine and a single community-detection sample only. It does not measure rendering, drawing, compositor, input-to-display latency, or full-app save/load interaction.
- The layout is an exploratory display arrangement; positions do not establish social distance, causality, or importance.

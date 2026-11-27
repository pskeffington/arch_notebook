# G3: G0 Connectivity versus Community Force (debug profile)

- **Task/owner:** SNG-DELIVERY-002 G3 benchmark follow-up / Codex implementation
- **Source baseline:** `f03db522e30cc4e87d01de9432bbbcff51494a2e` on `codex/ui-00-chart-baseline`; candidate source includes the uncommitted changes described below at capture time.
- **Changed source paths:** `src/layout.rs`, `src/main.rs`.
- **Fixture runner:** `scripts/benchmark-community-layout.py`.
- **Raw results:** [paired benchmark JSON](artifacts/G3-connectivity-vs-force-debug-20261126.json), SHA-256 `18af2a94150ef5f1b7269888f69cb69c4e7872837f1ca0c99a2d7d8c50aa815c`.
- **Binary profile:** debug, `/tmp/arch-notebook-target/debug/arch-notebook`, SHA-256 `79ed1775d6116026b43df31c5f3861dc6f4daf84dee0fdaaf4af1b6720e1b2fd`.
- **Host:** Linux x86_64, kernel `7.3.5-zen1-1-zen`, glibc 2.44.

## Method

The existing production G0 Connectivity placement was extracted into `layout::connectivity_layout` and the UI now calls that same function. The benchmark invokes it and the production `layout::force_layout` on the same fixture nodes and pinned coordinates. Each layout gets one warmup and 25 timed samples; sample order alternates. Community Force uses 20 iterations and the fixed benchmark seed; Connectivity is a one-pass degree sort and ring placement. Community detection is reported separately as a single untuned sample. Coordinate calculation is timed; checksum calculation and black-box retention happen after each timed interval.

Fixture JSON is generated in a temporary directory; the benchmark reads but does not mutate it. Hashes, input sizes, compiler profile binary hash, iteration count, sample count, coordinate checksums, medians, and p95s are stored in the raw JSON.

## Results

Times are milliseconds from the development/debug binary on this host. These values are not release-performance estimates.

| Fixture | Nodes | Records | Communities | Detection (one sample) | G0 Connectivity median / p95 | Community Force median / p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Sparse | 1,000 | 3,000 | 61 | 88.53 ms | 0.81 / 0.83 ms | 166.77 / 170.62 ms |
| Hub | 1,000 | 3,000 | 393 | 89.17 ms | 0.82 / 0.87 ms | 215.97 / 252.33 ms |
| Disconnected | 1,000 | 2,000 | 237 | 17.70 ms | 0.86 / 0.92 ms | 215.08 / 236.11 ms |
| Dense | 1,000 | 10,000 | 444 | 17.91 ms | 0.89 / 1.22 ms | 268.13 / 307.66 ms |

## Gate result and limits

- **Paired coordinate benchmark:** **PASS** for execution and data capture on the current debug binary. The production Connectivity baseline is about 207–300 times faster by median in these runs, because it assigns one degree-sorted ring while Community Force performs 20 iterative layout steps. The algorithms do different work and produce different arrangements; this ratio is not an optimization win/loss or an equal-quality comparison.
- **Correctness:** the Connectivity helper preserves the previous degree-descending order, UUID tie-break, ring coordinates, and pinned positions. A regression test covers degree ordering and pin preservation. The full 53-test suite and offline `cargo check`/debug build passed.
- **Release profile:** **BLOCKED**. Cargo's release target remained locked with no visible owner process. Attempts to build were interrupted after waiting; the preexisting release binary is stale and was not used for these results.
- **Native G3 arrange/restore:** **NOT RUN**. No current-candidate native UI claim is made.
- **G4 rendering:** **NOT MEASURED**. These are CPU coordinate timings only; they do not measure vector paint, text shaping, GPU, compositor, presentation, frame rate, or input-to-display latency.

## Reproduction

```sh
cargo fmt --all -- --check
git diff --check
CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1
CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline
CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline
python3 scripts/benchmark-community-layout.py \
  --binary /tmp/arch-notebook-target/debug/arch-notebook \
  --output /tmp/g3-connectivity-vs-force.json \
  --iterations 20 --samples 25 --sizes 1000 \
  --families sparse hub disconnected dense
```

Observed exit codes: formatting `0`, diff check `0`, tests `0` (53 passed), check `0`, debug build `0`, benchmark `0` (four cases). Release build remains blocked as stated above.

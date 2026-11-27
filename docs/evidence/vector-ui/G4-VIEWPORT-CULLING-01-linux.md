# G4-VIEWPORT-CULLING-01 · conservative route culling

- **Task / owner:** Reduce chart drawing work for routes outside the viewport and add a reproducible cull-count benchmark; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Baseline:** `15b9e403a7b064e1a4235dc9e2f79ca5b5013928`.
- **Build:** Release binary SHA-256 `1a7331856c8498fa72cad1148a3a31c11a20d1a29b967226d9ee517334879392`.
- **Fixture:** Synthetic fictional chart workloads only; canvas 1440×900; zoom 1; 5 samples per case.

## Results

| Gate | Result | Evidence |
| --- | --- | --- |
| Conservative route intersection | PASS | `chart_link_route_culling_preserves_crossings_and_nearby_decorations` covers horizontal/diagonal viewport crossings, nearby decoration margin, offscreen routes, empty paths, and conservative non-finite handling. |
| Release workload and cull counts | PASS | Eight cases cover sparse, hub, disconnected, and dense graphs at 100 and 1,000 entities. At neutral pan all valid routes remained visible; with a 2,000-pixel horizontal pan all valid routes were classified offscreen. No missing endpoints occurred in these fixtures. |
| Rust tests | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` exited 0; 55 passed. |
| Offline check and builds | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline`, `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline`, and `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` exited 0. |
| Formatting / whitespace | PASS | `cargo fmt --all -- --check` and `git diff --check` exited 0. |
| Paint speed / native presentation | NOT MEASURED | The benchmark counts candidate routes after conservative culling. It does not measure paint time, GPU, compositor, frame rate, or input-to-presentation latency; native visual review remains open. |

## Reproduction

```sh
python3 scripts/benchmark-chart-vector-geometry.py \
  --binary /tmp/arch-notebook-target/release/arch-notebook \
  --output /tmp/arch-notebook-chart-culling-20261126.json \
  --iterations 5 --sizes 100 1000 --zoom 1 --pan 0 0

python3 scripts/benchmark-chart-vector-geometry.py \
  --binary /tmp/arch-notebook-target/release/arch-notebook \
  --output /tmp/arch-notebook-chart-culling-pan-20261126.json \
  --iterations 5 --sizes 100 1000 --zoom 1 --pan 2000 0
```

Both commands exited 0 and returned eight cases. The pan-zero JSON SHA-256 is `e8e92fd5097e437f6b1c884d8793bbc12ec767076a0712050205a976a8d8c6e0`; the panned JSON SHA-256 is `955504945f02221b237edf6f46441922e26816d36118540674f0b2fb75606554`. Raw JSON is preserved in [neutral-pan results](artifacts/G4-viewport-culling-neutral-20261126.json) and [panned results](artifacts/G4-viewport-culling-panned-20261126.json).

The route AABB is expanded by `140 × |zoom| + 8` logical points to retain nearby labels, attributes, arrows, and strokes. The implementation preserves ambiguous/non-finite inputs conservatively. These results validate cull classification on named synthetic fixtures; they do not establish paint-time improvement or full decoration-bound correctness for every chart.

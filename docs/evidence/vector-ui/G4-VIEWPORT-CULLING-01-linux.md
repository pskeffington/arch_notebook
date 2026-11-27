# G4-VIEWPORT-CULLING-01 · conservative route culling

- **Task / owner:** Reduce chart drawing work for routes outside the viewport and add a reproducible cull-count benchmark; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Product candidate:** `e27ad5e842dd1c6d503f05c51844f12790502fa6`; **verification source:** `c406d814ba146d26cb1419b4a6f13b524fccd820` (headless paint-path regression driven through chart pointer events).
- **Build:** `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `4a373a10ace60f738232247d44f5eee16e0d289b6d755788e8e9c2255cb436d0`.
- **Fixture:** Synthetic fictional chart workloads only; canvas 1440×900; zoom 1; 5 samples per case.

## Results

| Gate | Result | Evidence |
| --- | --- | --- |
| Conservative route intersection | PASS | `chart_link_route_culling_preserves_crossings_and_nearby_decorations` covers straight and curved routes with outside endpoints, an offscreen-anchored self-loop reaching the viewport, a high-lane self-loop label, nearby attribute decorations, fully offscreen routes, empty paths, and conservative non-finite handling. |
| Headless paint-path regression | PASS | `headless_chart_paint_culls_offscreen_relationship_labels` renders the actual chart UI with 40 `CONTACTS` records, simulates pointer press/move/release across a 2,000-pixel drag, and confirms the chart pans and emitted relationship-label text shapes drop by more than 10. The cull's early `continue` precedes both route strokes and labels, exercising the real draw-skip path. This counts CPU-side egui shapes, not renderer time. Focused test and full-suite runs exited 0. |
| Release workload and cull counts | PASS | Eight cases cover sparse, hub, disconnected, and dense graphs at 100 and 1,000 entities. Neutral pan retained every valid record; a 2,000-pixel horizontal pan classified all valid records as offscreen. No missing endpoints occurred. The cull unions sampled route bounds with actual label bounds, adds allowance for attribute rows, and expands for arrow/stroke extent by `90 × |zoom| + 8` logical points. |
| Rust tests | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` exited 0; 57 passed. |
| Offline check and builds | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline`, `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline`, and `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --release --locked --offline` exited 0. |
| Formatting / whitespace | PASS | `cargo fmt --all -- --check` and `git diff --check` exited 0. |
| Paint speed / native presentation | NOT MEASURED | The benchmark counts candidate routes after conservative culling. It does not measure paint time, GPU, compositor, frame rate, or input-to-presentation latency; native visual review remains open. |
| Independent headless test review | PASS | The reviewer confirmed the pointer event path and shape-count scope by inspection; no independent test run or native rendering claim. See the [review handoff](G4-paint-path-headless-review-20261126.md). |

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

Both commands exited 0 and returned eight cases. The pan-zero JSON SHA-256 is `3a310a9e65c26117c76f5439b950746296a1dac2147dc00e204ccebd9d28237c`; the panned JSON SHA-256 is `59380bc5e68a679845010a4645be02c4bc78c572c93c1e923a2c99b897c90d45`. Raw JSON is preserved in [neutral-pan results](artifacts/G4-viewport-culling-neutral-20261126.json) and [panned results](artifacts/G4-viewport-culling-panned-20261126.json).

The cull evaluates each relationship record separately, preserving parallel, reversed, duplicate, and self-loop routes independently. Picking remains unchanged. The headless shape check establishes that the actual UI omits offscreen relationship draw commands in its output; it does not measure paint-time improvement, GPU/compositor cost, frame rate, input-to-presentation latency, or every possible chart decoration. Native visual/input review remains open.

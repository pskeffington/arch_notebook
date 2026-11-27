# G4-VIEWPORT-CULLING-01 · conservative route culling

- **Task / owner:** Reduce chart drawing work for routes outside the viewport and add a reproducible cull-count benchmark; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Culling source:** `4c132cf` (`Add chart-wide bridge proxy ranking and culling`). The later working-tree changes to `src/main.rs` update the headless proxy-focus fixture and its button label; they do not change culling. The benchmark binary below was built before those UI-only changes.
- **Build:** `/tmp/arch-notebook-target/release/arch-notebook`, SHA-256 `a97907b3bc36f96d409c78f50865a3afe7385357ba18e963fc5bf27d56e14b1f`.
- **Fixture:** Synthetic fictional chart workloads only; canvas 1440×900; zoom 1; 5 samples per case.

## Results

| Gate | Result | Evidence |
| --- | --- | --- |
| Conservative route intersection | PASS | `chart_link_route_culling_preserves_crossings_and_nearby_decorations` covers straight and curved routes with outside endpoints, an offscreen-anchored self-loop reaching the viewport, a high-lane self-loop label, nearby attribute decorations, fully offscreen routes, empty paths, and conservative non-finite handling. |
| Release workload and cull counts | PASS | Eight cases cover sparse, hub, disconnected, and dense graphs at 100 and 1,000 entities. Neutral pan retained every valid record; a 2,000-pixel horizontal pan classified all valid records as offscreen. No missing endpoints occurred. The cull unions sampled route bounds with actual label bounds, adds allowance for attribute rows, and expands for arrow/stroke extent by `90 × |zoom| + 8` logical points. |
| Rust tests | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` exited 0; 56 passed. |
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

Both commands exited 0 and returned eight cases. The pan-zero JSON SHA-256 is `335ac674f72e7f4bc08f6303e3cd9f1621cde550e4c9b47370882a920a697e6b`; the panned JSON SHA-256 is `5edd95ca41d6303a36c910ebc4b2c324d722e0c0d128eacc3f57b56fc3f270ed`. Raw JSON is preserved in [neutral-pan results](artifacts/G4-viewport-culling-neutral-20261126.json) and [panned results](artifacts/G4-viewport-culling-panned-20261126.json).

The cull evaluates each relationship record separately, preserving parallel, reversed, duplicate, and self-loop routes independently. Picking remains unchanged. These results validate candidate classification on named synthetic fixtures; they do not measure paint-time improvement, GPU/compositor cost, frame rate, input-to-presentation latency, or every possible chart decoration. Native visual/input review remains open.

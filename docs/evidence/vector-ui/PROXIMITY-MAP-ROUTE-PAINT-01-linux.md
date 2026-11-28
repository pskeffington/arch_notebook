# Main chart route paint cleanup — Linux evidence

- **Task / owner:** Reduce chart relationship paint overhead without changing route geometry; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Baseline:** `0763a00dfa893d3b0e86bf27ee15e6844a23cb82`.
- **Changed paths:** `src/main.rs`, `docs/build-acceptance.md`, this evidence file.

## Change

`connection_curve_points` now returns a fixed 17-point array, removing one heap vector allocation for each generated route. For noncoincident confidence-3 relationships, the paint loop recovers the quadratic control point from sample 8 and submits one native egui quadratic Bézier shape instead of 16 line-segment shapes. The exact route samples remain in use for culling, picking, labels, and arrowheads. Confidence-1 dotted links, confidence-2 dashed links, self-loops, and coincident-endpoint curves retain their sampled-segment paint paths.

## Results

| Gate | Result | Evidence |
| --- | --- | --- |
| Formatting / whitespace | PASS | `cargo fmt --all -- --check && git diff --check`, exit 0. |
| Focused route tests | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline connection_curve -- --nocapture` and `... cargo test --locked --offline chart_link_route_culling_preserves_crossings_and_nearby_decorations -- --nocapture`, each exit 0. |
| Full suite | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1`, exit 0: 71 passed, 0 failed, 3 ignored. |
| Check / build | PASS | Locked offline `cargo check` and debug `cargo build`, exit 0. |
| Clinton production UI benchmark | PASS, headless | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target ARCH_NOTEBOOK_CLINTON_CASE_PATH=/home/sysadmin/Research.Databases/TRANSCRIBE/clinton-emails-arch-notebook.case.json cargo test --locked --offline clinton_case_topology_map_renders_at_real_corpus_scale -- --ignored --nocapture`, exit 0. Loaded 2,813 entities and 6,173 links; rendered the sampled 48-group / 47-pair topology with 1,315 represented records at a 1440×1600 egui viewport. The benchmark is a successful regression run, not evidence of a speedup: the Clinton corpus links all use confidence 1, so this run does not exercise the new solid-link paint branch. |
| Topology source review | PASS | Reviewer confirmed the midpoint-derived control point mathematically reproduces the sampled quadratic and the current style width/color; self-loops remain on the prior path. Read-only source review only. |
| Native visual / GPU review | NOT RUN | No native visual or GPU result is claimed. |
| Exact-head CI | PENDING | The workflow runs on `main` pushes or pull requests, not this non-main branch push; no run is attached to this candidate commit. |

The paired Clinton benchmark reported (topology enabled / disabled): median frame-build 218,531,001 / 214,383,942 ns; median egui CPU tessellation 154,079,910 / 147,704,526 ns. These values include environment and analysis-pane overhead and cannot isolate this route-paint change. They are recorded for traceability only.

# Sampled contact map viewport culling — Linux evidence

- **Task / owner:** Skip offscreen sampled topology routes while preserving visible route selection; implementation session.
- **Repository / branch:** `/home/sysadmin/github/arch_notebook` / `codex/ui-00-chart-baseline`.
- **Baseline:** `8af57a12649dadd5478d08289caf2cdf6de2060b`.
- **Changed paths:** `src/main.rs`, `docs/build-acceptance.md`, this report.

## Implementation

Each interaction route is sampled as before, then its finite-point bounds are expanded by 10 screen points and checked against the sampled map's `ui.clip_rect()`. Routes entirely outside that region are skipped. Routes crossing into view, arrowheads, and routes within the 8-point picking tolerance remain eligible. Painting traverses retained routes directly without allocating a second vector of route geometry. Only on a map click, the same route loop tracks the nearest candidate in one scalar pair; node interaction is still checked first and wins. Route generation still runs before the bounds check, so this removes offscreen paint and pick work but does not eliminate route preparation.

## Results

| Gate | Result | Evidence |
| --- | --- | --- |
| Focused viewport regression | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline proximity_map_route_culling_keeps_crossings_and_pickable_edges -- --nocapture`, exit 0. Covers a long route crossing the viewport, an offscreen route, a route inside the pick margin, and empty input. |
| Full Rust suite | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1`, exit 0: 72 passed, 0 failed, 3 ignored. |
| Production UI / route click regression | PASS | Included in the full suite; existing headless topology test selects a visible routed contact through the production Analysis pane. |
| Clinton corpus production UI benchmark | PASS, headless | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target ARCH_NOTEBOOK_CLINTON_CASE_PATH=/home/sysadmin/Research.Databases/TRANSCRIBE/clinton-emails-arch-notebook.case.json cargo test --locked --offline clinton_case_topology_map_renders_at_real_corpus_scale -- --ignored --nocapture`, exit 0. The test loaded 2,813 entities and 6,173 links and exercised the sampled 48-group, 47-pair topology containing 1,315 represented records. |
| Check/build/format | PASS | `cargo fmt --all -- --check`, `git diff --check`, locked offline `cargo check`, and locked offline debug `cargo build` all exited 0. |
| Independent topology source review | PASS | Read-only review confirmed viewport clipping, crossing route retention, nearest-visible-route picking, and node-click priority. It noted route generation still precedes culling. |
| Clinton clipped-shape comparison | PASS, observed | Same retained-corpus paired UI test on the preceding product renderer `89ddc99` reported 88,792 shapes with topology; this candidate reported 88,204. The no-topology count stayed 87,855. Shape-count decrease is consistent with culling; frame-time samples varied substantially and do not establish a speedup. |
| Native visual/GPU review | NOT RUN | No native display or GPU result is claimed. |
| Exact-head CI | PENDING | The build workflow runs on main pushes or pull requests, not this non-main branch push. No PR was opened. |

Candidate benchmark values, topology enabled/disabled: 88,204 / 87,855 clipped shapes; median frame build 223,318,563 / 222,521,482 ns; median egui CPU tessellation 153,089,600 / 152,385,938 ns. The samples are too variable for latency conclusions.

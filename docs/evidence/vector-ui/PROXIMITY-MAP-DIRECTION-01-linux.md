# PROXIMITY-MAP-DIRECTION-01 — show direction in the group contact map

- **Task / owner:** Make directed ties legible in the hop-layer topology map; implementation session.
- **Repository / branch:** `arch_notebook` / `codex/ui-00-chart-baseline`.
- **Baseline:** `034d0294a56577ecf7499d4ab81a35a323d5fb32`.
- **Source commit:** `238018953315cf325b49203b7ccdfa4f2c7146e8`.
- **Candidate binary:** Linux debug, SHA-256 `792e1f0fb0a28843a9ec97652ecef20c9175e91a27b65ed3a65a36630e088cb7`.

## Change

The origin-centered contact map now retains its neutral, record-weighted group-contact line and overlays arrowheads for directed records. Reciprocal directed traffic uses two slightly offset arrows so both directions remain visible; undirected records retain no arrowhead. The map's explanatory label describes this encoding. Exact record and direction counts remain available in the interaction list, and clicking a group still focuses its member entities on the chart.

## Verification

| Check | Result | Evidence |
| --- | --- | --- |
| Headless UI direction rendering | PASS | `headless_ui_drives_geodesic_and_detected_community_actions` renders reciprocal directed group links and asserts exactly six colored line segments (two arrows, each drawn as a shaft and two arrowhead strokes); the separate undirected group contact adds no directional strokes. The same test clicks Group 2 and checks chart focus. |
| Full offline Rust suite | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` exited 0: 69 passed, 0 failed, 1 opt-in benchmark ignored. |
| Locked debug build | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline` exited 0. |
| Formatting and diff validation | PASS | `cargo fmt --all -- --check` and `git diff --check` exited 0. |
| Exact-head cross-platform CI | PASS | [Run 46062451089](https://github.com/pskeffington/arch_notebook/actions/runs/46062451089) completed all six jobs successfully for `2380189`: Linux tests/SQL/ANX/installer, Linux x86_64, Linux ARM64, Arch package, macOS Intel, and macOS Apple Silicon. |
| Native visual/input review | NOT TESTED | This acceptance uses actual `App::update`/egui shapes in a headless test. It does not claim that the arrow contrast, spacing, or hover behavior has been inspected in a native window. |

The map continues to aggregate records by proximity group while preserving record-level drilldown. Arrowheads visualize recorded link direction; they do not imply causal flow, influence, or analyst intent.

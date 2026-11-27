# PROXIMITY-MAP-SPACING-01 — separate group labels from adjacent nodes

- **Task / owner:** Improve dense hop-layer contact-map row readability; implementation session.
- **Repository / branch:** `arch_notebook` / `codex/ui-00-chart-baseline`.
- **Baseline:** `39fd1ba7bdf379cbd6d341782a1fe7f257a00569`.
- **Candidate source:** `0e948681ee50cc8590fa2de65672c20a705129b9`.
- **Candidate binary:** Linux debug, SHA-256 `4d2406772cb3b34b3f7d625e1bac6164ae43d0c3c37af1e6fdde4bbb83f81557`.

## Change

Map rows are spaced 52 px apart instead of 44 px. The production headless UI regression measures the actual egui layout height of a representative metric label at the map's 9 px font and verifies at least 4 px of clearance from the prior row's maximum-size label to the next maximum-size node. Map height grows by the same spacing per visible row; the map remains bounded by the existing group and hop sampling limits and scroll viewport.

## Verification

| Check | Result | Evidence |
| --- | --- | --- |
| Actual egui label-clearance measurement | PASS | `headless_ui_drives_geodesic_and_detected_community_actions` measures the representative label galley height using egui's configured fonts and verifies the production spacing/radius/offset constants leave at least 4 px. |
| Headless map interaction | PASS | The same test checks legend and group metric rendering, reciprocal direction strokes, and group-click focus. |
| Full offline Rust suite | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` exited 0: 69 passed, 0 failed, 1 opt-in benchmark ignored. |
| Locked debug build | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline` exited 0 with no warnings. |
| Formatting and diff validation | PASS | `cargo fmt --all -- --check` and `git diff --check` exited 0. |
| Native visual review | NOT TESTED | The local X display remains unavailable; actual visual separation, contrast, and hover interaction need native inspection. |
| Exact-head CI | PENDING | Run [46064779770](https://github.com/pskeffington/arch_notebook/actions/runs/46064779770) targets `40b7bc1`, the production-spacing parent of this test-only measurement follow-up; a fresh run is required to cover `0e94868`. |

This changes only vertical layout spacing. Group membership, interaction counts, record drilldown, direction encodings, and click-to-focus behavior are unchanged.

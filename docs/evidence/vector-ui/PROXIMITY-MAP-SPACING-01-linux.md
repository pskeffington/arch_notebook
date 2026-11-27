# PROXIMITY-MAP-SPACING-01 — separate group labels from adjacent nodes

- **Task / owner:** Improve dense hop-layer contact-map row readability; implementation session.
- **Repository / branch:** `arch_notebook` / `codex/ui-00-chart-baseline`.
- **Baseline:** `39fd1ba7bdf379cbd6d341782a1fe7f257a00569`.
- **Candidate source:** pending commit on this branch.
- **Candidate binary:** Linux debug, SHA-256 `4d2406772cb3b34b3f7d625e1bac6164ae43d0c3c37af1e6fdde4bbb83f81557`.

## Change

Map rows are spaced 52 px apart instead of 44 px. With the 15 px maximum node radius, 5 px label offset, and estimated 11 px metric-label height, this leaves 6 px from the prior row's metric label to the next row's largest node. Map height grows by the same spacing per visible row; the map remains bounded by the existing group and hop sampling limits and scroll viewport.

## Verification

| Check | Result | Evidence |
| --- | --- | --- |
| Row geometry regression | PASS | `proximity_group_rows_clear_metric_labels_from_the_next_node` verifies that the configured row spacing leaves at least the required 4 px clearance. |
| Headless map interaction | PASS | `headless_ui_drives_geodesic_and_detected_community_actions` checks legend and group metric rendering, reciprocal direction strokes, and group-click focus. |
| Full offline Rust suite | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` exited 0: 70 passed, 0 failed, 1 opt-in benchmark ignored. |
| Locked debug build | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline` exited 0 with no warnings. |
| Formatting and diff validation | PASS | `cargo fmt --all -- --check` and `git diff --check` exited 0. |
| Native visual review | NOT TESTED | The local X display remains unavailable; actual visual separation, contrast, and hover interaction need native inspection. |
| Exact-head CI | NOT RUN | A new run must cover this source commit. Existing run [46064081779](https://github.com/pskeffington/arch_notebook/actions/runs/46064081779) targets its parent `39fd1ba`, not this spacing change. |

This changes only vertical layout spacing. Group membership, interaction counts, record drilldown, direction encodings, and click-to-focus behavior are unchanged.

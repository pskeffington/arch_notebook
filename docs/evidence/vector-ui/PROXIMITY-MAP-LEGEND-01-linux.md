# PROXIMITY-MAP-LEGEND-01 — clarify group map encodings

- **Task / owner:** Improve at-a-glance interpretation of the hop-layer contact map; implementation session.
- **Repository / branch:** `arch_notebook` / `codex/ui-00-chart-baseline`.
- **Baseline:** `1ece38ba1565d927a3260c7396cb1745d6a23aad`.
- **Candidate source:** pending commit on this branch.
- **Candidate binary:** Linux debug, SHA-256 `d49f7cf0fc31df900e8dedd5871d65cdd95aa57ff8bf8c28a5e2859f6eb8ac93`.

## Change

The group contact map now explains the visual variables in place: larger nodes contain more entities, thicker lines represent more relationship records, arrowheads show recorded direction, and lines without arrowheads represent undirected records. Group labels use a clearer `internal` count to distinguish within-group records from the cross-group counts represented by connecting lines. Hover text retains the exact group size and internal-record count. Node selection still fits the group's members on the chart.

## Verification

| Check | Result | Evidence |
| --- | --- | --- |
| Headless production UI | PASS | `headless_ui_drives_geodesic_and_detected_community_actions` verifies that the map legend appears, a group node displays `3 entities · 3 internal`, reciprocal arrow strokes are rendered, and clicking the group still focuses its member entities. |
| Full offline Rust suite | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` exited 0: 69 passed, 0 failed, 1 opt-in benchmark ignored. |
| Locked debug build | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline` exited 0. |
| Formatting and diff validation | PASS | `cargo fmt --all -- --check` and `git diff --check` exited 0. |
| Native visual/input review | NOT TESTED | The local X display is unavailable (`xwininfo` cannot open `:0`); this record does not claim native spacing, contrast, or hover inspection. |
| Exact-head CI | NOT RUN | This candidate has not yet been pushed, so no workflow run covers it. |

The legend describes relative visual encodings rather than exact values. Detailed relationship counts and record-level drilldown remain in the group interaction list. The structural map does not imply causal flow or intent.

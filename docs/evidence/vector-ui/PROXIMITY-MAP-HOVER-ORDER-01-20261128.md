# PROXIMITY-MAP-HOVER-ORDER-01 · 2026-11-28

- **Task / owner:** Keep hovered sampled contact routes legible at crossings without covering nodes; Codex single-session implementation, independent topology source review.
- **Repository / branch:** `arch_notebook` / `codex/ui-00-chart-nested-conditions`.
- **Baseline:** `2b6cb00b59aaaf60ea05603314ab8c346e23f34c`.
- **Scope:** proximity-map hover layering, exact node hit-rectangle priority, regression coverage, and acceptance notes.

## Change

Hovered group-contact routes now paint after base routes and before selected routes and group nodes. This lets the highlighted route stand out at crossings while leaving group fills, proxy rings, and node targets on top. Hover suppression checks the same square hit rectangle used by node interaction. Hover routes use a distinct color from direction arrows so the headless paint-order assertion specifically checks the complete 12-segment highlight.

## Acceptance

| Gate | Result | Evidence |
| --- | --- | --- |
| Headless hover preview, picking, and layering | PASS | `headless_ui_drives_geodesic_and_detected_community_actions` passed. It asserts exactly 12 distinct hover-color route segments and that the last route segment paints before group fills; the route click still selects its group-pair interaction. |
| Dense production contact map | PASS | `dense_proximity_map_culls_before_sampling_without_losing_visible_routes`: 48 groups / 576 pairs / 156 viewport candidates / 420 culled / 1,872 painted route segments in a 291×193 clip. |
| Retained Clinton production Analysis-pane view | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target ARCH_NOTEBOOK_CLINTON_CASE_PATH=/home/sysadmin/Research.Databases/TRANSCRIBE/clinton-emails-arch-notebook.case.json cargo test --locked --offline clinton_case_topology_map_renders_at_real_corpus_scale -- --ignored --nocapture`, exit 0. The test loaded 2,813 entities and 6,173 records, rendered 48 sampled groups and 47 pairs representing 1,315 records, and exposed selected-group details after scrolling to that panel. |
| Full Rust suite | PASS | `CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline`, exit 0; 78 passed, 0 failed, 3 ignored. |
| Formatting / diff | PASS | `cargo fmt --all` and `git diff --check`, exit 0. |
| Independent topology source review | PASS | Reviewer confirmed exact square hit-target correspondence, distinct hover stroke assertions, and group nodes/proxy rings painted after routes. |
| Native visual and pointer review | BLOCKED | Native display/input access is unavailable in this session. Headless evidence does not establish real pointer feel or contrast. |
| Exact-head CI | PENDING | Base commit `2b6cb00` passed run [46111073491](https://github.com/pskeffington/arch_notebook/actions/runs/46111073491). A new run is required for this follow-up commit. |

The first retained-corpus UI attempt exposed an existing test viewport assumption: after scrolling the topology heading into a 900-point view, the test expected the lower selected-group contact section in the same frame. The test now scrolls independently to verify the map and the selected-group panel. No case or SQLite source was modified.

## Limitations

The retained case is used for structural rendering acceptance, not email completeness, semantic content review, Qwen analysis, or IBM application compatibility. No latency, GPU, FPS, or native visual claim is made.

## Next

Collect exact-head CI for this commit and perform native visual/pointer review when the display/input path is available.

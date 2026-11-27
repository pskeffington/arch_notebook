# PROXIMITY-MAP-HIT-GEOMETRY-01 — preserve node and label clearance

- **Task / owner:** Verify topology node hit geometry and scroll extents; implementation session.
- **Repository / branch / baseline:** `arch_notebook` / `codex/ui-00-chart-baseline` / `5c63ac219a5eb0a72c0602a4905aef612d65e6d1`.
- **Candidate source:** recorded after commit.
- **Scope:** `src/main.rs`, this report, and `docs/build-acceptance.md`.

## Change

Node positions, display radii, pointer-target radii, and map height are now computed through named production helpers. Proxy groups use a larger target to include the outer ring. The headless production-UI regression checks at least 4 points between the largest possible adjacent-row hit targets, between a group metric and the next hit target, and between the final target and the scroll-content boundary.

The sampled-map label now reports sampled versus total groups, mapped versus total group-contact pairs, and mapped versus total relationship records, including omitted counts. The full contact index remains available below. The production-UI regression checks that the coverage line matches the analyzed sample and full graph totals.

## Results

| Check | Result | Evidence |
| --- | --- | --- |
| Production headless UI geometry, sample coverage and navigation | PASS | `headless_ui_drives_geodesic_and_detected_community_actions` passed, including geometry and computed coverage assertions, group focus, line selection, and exact-record inspection. |
| Retained Clinton production topology UI | PASS | `clinton_case_topology_map_renders_at_real_corpus_scale` passed against the local 2,813-entity, 6,173-record case at 1440×900. |
| Retained Clinton topology sampler | PASS | `clinton_case_topology_sampling_stays_bounded_and_covers_hops` passed with the 48-group sample spanning all 10 displayed hop columns. |
| Full offline Rust suite | PASS | 69 passed, 0 failed, 3 ignored. |
| Offline check and debug build | PASS | Locked offline `cargo check` and `cargo build` exited 0. |
| Formatting and diff validation | PASS | `cargo fmt --all -- --check` and `git diff --check` exited 0. |
| Native pointer/display review | BLOCKED | X11/Wayland access unavailable; geometry assertions do not establish native appearance or pointer feel. |
| Cross-platform CI | PENDING | Run [46076165646](https://github.com/pskeffington/arch_notebook/actions/runs/46076165646) targets geometry commit `a577698`; it is queued with no jobs started. The subsequent coverage-label follow-up needs its own exact-head CI run. |

The Clinton tests read the local case and do not emit names or IDs into repository evidence.

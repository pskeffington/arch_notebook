# PROXIMITY-MAP-SELECTION-FEEDBACK-01 — show selected groups and contacts

- **Task / owner:** Make sampled-map group/contact selection visible at the point of interaction; implementation session.
- **Repository / branch / baseline:** `arch_notebook` / `codex/ui-00-chart-baseline` / `189828e967e3de65f02b829ca0155a6c0c098dbf`.
- **Candidate source:** recorded after commit.
- **Scope:** `src/main.rs`, this evidence report, and the build/roadmap acceptance records.

## Change

Selecting a group now draws an outer pale-amber ring around its map node; proxy rings remain distinct inside it. Selecting a group-contact pair draws its line in the same contrasting color at a wider stroke. The existing selected-contact summary and exact-record list remain the details pane for the highlighted edge.

## Results

| Check | Result | Evidence |
| --- | --- | --- |
| Production headless group selection | PASS | `headless_ui_drives_geodesic_and_detected_community_actions` confirms the selected group detail is present and an actual selected-color circle stroke is painted. |
| Production headless contact selection | PASS | The same test clicks a contact row, confirms the exact selected pair and record count, and verifies a selected-color line stroke is painted. |
| Retained Clinton topology UI and sampler | PASS | 2,813 entities, 6,173 records; the 48-group sample covers all 10 displayed hops; measured sample crossings remain 23 before / 3 after row ordering. |
| Full offline Rust suite | PASS | 70 passed, 0 failed, 3 ignored. |
| Locked offline check/debug build and format | PASS | `cargo check`, `cargo build`, `cargo fmt --all -- --check`, and `git diff --check` pass. |
| Cross-platform CI | PENDING | Dispatch exact-head CI after push. |
| Native pointer/visual review | BLOCKED | Screen capture has worked intermittently, but X11 automation cannot connect, Wayland input tools are unavailable, `/dev/uinput` is absent, and session D-Bus access is denied. Headless production-UI events verify rendering and state, but not native pointer feel or contrast. |

No corpus-specific names, IDs, or relationship contents are included in repository evidence.

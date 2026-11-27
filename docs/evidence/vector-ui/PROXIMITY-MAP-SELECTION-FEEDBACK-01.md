# PROXIMITY-MAP-SELECTION-FEEDBACK-01 — show selected groups and contacts

- **Task / owner:** Make sampled-map group/contact selection visible at the point of interaction; implementation session.
- **Repository / branch / baseline:** `arch_notebook` / `codex/ui-00-chart-baseline` / `7b2cab105e57efb41513854a791a7629adcfd307`.
- **Candidate source:** recorded after commit.
- **Scope:** `src/main.rs`, this evidence report, and the build/roadmap acceptance records.

## Change

Selecting a group now draws an outer pale-amber ring around its map node; proxy rings remain distinct inside it. Selecting a sampled group-contact pair draws its line in the same contrasting color at a wider stroke. When selections come from the complete virtualized index but are outside the bounded map sample, the details pane states that they are not drawn in the map while keeping exact records available.

## Results

| Check | Result | Evidence |
| --- | --- | --- |
| Production headless group selection | PASS | `headless_ui_drives_geodesic_and_detected_community_actions` confirms the selected group detail is present and an actual selected-color circle stroke is painted. |
| Production headless contact selection | PASS | The same test clicks a contact row, confirms the exact selected pair and record count, and verifies a selected-color line stroke is painted. |
| Retained Clinton topology UI and sampler | PASS | Real-corpus UI regression selects an omitted group and omitted pair, checks truthful outside-sample labels and exact-record detail; 2,813 entities, 6,173 records, 48 sampled groups across 10 hops. |
| Full offline Rust suite | PASS | 70 passed, 0 failed, 3 ignored. |
| Locked offline check/debug build and format | PASS | `cargo check`, `cargo build`, `cargo fmt --all -- --check`, and `git diff --check` pass. |
| Cross-platform CI | PENDING | Dispatch exact-head CI after push. |
| Native pointer/visual review | BLOCKED | Screen capture has worked intermittently, but X11 automation cannot connect, Wayland input tools are unavailable, `/dev/uinput` is absent, and session D-Bus access is denied. Headless production-UI events verify rendering and state, but not native pointer feel or contrast. |

No corpus-specific names, IDs, or relationship contents are included in repository evidence.

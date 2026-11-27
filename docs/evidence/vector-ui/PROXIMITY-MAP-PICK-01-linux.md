# PROXIMITY-MAP-PICK-01 — select and inspect contact lines

- **Task / owner:** Connect group contact-map lines to exact relationship-record review; implementation session.
- **Repository / branch:** `arch_notebook` / `codex/ui-00-chart-baseline`.
- **Baseline:** `15d9eef2dd6900f63f09b5df2c295ef539bf7e47`.
- **Candidate source:** `159955cd446c2661a224d9e4275d4a50479a8094`.
- **Candidate binary:** Linux debug, SHA-256 `9d83dd5ff86867981e98767a8c056d896be4b04e660f8e3d023cb3b4f35083bb`.

## Change

Clicking within 8 points of a visible group-contact segment selects the nearest rendered interaction. The existing interaction detail below the map then shows the exact-record count and record-inspection buttons. The selected tie is drawn wider in a dedicated pale amber color, and a note identifies it while the interaction is selected. Group-node clicks take precedence and retain their existing chart-fit behavior.

Hovering a contact line previews both group numbers, relationship-record total, distinct endpoint-pair total, and forward/reverse/undirected counts in a reserved status line below the map. The line preview does not depend on waiting for a tooltip and does not shift the remaining analysis-pane content.

## Verification

| Check | Result | Evidence |
| --- | --- | --- |
| Headless production-UI line navigation and hover preview | PASS | `headless_ui_drives_geodesic_and_detected_community_actions` renders the map at a 1440×1600 viewport, hovers a weighted group-contact line and checks the exact aggregate preview, then clicks it and verifies the exact `(first_group, second_group)` selection, exact-record detail heading, and selected-line highlight. The same fixture checks reciprocal arrows and group-node click-to-focus. |
| Full offline Rust suite | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` exited 0: 69 passed, 0 failed, 1 opt-in benchmark ignored. |
| Locked debug build | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline` exited 0. |
| Formatting and diff validation | PASS | `cargo fmt --all -- --check` and `git diff --check` exited 0. |
| Native visual/input review | NOT TESTED | The Wayland socket is inaccessible from this sandbox and no nested display server is installed. Native spacing, color contrast, and pointer feel remain unreviewed. |
| Exact-head cross-platform CI | PENDING | The candidate has been pushed; workflow run reference will be recorded after dispatch and completion. |

The line is picked by nearest distance to its rendered straight segment. The detail panel remains the source of exact direction, pair, and record counts; the graph line itself does not assert causation or intent.

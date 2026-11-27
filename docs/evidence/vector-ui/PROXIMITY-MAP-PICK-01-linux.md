# PROXIMITY-MAP-PICK-01 — select and inspect contact lines

- **Task / owner:** Connect group contact-map lines to exact relationship-record review; implementation session.
- **Repository / branch:** `arch_notebook` / `codex/ui-00-chart-baseline`.
- **Baseline:** `15d9eef2dd6900f63f09b5df2c295ef539bf7e47`.
- **Candidate source:** `bfd752aa86306cefe8d8b58df18c32447cab6a6e`.
- **Candidate binary:** Linux debug, SHA-256 `cd452cbb99591b6cb0750cc57dd471541cd99de1f53b9a736ab812dfce90690f`.

## Change

Clicking within 8 points of a visible group-contact segment selects the nearest rendered interaction. The existing interaction detail below the map then shows the exact-record count and record-inspection buttons. The selected tie is drawn wider in a dedicated pale amber color, and a note identifies it while the interaction is selected. Group-node clicks take precedence and retain their existing chart-fit behavior.

## Verification

| Check | Result | Evidence |
| --- | --- | --- |
| Headless production-UI line navigation | PASS | `headless_ui_drives_geodesic_and_detected_community_actions` renders the map at a 1440×1600 viewport, clicks a weighted group-contact line, verifies the exact `(first_group, second_group)` selection, confirms the exact-record detail heading, and confirms the selected line uses the highlight stroke. The same fixture checks reciprocal arrows and group-node click-to-focus. |
| Full offline Rust suite | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` exited 0: 69 passed, 0 failed, 1 opt-in benchmark ignored. |
| Locked debug build | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo build --locked --offline` exited 0. |
| Formatting and diff validation | PASS | `cargo fmt --all -- --check` and `git diff --check` exited 0. |
| Native visual/input review | NOT TESTED | The Wayland socket is inaccessible from this sandbox and no nested display server is installed. Native spacing, color contrast, and pointer feel remain unreviewed. |
| Exact-head cross-platform CI | PENDING | The candidate has been pushed; workflow run reference will be recorded after dispatch and completion. |

The line is picked by nearest distance to its rendered straight segment. The detail panel remains the source of exact direction, pair, and record counts; the graph line itself does not assert causation or intent.

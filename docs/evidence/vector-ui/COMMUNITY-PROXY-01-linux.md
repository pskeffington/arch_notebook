# COMMUNITY-PROXY-01 · chart-wide bridge entities

- **Task / owner:** Add unique-neighbor and cross-community proxy ranking to detected-community analysis; implementation session.
- **Source branch / product candidate:** `codex/ui-00-chart-baseline`, `e27ad5e842dd1c6d503f05c51844f12790502fa6` (extends proxy ranking from `4c132cf`). Verification source `d0ce0c77e9fcee445536d0f55aceebf70e5fbd84` adds a separate chart-paint culling regression.
- **Scope:** Rust analysis and Analysis-pane UI, analyst parity documentation, and deterministic regressions.

## Result

| Gate | Result | Evidence |
| --- | --- | --- |
| Unique-neighbor counts | PASS | A three-community fixture confirms the boundary entity has 5 distinct direct neighbors; repeated records do not inflate this count. |
| Community bridge ranking | PASS | The same fixture confirms the bridge contacts 2 other communities and has 5 relationship records; the UI presents community, unique-neighbor, and record counts separately. |
| Headless UI visibility and navigation | PASS | The headless event test opens **Detect communities**, scrolls to **CROSS-COMMUNITY PROXIES**, selects **Focus · A3**, returns to Chart, selects the bridge, and verifies that the next app frame pans the chart to it. |
| Focused regression | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline headless_ui_drives_geodesic_and_detected_community_actions -- --nocapture` exited 0; 1 passed. |
| Full Rust suite | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo test --locked --offline -- --test-threads=1` exited 0; 57 passed, 0 failed. |
| Compile check | PASS | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/tmp/arch-notebook-target cargo check --locked --offline` exited 0. |
| Debug and release builds | PASS | Locked offline debug and release builds exited 0. |
| Formatting / patch whitespace | PASS | `cargo fmt --all -- --check` and `git diff --check` exited 0. |
| Native visual review | NOT RUN | This session did not validate the ranking in an interactive desktop window. |

## Method

For each valid, non-self relationship record, both endpoints receive the other endpoint as a distinct neighbor. Cross-community records add the counterpart's community to each endpoint's external-contact set. An entity is listed when it contacts at least two other detected communities. Rows sort by distinct external communities, distinct neighboring entities, relationship-record count, then entity UUID. Direction and duplicate records do not change the unique-neighbor or community-contact counts; relationship records remain counted separately. The community partition remains the existing deterministic greedy-modularity heuristic and is an exploratory structural grouping, not a claim about real-world affiliation or influence.

## Changed paths

- `src/main.rs`
- `README.md`
- `docs/analyst-notebook-parity.md`
- `docs/evidence/vector-ui/COMMUNITY-PROXY-01-linux.md`

Native UI review and independent code review remain open.
